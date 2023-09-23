use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{Pio, PioPin};
use embassy_rp::Peripheral;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;

use crate::pio_uart::uart_rx::{PioUartRx, PioUartRxReader};
use crate::pio_uart::uart_tx::PioUartTx;
use crate::Irqs;

pub struct PioUart<'a> {
    tx: PioUartTx<'a>,
    rx: PioUartRx<'a>,
    reader: PioUartRxReader<'a>,
}

impl<'a> PioUart<'a> {
    pub fn new(
        baud: u64,
        pio: impl Peripheral<P = PIO0> + 'a,
        tx_pin: impl PioPin,
        rx_pin: impl PioPin,
        pipe: &'a mut embassy_sync::pipe::Pipe<ThreadModeRawMutex, 20>,
    ) -> PioUart<'a> {
        let Pio {
            mut common,
            sm0,
            sm1,
            ..
        } = Pio::new(pio, Irqs);
        let reader = pipe.reader();
        let writer = pipe.writer();

        let tx = PioUartTx::new(&mut common, sm0, tx_pin, baud);
        let (rx, reader) = PioUartRx::new(&mut common, sm1, rx_pin, baud, reader, writer);

        PioUart { tx, rx, reader }
    }

    pub fn split(self) -> (PioUartRx<'a>, PioUartTx<'a>, PioUartRxReader<'a>) {
        (self.rx, self.tx, self.reader)
    }
}

pub mod uart_tx {
    use core::convert::Infallible;

    use embassy_rp::gpio::Level;
    use embassy_rp::peripherals::PIO0;
    use embassy_rp::pio::{
        Common, Config, Direction, FifoJoin, PioPin, ShiftDirection, StateMachine,
    };
    use embedded_io::ErrorType;
    use embedded_io_async::Write;
    use fixed::traits::ToFixed;
    use fixed_macro::types::U56F8;

    pub struct PioUartTx<'a> {
        sm_tx: StateMachine<'a, PIO0, 0>,
    }

    impl<'a> PioUartTx<'a> {
        pub fn new(
            common: &mut Common<'a, PIO0>,
            mut sm_tx: StateMachine<'a, PIO0, 0>,
            tx_pin: impl PioPin,
            baud: u64,
        ) -> Self {
            let prg = pio_proc::pio_asm!(
                r#"
                .side_set 1 opt

                ; An 8n1 UART transmit program.
                ; OUT pin 0 and side-set pin 0 are both mapped to UART TX pin.

                    pull       side 1 [7]  ; Assert stop bit, or stall with line in idle state
                    set x, 7   side 0 [7]  ; Preload bit counter, assert start bit for 8 clocks
                bitloop:                   ; This loop will run 8 times (8n1 UART)
                    out pins, 1            ; Shift 1 bit from OSR to the first OUT pin
                    jmp x-- bitloop   [6]  ; Each loop iteration is 8 cycles.
            "#
            );
            let tx_pin = common.make_pio_pin(tx_pin);
            sm_tx.set_pins(Level::High, &[&tx_pin]);
            sm_tx.set_pin_dirs(Direction::Out, &[&tx_pin]);

            let mut cfg = Config::default();

            cfg.set_out_pins(&[&tx_pin]);
            cfg.use_program(&common.load_program(&prg.program), &[&tx_pin]);
            cfg.shift_out.auto_fill = false;
            cfg.shift_out.direction = ShiftDirection::Right;
            cfg.shift_out.threshold = 32;
            cfg.fifo_join = FifoJoin::TxOnly;
            cfg.clock_divider = (U56F8!(125_000_000) / (8 * baud)).to_fixed();
            sm_tx.set_config(&cfg);
            sm_tx.set_enable(true);

            Self { sm_tx }
        }

        pub async fn write_u8(&mut self, data: u8) {
            self.sm_tx.tx().wait_push(data as u32).await;
        }
    }

    impl ErrorType for PioUartTx<'_> {
        type Error = Infallible;
    }

    impl Write for PioUartTx<'_> {
        async fn write(&mut self, buf: &[u8]) -> Result<usize, Infallible> {
            for byte in buf {
                self.write_u8(*byte).await;
            }
            Ok(buf.len())
        }
    }
}

