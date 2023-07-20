use super::responses::{LowPowerResponse, OkResponse, VerResponse};
use crate::NoResponse;
use atat::digest::ParseError;
use atat::helpers::LossyStr;
use atat::{AtatCmd, Error, InternalError};
use atat_derive::AtatCmd;
use defmt::error;
use heapless::{String, Vec};

use atat::nom::{branch, bytes, character, sequence};

/// 4.1 AT
/// Used to test if the communication with the device is working
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("AT", OkResponse, cmd_prefix = "", timeout_ms = 1000)]
pub struct VerifyComIsWorking {}

/// 4.2 VER
/// Get the version of the firmware running on the unit
#[derive(Clone, Debug)]
pub struct FirmwareVersion {}
impl AtatCmd<16> for FirmwareVersion {
    type Response = VerResponse;

    fn parse(&self, resp: Result<&[u8], InternalError>) -> Result<Self::Response, Error> {
        if resp.is_err() {
            return Err(Error::Parse);
        }
        let buf = resp.unwrap();
        let parse = Self::parse(buf).map_err(|_| Error::Parse);
        if let Err(e) = parse {
            return Err(Error::Parse);
        }
        let (major, minor, patch) = parse.unwrap();

        match (
            major.parse::<u8>(),
            minor.parse::<u8>(),
            patch.parse::<u8>(),
        ) {
            (Ok(major), Ok(minor), Ok(patch)) => Ok(VerResponse {
                major,
                minor,
                patch,
            }),
            _ => {
                #[cfg(feature = "debug")]
                error!("Failed to parse u8 values for software version");
                Err(Error::Parse)
            }
        }
    }

    fn as_bytes(&self) -> Vec<u8, 16> {
        use core::fmt::Write;
        let mut buf = Vec::new();
        write!(buf, "AT+VER\r\n").unwrap();
        buf
    }
}

impl FirmwareVersion {
    fn parse(buf: &[u8]) -> Result<(&str, &str, &str), ParseError> {
        let s = core::str::from_utf8(buf).map_err(|_| ParseError::NoMatch)?;
        let mut s = s.split('.');

        let major = s.next().ok_or(ParseError::NoMatch)?;
        let minor = s.next().ok_or(ParseError::NoMatch)?;
        let patch = s.next().ok_or(ParseError::NoMatch)?;
        Ok((major, minor, patch))
    }
}

/// 4.4 RESET
/// Reset the module
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+RESET", OkResponse, timeout_ms = 5000)]
pub struct Reset {}

/// 4.30 LOWPOWER until woken up
/// Sleep until woken by RX
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("LOWPOWER", LowPowerResponse)]
pub struct LowPowerUntilWokenUp {}

/// 4.30 LOWPOWER for x milliseconds
/// Sleep for x milliseconds
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("LOWPOWER", LowPowerResponse)]
pub struct LowPowerForMilliseconds {
    pub sleep_for_millis: u32,
}

/// 4.30 LOWPOWER deep sleep enable
/// Enter deep power saving mode
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("LOWPOWER=AUTOON", NoResponse)]
pub struct LowPowerDeepSleepEnable {}

/// 4.30 LOWPOWER deep sleep disable
/// Stop deep power saving mode
/// Needs 4x 0xFF over UART to be first sent
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("LOWPOWER=AUTOOFF", NoResponse)]
pub struct LowPowerDeepSleepDisable {}
