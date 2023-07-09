use super::responses::{VerResponse, LowPowerResponse};
use crate::NoResponse;
use atat_derive::AtatCmd;

/// 4.1 AT
/// Used to test if the communication with the device is working
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("AT", NoResponse, cmd_prefix = "", timeout_ms = 5000)]
pub struct VerifyComIsWorking {}

/// 4.2 VER
/// Get the version of the firmware running on the unit
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("VER", VerResponse)]
pub struct FirmwareVersion {}

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
    pub sleep_for_millis: u32
}

/// 4.30 LOWPOWER deep sleep enable
/// Enter deep power saving mode
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("LOWPOWER=AUTOON", NoResponse)]
pub struct LowPowerDeepSleepEnable {
}

/// 4.30 LOWPOWER deep sleep disable
/// Stop deep power saving mode
/// Needs 4x 0xFF over UART to be first sent
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("LOWPOWER=AUTOOFF", NoResponse)]
pub struct LowPowerDeepSleepDisable {
}

