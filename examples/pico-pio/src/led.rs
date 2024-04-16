use atat::nom;
use atat::nom::{bytes, character, sequence};
use defmt::unwrap;
use embassy_executor::Spawner;
use embassy_rp::gpio::Output;
use embassy_rp::peripherals::{PIN_7, PIN_8, PIN_9};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::Receiver;
use embassy_time::{Duration, Timer};

pub static mut LED_CHANNEL: Option<embassy_sync::channel::Channel<NoopRawMutex, LedCommand, 4>> =
    None;

pub struct Led<'d> {
    pub red: bool,
    pub green: bool,
    pub blue: bool,
    pub pin_red: Output<'d, PIN_8>,
    pub pin_green: Output<'d, PIN_9>,
    pub pin_blue: Output<'d, PIN_7>,
}

pub enum LedCommand {
    SetColor(bool, bool, bool),
    Pulse(bool, bool, bool, usize, u64, u64),
}

impl LedCommand {
    pub fn parse(buf: &[u8]) -> Result<Self, ()> {
        if buf.len() < 1 {
            return Err(());
        }
        let val = sequence::tuple((
            bytes::streaming::take_while::<_, _, nom::error::Error<_>>(character::is_alphabetic),
            bytes::streaming::tag::<_, _, nom::error::Error<_>>(b","),
            bytes::streaming::take_while1::<_, _, nom::error::Error<_>>(|_b| true),
        ))(buf);

        if let Err(_) = val {
            return Err(());
        }
        let (_, (cmd, _, value)) = val.unwrap();
        return match cmd {
            b"color" => {
                let val = sequence::tuple((
                    bytes::streaming::take_while1::<_, _, nom::error::Error<_>>(
                        character::is_digit,
                    ),
                    bytes::streaming::tag::<_, _, nom::error::Error<_>>(b","),
                    bytes::streaming::take_while1::<_, _, nom::error::Error<_>>(
                        character::is_digit,
                    ),
                    bytes::streaming::tag::<_, _, nom::error::Error<_>>(b","),
                    bytes::streaming::take_while1::<_, _, nom::error::Error<_>>(
                        character::is_digit,
                    ),
                ))(value);
                if let Err(_) = val {
                    return Err(());
                }
                let (_, (red, _, green, _, blue)) = val.unwrap();
                let red = red[0] == b'1';
                let green = green[0] == b'1';
                let blue = blue[0] == b'1';
                Ok(LedCommand::SetColor(red, green, blue))
            }
            b"pulse" => {
                let val = sequence::tuple((
                    bytes::streaming::take_while1::<_, _, nom::error::Error<_>>(
                        character::is_digit,
                    ),
                    bytes::streaming::tag::<_, _, nom::error::Error<_>>(b","),
                    bytes::streaming::take_while1::<_, _, nom::error::Error<_>>(
                        character::is_digit,
                    ),
                    bytes::streaming::tag::<_, _, nom::error::Error<_>>(b","),
                    bytes::streaming::take_while1::<_, _, nom::error::Error<_>>(
                        character::is_digit,
                    ),
                    bytes::streaming::tag::<_, _, nom::error::Error<_>>(b","),
                    bytes::streaming::take_while1::<_, _, nom::error::Error<_>>(
                        character::is_digit,
                    ),
                    bytes::streaming::tag::<_, _, nom::error::Error<_>>(b","),
                    bytes::streaming::take_while1::<_, _, nom::error::Error<_>>(
                        character::is_digit,
                    ),
                    bytes::streaming::tag::<_, _, nom::error::Error<_>>(b","),
                    bytes::streaming::take_while1::<_, _, nom::error::Error<_>>(
                        character::is_digit,
                    ),
                ))(value);

                if let Err(_) = val {
                    return Err(());
                }
                let (_, (red, _, green, _, blue, _, pulse, _, delay_on_ms, _, delay_off_ms)) =
                    val.unwrap();
                let red = red[0] == b'1';
                let green = green[0] == b'1';
                let blue = blue[0] == b'1';
                let pulse =
                    usize::from_str_radix(core::str::from_utf8(pulse).unwrap(), 10).unwrap();
                let delay_on_ms =
                    u64::from_str_radix(core::str::from_utf8(delay_on_ms).unwrap(), 10).unwrap();
                let delay_off_ms =
                    u64::from_str_radix(core::str::from_utf8(delay_off_ms).unwrap(), 10).unwrap();
                Ok(LedCommand::Pulse(
                    red,
                    green,
                    blue,
                    pulse,
                    delay_on_ms,
                    delay_off_ms,
                ))
            }
            _ => Err(()),
        };
    }

