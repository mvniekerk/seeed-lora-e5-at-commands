#![no_std]
#![no_main]
extern crate alloc;

#[global_allocator]
static HEAP: Heap = Heap::empty();

use defmt::{error, info, unwrap};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::UART1;
use embassy_rp::uart::DataBits::DataBits8;
use embassy_rp::uart::{
    BufferedInterruptHandler, BufferedUart, BufferedUartRx, BufferedUartTx, Config, Parity,
    StopBits,
};
use {defmt_rtt as _, panic_probe as _};

use atat::{asynch::Client, Ingress};
use atat::{AtatIngress, ResponseSlot, UrcChannel};
use embassy_time::{Duration, Timer};
use embedded_alloc::Heap;
use seeed_lora_e5_at_commands::client::asynch::{JoinStatus, SeeedLoraE5Client};
use seeed_lora_e5_at_commands::digester::LoraE5Digester;
use seeed_lora_e5_at_commands::lora::types::{LoraClass, LoraJoinMode, LoraRegion};
use seeed_lora_e5_at_commands::urc::URCMessages;
use static_cell::StaticCell;

const APP_KEY: u128 = 0xd65b042878144e038a744359c7cd1f9d;
const DEV_EUI: u64 = 0x68419fa0f7e74b0d;

// Chunk size in bytes when receiving data. Value should be matched to buffer
// size of receive() calls.
// TODO should be 1012
const RX_SIZE: usize = 1012;

// Constants derived from TX_SIZE and RX_SIZE
const INGRESS_BUF_SIZE: usize = RX_SIZE;
const URC_SUBSCRIBERS: usize = 0;
// const URC_CAPACITY: usize = RX_SIZE * 1;
const URC_CAPACITY: usize = 40;

type AtIngress<'a> =
    Ingress<'a, LoraE5Digester, URCMessages, INGRESS_BUF_SIZE, URC_CAPACITY, URC_SUBSCRIBERS>;

type AtLoraE5Client<'a> = Client<'a, BufferedUartTx<'a, UART1>, INGRESS_BUF_SIZE>;

bind_interrupts!(struct Irqs {
    UART1_IRQ => BufferedInterruptHandler<UART1>;
});

macro_rules! singleton {
    ($val:expr) => {{
        type T = impl Sized;
        static STATIC_CELL: StaticCell<T> = StaticCell::new();
        let (x,) = STATIC_CELL.init(($val,));
        x
    }};
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let (tx_pin, rx_pin, uart) = (p.PIN_8, p.PIN_9, p.UART1);

    let tx_buf = &mut singleton!([0u8; 32])[..];
    let rx_buf = &mut singleton!([0u8; 280])[..];
    let mut config = Config::default();
    config.baudrate = 9600;
    config.parity = Parity::ParityNone;
    config.stop_bits = StopBits::STOP1;
    config.data_bits = DataBits8;
    let uart = BufferedUart::new(uart, Irqs, tx_pin, rx_pin, tx_buf, rx_buf, config);
    let (rx, tx) = uart.split();

    let digester = LoraE5Digester::default();

    static RES_SLOT: ResponseSlot<INGRESS_BUF_SIZE> = ResponseSlot::new();
    static INGRESS_BUF: StaticCell<[u8; INGRESS_BUF_SIZE]> = StaticCell::new();
    static URC_CHANNEL: UrcChannel<URCMessages, URC_CAPACITY, URC_SUBSCRIBERS> = UrcChannel::new();
    let ingress = Ingress::new(
        digester,
        INGRESS_BUF.init([0; INGRESS_BUF_SIZE]),
        &RES_SLOT,
        &URC_CHANNEL,
    );
    static BUF: StaticCell<[u8; 1024]> = StaticCell::new();
    let client = Client::new(tx, &RES_SLOT, BUF.init([0; 1024]), atat::Config::default());
    unwrap!(spawner.spawn(read_task(ingress, rx)));
    unwrap!(spawner.spawn(client_task(client)));
}

