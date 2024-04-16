use super::responses::{LowPowerResponse, OkResponse, VerResponse};
use crate::NoResponse;
use atat::digest::ParseError;
use atat::{AtatCmd, Error, InternalError};
use atat_derive::AtatCmd;
use core::str::FromStr;
#[cfg(feature = "debug")]
use defmt::error;
use heapless::String;

/// 4.1 AT
/// Used to test if the communication with the device is working
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("AT", OkResponse, cmd_prefix = "", timeout_ms = 1000)]
pub struct VerifyComIsWorking {}

/// 4.2 VER
/// Get the version of the firmware running on the unit
#[derive(Clone, Debug)]
pub struct FirmwareVersion {}
impl AtatCmd for FirmwareVersion {
    type Response = VerResponse;

    const MAX_LEN: usize = 8;

    fn write(&self, buf: &mut [u8]) -> usize {
        buf.copy_from_slice(b"AT+VER\r\n");
        8
    }

    fn parse(&self, resp: Result<&[u8], InternalError>) -> Result<Self::Response, Error> {
        if resp.is_err() {
            return Err(Error::Parse);
        }
        let buf = resp.unwrap();
        let parse = Self::parse(buf).map_err(|_| Error::Parse);
        if parse.is_err() {
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

/// 4.21 FDEFAULT
#[derive(Clone, Debug)]
pub struct FactoryReset {}

impl AtatCmd for FactoryReset {
    type Response = OkResponse;

    const MAX_LEN: usize = 20;

    const MAX_TIMEOUT_MS: u32 = 15000;

    fn write(&self, buf: &mut [u8]) -> usize {
        buf.copy_from_slice(b"+AT+FDEFAULT=Seeed\r\n");
        20
    }

    fn parse(&self, _resp: Result<&[u8], InternalError>) -> Result<Self::Response, Error> {
        Ok(OkResponse {
            ok: String::from_str("OK").unwrap(),
        })
    }
}

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
