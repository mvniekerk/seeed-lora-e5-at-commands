#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

extern crate alloc;

#[global_allocator]
static HEAP: Heap = Heap::empty();

use defmt::{error, info, unwrap};
use embassy_executor::Spawner;
use embassy_executor::_export::StaticCell;
use embassy_rp::interrupt;
use embassy_rp::peripherals::UART1;
use embassy_rp::uart::DataBits::DataBits8;
use embassy_rp::uart::{BufferedUart, BufferedUartRx, BufferedUartTx, Config, Parity, StopBits};
use {defmt_rtt as _, panic_probe as _};

use atat::helpers::LossyStr;
use atat::{AtatIngress, UrcSubscription};
use atat::{asynch::Client, Buffers, Ingress};
use embassy_time::{Duration, Timer};
use embedded_alloc::Heap;
use seeed_lora_e5_at::client::asynch::{JoinStatus, SeeedLoraE5Client};
use seeed_lora_e5_at::digester::LoraE5Digester;
use seeed_lora_e5_at::lora::types::{LoraClass, LoraJoinMode, LoraJoiningStatus, LoraRegion};
use seeed_lora_e5_at::urc::{LAST_LORA_MESSAGE_RECEIVED, LORA_JOIN_STATUS, LORA_MESSAGE_RECEIVED_COUNT, URCMessages};
use atat::AtatUrcChannel;

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
    let (tx_pin, rx_pin, uart) = (p.PIN_4, p.PIN_5, p.UART1);

    let irq = interrupt::take!(UART1_IRQ);
    let tx_buf = &mut singleton!([0u8; 32])[..];
    let rx_buf = &mut singleton!([0u8; 280])[..];
    let mut config = Config::default();
    config.baudrate = 9600;
    config.parity = Parity::ParityNone;
    config.stop_bits = StopBits::STOP1;
    config.data_bits = DataBits8;
    let uart = BufferedUart::new(uart, irq, tx_pin, rx_pin, tx_buf, rx_buf, config);
    let (rx, tx) = uart.split();

    // Atat client
    let config = atat::Config::default()
        .flush_timeout(Duration::from_millis(2000))
        .cmd_cooldown(Duration::from_millis(200))
        .tx_timeout(Duration::from_millis(2000));

    let digester = LoraE5Digester::default();
    static BUFFERS: Buffers<URCMessages, INGRESS_BUF_SIZE, URC_CAPACITY, URC_SUBSCRIBERS> =
        atat::Buffers::<URCMessages, INGRESS_BUF_SIZE, URC_CAPACITY, URC_SUBSCRIBERS>::new();
    let (ingress, client) = BUFFERS.split(tx, digester, config);

    unwrap!(spawner.spawn(read_task(ingress, rx)));
    unwrap!(spawner.spawn(client_task(client, spawner.clone())));
}

#[embassy_executor::task]
async fn read_task(mut ingress: AtIngress<'static>, mut rx: BufferedUartRx<'static, UART1>) {
    ingress.read_from(&mut rx).await;
}

#[embassy_executor::task]
async fn client_task(client: AtLoraE5Client<'static>, spawner: Spawner) {

    let client = SeeedLoraE5Client::new(client).await;
    if let Err(e) = client {
        error!("Error creating client");
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
        error!("Error setting app key");
    } else {
        info!("App key set");
    }

    if let Err(e) = client.lora_region_set(LoraRegion::Eu868).await {
        error!("Error setting lora region");
    } else {
        info!("Lora region set");
    }

    if let Err(e) = client.lora_class_set(LoraClass::ClassC).await {
        error!("Error setting lora class");
    } else {
        info!("Lora class set to Class C");
    }

    if let Err(e) = client.adr_set(false).await {
        error!("Error setting lora adr");
    } else {
        info!("Lora adr set to false");
    }

    if let Err(e) = client.dr_set(5).await {
        error!("Error setting lora dr");
    } else {
        info!("Lora dr set to 5");
    }

    if let Err(e) = client.confirm_send_set(false).await {
        error!("Error confirm set");
    } else {
        info!("Lora send ACK set to false");
    }

    if let Err(e) = client.auto_join_set(false, 3).await {
        error!("Error setting auto join");
    } else {
        info!("Auto join disabled");
    }

    let mut joined = false;
    while !joined {
        if let Err(e) = client.lora_join_otaa().await {
            error!("Error joining");
        } else {
            info!("Started joining OTAA");
        }

        loop {
            match LORA_JOIN_STATUS.wait().await {
                JoinStatus::Success => {
                    info!("Joined");
                    joined = true;
                    break;
                }
                JoinStatus::Joining => {}
                JoinStatus::Failure => {
                    info!("Join failed");
                    break;
                }
                JoinStatus::NotJoined => {
                    info!("Join failed");
                    break;
                }
                JoinStatus::Unknown => {
                    error!("Unknown error");
                }
            }
        }
        if !joined {
            error!("Failed to join");
        }
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
        // match client.send(3, 12, b"Hello from Lora-E5").await {
        //     Ok(_d) => {
        //         info!("Sent bytes");
        //     }
        //     Err(e) => error!("Error sending {}", e),
        // }
        for _i in 0..4 {
            // let downlink_frame_count_get = client.downlink_frame_count().await;
            let downlink_frame_count_get = LORA_MESSAGE_RECEIVED_COUNT.try_signaled_value().unwrap_or_default();
            if downlink_frame_count_get != downlink_frame_count {
                info!(
                    "Downlink frame count changed: {:?}",
                    downlink_frame_count_get
                );
                downlink_frame_count = downlink_frame_count_get;
                // let recv = client.receive().await;
                let data = LAST_LORA_MESSAGE_RECEIVED.wait().await;
                let bytes = &data.payload;
                info!("Received {:?} bytes, {:?} PORT", data.length, data.port);

                let l = core::str::from_utf8(&bytes[0..(data.length as usize)]).unwrap();
                info!("Bytes as string: {:?}", l);
            }
            Timer::after(Duration::from_secs(5)).await;
        }
    }
}