#[embassy_executor::task]
async fn read_task(mut ingress: AtIngress<'static>, mut rx: BufferedUartRx<'static, UART1>) {
    ingress.read_from(&mut rx).await;
}

#[embassy_executor::task]
async fn client_task(client: AtLoraE5Client<'static>) {
    let client = SeeedLoraE5Client::new(client).await;
    if let Err(e) = client {
        error!("Error creating client {}", e);
        return;
    }
    let mut client = client.unwrap();

    if let Err(e) = client.join_mode_set(LoraJoinMode::Otaa).await {
        error!("Error setting join mode {}", e);
    } else {
        info!("Join mode set to OTAA");
    }

    if let Err(e) = client.dev_eui_set(DEV_EUI).await {
        error!("Error setting dev eui {}", e);
    } else {
        info!("Dev EUI set");
    }

    if let Err(e) = client.app_eui_set(0x0).await {
        error!("Error setting app eui {}", e);
    } else {
        info!("App EUI set");
    }

    if let Err(e) = client.app_key_set(APP_KEY).await {
        error!("Error setting app key {}", e);
    } else {
        info!("App key set");
    }

    if let Err(e) = client.lora_region_set(LoraRegion::Eu868).await {
        error!("Error setting lora region {}", e);
    } else {
        info!("Lora region set");
    }

    if let Err(e) = client.lora_class_set(LoraClass::ClassC).await {
        error!("Error setting lora class {}", e);
    } else {
        info!("Lora class set to Class C");
    }

    if let Err(e) = client.adr_set(false).await {
        error!("Error setting lora adr {}", e);
    } else {
        info!("Lora adr set to false");
    }

    if let Err(e) = client.dr_set(5).await {
        error!("Error setting lora dr {}", e);
    } else {
        info!("Lora dr set to 5");
    }

    if let Err(e) = client.confirm_send_set(false).await {
        error!("Error confirm set {}", e);
    } else {
        info!("Lora send ACK set to false");
    }

    if let Err(e) = client.auto_join_set(false, 3).await {
        error!("Error setting auto join {}", e);
    } else {
        info!("Auto join disabled");
    }

    loop {
        if matches!(
            client.lora_join_otaa_and_wait_for_result().await,
            Ok(JoinStatus::Success)
        ) {
            break;
        }
        error!("Failed to join, retrying");
    }

    let mut uplink_frame_count = 0;
    let mut downlink_frame_count = 0;
    loop {
        let uplink_frame_count_get = client.uplink_frame_count().await;
        if let Ok(uplink_frame_count_get) = uplink_frame_count_get {
            if uplink_frame_count_get != uplink_frame_count {
                info!("Uplink frame count: {:?}", uplink_frame_count_get);
                uplink_frame_count = uplink_frame_count_get;
            }
        }
        let _ = client.confirm_send_set(true).await;
        match client.send(1, 12, b"Hello from Lora-E5").await {
            Ok(_d) => {
                info!("Sent bytes");
            }
            Err(e) => error!("Error sending {}", e),
        }
        for _i in 0..4 {
            let downlink_frame_count_get =
                client.downlink_message_count().await.unwrap_or_default();
            if downlink_frame_count_get != downlink_frame_count {
                info!(
                    "Downlink frame count changed: {:?}",
                    downlink_frame_count_get
                );
                downlink_frame_count = downlink_frame_count_get;
                let rx = client.receive().await;
                if rx.is_err() {
                    error!("Error getting received bytes");
                }
                let (data, stats) = rx.unwrap();
                let bytes = &data.payload;
                info!(
                    "Received bytes: {:?}, port: {:?}, RXWIN: {}, RSSI: {}, SNR: {}",
                    data.length, data.port, stats.rxwin, stats.rssi, stats.snr
                );

                let l = core::str::from_utf8(&bytes[0..data.length]).unwrap();
                info!("Bytes as string: {:?}", l);
            }
            Timer::after(Duration::from_secs(5)).await;
        }
    }
}