pub mod uart_rx {
    use core::convert::Infallible;

    use embassy_rp::gpio::Level;
    use embassy_rp::peripherals::PIO0;
    use embassy_rp::pio::{
        Common, Config, Direction, FifoJoin, PioPin, ShiftDirection, StateMachine,
    };
    use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
    use embedded_io::ErrorType;
    use embedded_io_async::Read;
    use fixed::traits::ToFixed;
    use fixed_macro::types::U56F8;

    pub struct PioUartRx<'a> {
        reader: embassy_sync::pipe::Reader<'a, ThreadModeRawMutex, 20>,
    }

    pub struct PioUartRxReader<'a> {
        sm_rx: StateMachine<'a, PIO0, 1>,
        writer: embassy_sync::pipe::Writer<'a, ThreadModeRawMutex, 20>,
    }

    impl<'a> PioUartRx<'a> {
        pub fn new(
            common: &mut Common<'a, PIO0>,
            mut sm_rx: StateMachine<'a, PIO0, 1>,
            rx_pin: impl PioPin,
            baud: u64,
            reader: embassy_sync::pipe::Reader<'a, ThreadModeRawMutex, 20>,
            writer: embassy_sync::pipe::Writer<'a, ThreadModeRawMutex, 20>,
        ) -> (Self, PioUartRxReader<'a>) {
            let prg = pio_proc::pio_asm!(
                r#"
                ; Slightly more fleshed-out 8n1 UART receiver which handles framing errors and
                ; break conditions more gracefully.
                ; IN pin 0 and JMP pin are both mapped to the GPIO used as UART RX.

                start:
                    wait 0 pin 0        ; Stall until start bit is asserted
                    set x, 7    [10]    ; Preload bit counter, then delay until halfway through
                rx_bitloop:             ; the first data bit (12 cycles incl wait, set).
                    in pins, 1          ; Shift data bit into ISR
                    jmp x-- rx_bitloop [6] ; Loop 8 times, each loop iteration is 8 cycles
                    jmp pin good_rx_stop   ; Check stop bit (should be high)

                    irq 4 rel           ; Either a framing error or a break. Set a sticky flag,
                    wait 1 pin 0        ; and wait for line to return to idle state.
                    jmp start           ; Don't push data if we didn't see good framing.

                good_rx_stop:           ; No delay before returning to start; a little slack is
                    in null 24
                    push                ; important in case the TX clock is slightly too fast.
            "#
            );
            let mut cfg = Config::default();
            cfg.use_program(&common.load_program(&prg.program), &[]);

            let rx_pin = common.make_pio_pin(rx_pin);
            sm_rx.set_pins(Level::High, &[&rx_pin]);
            cfg.set_in_pins(&[&rx_pin]);
            cfg.set_jmp_pin(&rx_pin);
            sm_rx.set_pin_dirs(Direction::In, &[&rx_pin]);

            cfg.clock_divider = (U56F8!(125_000_000) / (8 * baud)).to_fixed();
            cfg.shift_in.auto_fill = false;
            cfg.shift_in.direction = ShiftDirection::Right;
            cfg.shift_in.threshold = 32;
            cfg.fifo_join = FifoJoin::RxOnly;
            sm_rx.set_config(&cfg);
            sm_rx.set_enable(true);

            (Self { reader }, PioUartRxReader { sm_rx, writer })
        }
    }

    impl ErrorType for PioUartRx<'_> {
        type Error = Infallible;
    }

    impl Read for PioUartRx<'_> {
        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Infallible> {
            let i = self.reader.read(buf).await;
            Ok(i)
        }
    }

    #[embassy_executor::task]
    pub async fn read_from_pio_uart_task(mut pio_uart: PioUartRxReader<'static>) {
        loop {
            let byte = pio_uart.sm_rx.rx().wait_pull().await as u8;
            pio_uart.writer.write(&[byte]).await;
        }
    }
}