    pub async fn handle(&self) -> Result<(), ()> {
        match self {
            LedCommand::SetColor(red, green, blue) => {
                send_led_command(LedCommand::SetColor(*red, *green, *blue)).await;
                Ok(())
            }
            LedCommand::Pulse(red, green, blue, pulse, delay_on_ms, delay_off_ms) => {
                send_led_command(LedCommand::Pulse(
                    *red,
                    *green,
                    *blue,
                    *pulse,
                    *delay_on_ms,
                    *delay_off_ms,
                ))
                .await;
                Ok(())
            }
        }
    }
}

pub fn init_led(
    mut pin_red: Output<'static, PIN_8>,
    mut pin_green: Output<'static, PIN_9>,
    mut pin_blue: Output<'static, PIN_7>,
    spawner: &Spawner,
) {
    pin_red.set_low();
    pin_green.set_low();
    pin_blue.set_low();

    let led = Led {
        pin_red,
        pin_green,
        pin_blue,
        red: false,
        green: false,
        blue: false,
    };
    unsafe {
        let channel = embassy_sync::channel::Channel::new();
        LED_CHANNEL = Some(channel);
    }

    let receiver = unsafe {
        let channel = LED_CHANNEL.as_mut().unwrap();
        let receiver = channel.receiver();

        receiver
    };
    unwrap!(spawner.spawn(led_command_handler(led, receiver)));
}

pub async fn send_led_command(command: LedCommand) {
    let publisher = {
        let channel = unsafe { LED_CHANNEL.as_mut().unwrap() };
        channel.sender()
    };
    publisher.send(command).await;
}

#[embassy_executor::task]
async fn led_command_handler(
    mut led: Led<'static>,
    receiver: Receiver<'static, NoopRawMutex, LedCommand, 4>,
) {
    loop {
        let val = receiver.receive().await;
        match val {
            LedCommand::Pulse(red, green, blue, pulse, delay_on_ms, delay_off_ms) => {
                led.pulse(red, green, blue, pulse, delay_on_ms, delay_off_ms)
                    .await
            }
            LedCommand::SetColor(red, green, blue) => led.set_color(red, green, blue),
        }
    }
}

impl<'a> Led<'a> {
    pub fn set_color(&mut self, red: bool, green: bool, blue: bool) {
        if red {
            self.pin_red.set_high()
        } else {
            self.pin_red.set_low()
        };
        if green {
            self.pin_green.set_high()
        } else {
            self.pin_green.set_low()
        };
        if blue {
            self.pin_blue.set_high()
        } else {
            self.pin_blue.set_low()
        };
        self.red = red;
        self.green = green;
        self.blue = blue;
    }

    pub async fn pulse(
        &mut self,
        red: bool,
        green: bool,
        blue: bool,
        pulse: usize,
        delay_on_ms: u64,
        delay_off_ms: u64,
    ) {
        let (original_red, original_green, original_blue) = (self.red, self.green, self.blue);
        self.set_color(false, false, false);
        for _ in 0..pulse {
            self.set_color(red, green, blue);
            Timer::after(Duration::from_millis(delay_on_ms)).await;
            self.set_color(false, false, false);
            Timer::after(Duration::from_millis(delay_off_ms)).await;
        }
        self.set_color(original_red, original_green, original_blue);
    }
}
