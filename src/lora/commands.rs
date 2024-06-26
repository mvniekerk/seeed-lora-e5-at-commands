use super::responses::{
    AbpDevAddrResponse, AdrGetSetResponse, AppKeySetResponse, DataRateGetSetResponse,
    LoRaWANClassGetSetResponse, LoraOtaaAutoJoinResponse, LoraOtaaJoinResponse,
    MaxPayloadLengthGetResponse, ModeGetSetResponse, OtaaAppEuiResponse, OtaaDevEuiResponse,
    PortGetSetResponse, RepeatGetSetResponse, RetryGetSetResponse, TxPowerForceSetResponse,
    TxPowerTable, UplinkDownlinkCounterGetResponse,
};
use crate::lora::types::{LoraClass, LoraRegion};
use crate::NoResponse;
use atat::{AtatCmd, AtatLen, Error, InternalError};
use atat_derive::{AtatCmd, AtatLen};
use core::str::FromStr;
use heapless::String;
use serde_at::{HexStr, SerializeOptions};

/// 4.3 ABP DevAddr Get
/// Get the ABP mode DevAddr
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+ID", AbpDevAddrResponse)]
pub struct AbpDevAddrGet {
    pub dev_addr_text: String<14>,
}

/// 4.3 ABP DevAddr Set
/// Set the ABP DevAddr
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+ID", AbpDevAddrResponse)]
pub struct AbpDevAddSet {
    pub dev_addr_text: String<14>,
    pub dev_addr: HexStr<u32>,
}

/// 4.3 OTAA DevEUI Get
/// Get the OTAA DevEUI
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+ID=DevEui", OtaaDevEuiResponse)]
pub struct DevEuiGet {}

/// 4.3 OTAA DevEUI Set
/// Set the OTAA DevEUI
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+ID", OtaaDevEuiResponse)]
pub struct DevEuiSet {
    pub dev_eui_text: String<12>,
    pub dev_eui: HexStr<u64>,
}

impl DevEuiSet {
    pub fn dev_eui(dev_eui: u64) -> Self {
        let dev_eui = HexStr {
            val: dev_eui,
            add_0x_with_encoding: false,
            hex_in_caps: true,
            delimiter_after_nibble_count: 2,
            delimiter: ' ',
            skip_last_0_values: false,
        };
        Self {
            dev_eui_text: String::from_str("DevEui").unwrap(),
            dev_eui,
        }
    }
}

/// 4.3 OTAA AppEUI Get
/// Get the OTAA DevEUI
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+ID=AppEui", OtaaAppEuiResponse)]
pub struct AppEuiGet {}

/// 4.3 OTAA AppEUI Set
/// Set the OTAA AppEUI
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+ID", OtaaAppEuiResponse)]
pub struct AppEuiSet {
    pub app_eui_text: String<12>,
    pub app_eui: HexStr<u64>,
}

impl AppEuiSet {
    pub fn app_eui(app_eui: u64) -> Self {
        let app_eui = HexStr {
            val: app_eui,
            add_0x_with_encoding: false,
            hex_in_caps: true,
            delimiter_after_nibble_count: 2,
            delimiter: ' ',
            skip_last_0_values: false,
        };
        Self {
            app_eui_text: String::from_str("AppEui").unwrap(),
            app_eui,
        }
    }
}

/// 4.5 MSG
/// Send unchecked message
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+MSG", NoResponse)]
pub struct MessageStringUnconfirmed {
    pub message: String<128>,
}

/// 4.5.1 Link check
/// Send an empty string message in order to get the
/// link status.
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+MSG", NoResponse)]
pub struct LinkCheck {}

/// 4.6 CMSG
/// Send a string that needs to be confirmed by the server
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CMSG", NoResponse)]
pub struct MessageStringConfirmed {
    pub message: String<128>,
}

/// 4.7 MSGHEX
/// Send hex format data frame that doesn't need to be confirmed by the server
#[derive(Clone, Debug, AtatLen)]
pub struct MessageHexUnconfirmed {
    pub message: HexStr<[u8; 242]>,
}

impl AtatCmd for MessageHexUnconfirmed {
    type Response = NoResponse;
    const MAX_LEN: usize = MessageHexUnconfirmed::LEN + 20;

    const EXPECTS_RESPONSE_CODE: bool = false;

    fn write(&self, buf: &mut [u8]) -> usize {
        let _ = buf.copy_from_slice(b"AT+MSGHEX=");
        let hex_str = serde_at::to_string::<HexStr<[u8; 242]>, { MessageHexUnconfirmed::LEN }>(
            &self.message,
            "",
            SerializeOptions::default(),
        )
        .expect("Failed to serialize message");
        let len = hex_str.len();
        let buf = &mut buf[10..];
        buf[..len].copy_from_slice(hex_str.as_bytes());
        let end = len + 10;
        let buf = &mut buf[..end];
        buf.copy_from_slice(b"\r\n");
        end + 2
    }

    fn parse(&self, _resp: Result<&[u8], InternalError>) -> Result<Self::Response, Error> {
        Ok(NoResponse {})
    }
}

/// 4.7.1 MSGHEX empty
/// Send server unconfirmed payload with zero length
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+MSGHEX", NoResponse)]
pub struct MessageHexUnconfirmedEmpty {}

/// 4.8 CMSGHEX
/// Send hex format data that needs to be confirmed by the server
#[derive(Clone, Debug, AtatLen)]
pub struct MessageHexConfirmed {
    pub message: HexStr<[u8; 242]>,
}

impl AtatCmd for MessageHexConfirmed {
    type Response = NoResponse;
    const MAX_LEN: usize = MessageHexConfirmed::LEN + 22;

    const EXPECTS_RESPONSE_CODE: bool = false;

    fn write(&self, buf: &mut [u8]) -> usize {
        buf.copy_from_slice(b"AT+CMSGHEX=");
        let buf = &mut buf[11..];
        let hex_str = serde_at::to_string::<HexStr<[u8; 242]>, { MessageHexConfirmed::LEN }>(
            &self.message,
            "",
            SerializeOptions::default(),
        )
        .expect("Failed to serialize message");
        let len = hex_str.len();
        let buf = &mut buf[11..];
        buf[..len].copy_from_slice(hex_str.as_bytes());
        let end = len + 11;
        let buf = &mut buf[..end];
        buf.copy_from_slice(b"\r\n");
        end + 2
    }

    fn parse(&self, _resp: Result<&[u8], InternalError>) -> Result<Self::Response, Error> {
        Ok(NoResponse {})
    }
}

/// 4.8.1 CMSGHEX empty
/// Send server confirmed payload with zero length
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CMSGHEX", NoResponse)]
pub struct MessageHexConfirmedEmpty {}

/// 4.9 PMSG
/// Send string format propriety LoRaWAN frames
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+PMSG", NoResponse)]
pub struct ProprietyMessageString {
    pub message: String<128>,
}

/// 4.10 PMSGHEX
/// Send hex format data propriety LoRaWAN frames
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+PMSGHEX", NoResponse)]
pub struct ProprietyMessageHex {
    pub message: HexStr<[u8; 256]>,
}

/// 4.11 PORT Get
/// Get PORT number that is to be used by MSG/CMSG/MSGHEX/CMSGHEX
/// Range from 1 to 255
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+PORT=?", PortGetSetResponse)]
pub struct LoraPortGet {}

/// 4.11 PORT Set
/// Set PORT number that is to be used by MSG/CMSG/MSGHEX/CMSGHEX
/// Range from 1 to 255
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+PORT", PortGetSetResponse)]
pub struct LoraPortSet {
    pub port: u8,
}

/// 4.12 ADR Get
/// Get ADR function of LoRaWAN module
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+ADR=?", AdrGetSetResponse)]
pub struct LoraAdrGet {}

/// 4.12 ADR Set
/// Set ADR function of LoRaWAN module. Either ON or OFF
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+ADR", AdrGetSetResponse)]
pub struct LoraAdrSet {
    pub on: String<6>,
}

impl LoraAdrSet {
    pub fn on() -> Self {
        Self {
            on: "ON".try_into().unwrap(),
        }
    }

    pub fn off() -> Self {
        Self {
            on: "OFF".try_into().unwrap(),
        }
    }
}

/// 4.13.1 DR get
/// Get the data rate
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+DR", DataRateGetSetResponse)]
pub struct LoraDrGet {}

/// 4.13.1 DR set
/// Set the data rate
/// dr0 .. dr15
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+DR", DataRateGetSetResponse)]
pub struct LoraDrSet {
    pub data_rate: String<8>,
}

impl LoraDrSet {
    pub fn new(dr: u8) -> Self {
        let dr = match dr {
            0 => "DR0",
            1 => "DR1",
            2 => "DR2",
            3 => "DR3",
            4 => "DR4",
            5 => "DR5",
            6 => "DR6",
            7 => "DR7",
            8 => "DR8",
            9 => "DR9",
            10 => "DR10",
            11 => "DR11",
            12 => "DR12",
            13 => "DR13",
            14 => "DR14",
            15 => "DR15",
            _ => panic!("Invalid data rate"),
        };
        Self {
            data_rate: dr.try_into().unwrap(),
        }
    }
}

/// 4.13.2 DR scheme get
/// Get the data rate scheme
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+DR=SCHEME", DataRateGetSetResponse)]
pub struct DataRateSchemeGet {}

/// 4.13.2 DR scheme set
/// Set the data rate scheme
/// One of EU868 US915 US915HYBRID CN779 EU433 AU915 AU915OLD CN470 AS923 KR920 IN865 RU864 CN470PREQUEL STE920 JP920
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+DR", DataRateGetSetResponse)]
pub struct DataRateSchemeSet {
    pub scheme: String<24>,
}

impl DataRateSchemeSet {
    pub fn region(region: LoraRegion) -> Self {
        Self {
            scheme: region.into(),
        }
    }
}

/// 4.15.2 POWER force set
/// Force set the dBm TX power
#[derive(Clone, Debug)]
pub struct TxPowerForceSet {
    pub db_m: u8,
}

impl TxPowerForceSet {
    pub fn new(db_m: u8) -> Self {
        Self { db_m }
    }
}

impl AtatCmd for TxPowerForceSet {
    type Response = TxPowerForceSetResponse;

    const MAX_LEN: usize = 22;

    const MAX_TIMEOUT_MS: u32 = 30000;

    fn write(&self, mut buf: &mut [u8]) -> usize {
        use embedded_io::Write;
        let _ = write!(buf, "AT+POWER={}, FORCE\r\n", self.db_m);
        let mut count = 0;
        for byte in buf.iter() {
            if *byte == b'\r' {
                break;
            }
            count += 1;
        }
        count + 2
    }

    fn parse(&self, _resp: Result<&[u8], InternalError>) -> Result<Self::Response, Error> {
        Ok(TxPowerForceSetResponse { db_m: 0 })
    }
}

/// 4.15.3 POWER table get
/// Get the TX power table in dbM
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+POWER", TxPowerTable)]
pub struct TxPowerTableGet {
    pub table_txt: String<10>,
}

impl Default for TxPowerTableGet {
    fn default() -> Self {
        Self {
            table_txt: "TABLE".try_into().unwrap(),
        }
    }
}

/// 4.16 REPT Get
/// Get the number of repeats for unconfirmed messages
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+REPT=?", RepeatGetSetResponse)]
pub struct RepeatGet {}

/// 4.16 REPT Set
/// Set the number of repeats for unconfirmed messages
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+REPT", RepeatGetSetResponse)]
pub struct RepeatSet {
    pub repeat: u8,
}

/// 4.17 RETRY Get
/// Get the number of retries for confirmed messages
/// Range from 0 to 255
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+RETRY=?", RetryGetSetResponse)]
pub struct RetryGet {}

/// 4.17 RETRY Set
/// Set the number of retries for confirmed messages
/// Range from 0 to 255
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+RETRY", RetryGetSetResponse)]
pub struct RetrySet {
    pub retry: u8,
}

/// 4.20 KEY App key set
/// Set the AppKey for OTAA
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+KEY", AppKeySetResponse)]
pub struct AppKeySet {
    pub app_key_text: String<82>,
    pub key: HexStr<[u8; 16]>,
}

impl AppKeySet {
    pub fn app_key(app_key: u128) -> Self {
        let key = HexStr::<_> {
            val: app_key.to_le_bytes(),
            ..Default::default()
        };
        Self {
            app_key_text: "APPKEY".try_into().unwrap(),
            key,
        }
    }
}

/// 4.23 MODE Get
/// Get the mode (Test, OTAA or ABP)
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+MODE", ModeGetSetResponse)]
pub struct ModeGet {}

/// 4.23 MODE Set
/// Set the mode (Test, OTAA or ABP)
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd(
    "+MODE",
    ModeGetSetResponse,
    timeout_ms = 10000,
    quote_escape_strings = false
)]
pub struct ModeSet {
    pub mode: String<12>,
}

impl ModeSet {
    pub fn otaa() -> Self {
        Self {
            mode: "LWOTAA".try_into().unwrap(),
        }
    }

    pub fn abp() -> Self {
        Self {
            mode: "LWABP".try_into().unwrap(),
        }
    }

    pub fn test() -> Self {
        Self {
            mode: "TEST".try_into().unwrap(),
        }
    }
}

/// 4.24 OTAA Join
/// Join a network using OTAA
#[derive(Clone, Debug)]
pub struct LoraJoinOtaa {}
impl AtatCmd for LoraJoinOtaa {
    type Response = LoraOtaaJoinResponse;

    const MAX_LEN: usize = 9;

    const MAX_TIMEOUT_MS: u32 = 10000;

    fn write(&self, buf: &mut [u8]) -> usize {
        buf.copy_from_slice(b"AT+JOIN\r\n");
        9
    }

    fn parse(&self, resp: Result<&[u8], InternalError>) -> Result<Self::Response, Error> {
        match resp {
            Ok(resp) => {
                let response = core::str::from_utf8(resp)
                    .map_err(|_| Error::Parse)
                    .map(|s| s.try_into().map_err(|_| Error::Parse))
                    .flatten()?;
                Ok(Self::Response { response })
            }
            Err(_err) => Err(Error::Parse),
        }
    }
}

/// 4.24 OTAA Join force
/// Force join a network using OTAA
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+JOIN=FORCE", LoraOtaaJoinResponse)]
pub struct LoraJoinOtaaForce {}

/// 4.24.1 OTAA Join at data rate
/// Join a network using OTAA at a data rate DR0 .. DR15
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+JOIN=FORCE", LoraOtaaJoinResponse)]
pub struct LoraJoinOtaaAtDataRate {
    pub data_rate: String<8>,
}

/// 4.24.2 OTAA disable auto join
/// Disable auto joining
#[derive(Clone, Debug)]
pub struct LoraAutoJoinOtaaDisable {}

impl AtatCmd for LoraAutoJoinOtaaDisable {
    type Response = LoraOtaaAutoJoinResponse;

    const MAX_LEN: usize = 11;

    fn write(&self, buf: &mut [u8]) -> usize {
        buf.copy_from_slice(b"AT+JOIN=0\r\n");
        11
    }

    fn parse(&self, resp: Result<&[u8], InternalError>) -> Result<Self::Response, Error> {
        let buf = resp.map_err(|_| Error::Parse)?;
        let resp = core::str::from_utf8(buf)
            .map_err(|_| Error::Parse)
            .map(|b| b.try_into().map_err(|_| Error::Parse))
            .flatten()?;
        Ok(Self::Response { response: resp })
    }
}

/// 4.24.2 OTAA auto join 0
/// Setup auto join using its interval as per auto join mode 0
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+JOIN=AUTO, ", LoraOtaaAutoJoinResponse, cmd_prefix = "")]
pub struct LoraAutoJoinOtaaMode0 {
    pub interval: u32,
}

/// 4.24.2 OTAA auto join
/// Setup auto join using the setup provided
/// If min_period is 0, then auto join mode is OFF
/// If max_period is 0, then it is in auto join mode 0
/// If steps is 0, then it is in auto join mode 1
/// Otherwise, it is in auto join mode 2
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+JOIN", LoraOtaaAutoJoinResponse)]
pub struct LoraAutoJoinOtaaMode {
    pub cmd: String<8>,
    pub min_period: u32,
    pub max_period: u32,
    pub steps: u32,
}

impl LoraAutoJoinOtaaMode {
    pub fn mode0(min_period: u32) -> Self {
        Self {
            cmd: "AUTO".try_into().unwrap(),
            min_period,
            max_period: 0,
            steps: 0,
        }
    }

    pub fn mode1(min_period: u32, max_period: u32) -> Self {
        Self {
            cmd: "AUTO".try_into().unwrap(),
            min_period,
            max_period,
            steps: 0,
        }
    }

    pub fn mode2(min_period: u32, max_period: u32, steps: u32) -> Self {
        Self {
            cmd: "AUTO".try_into().unwrap(),
            min_period,
            max_period,
            steps,
        }
    }
}

/// 4.26 CLASS Get
/// Get LoRaWAN class (A, B or C)
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CLASS=?", LoRaWANClassGetSetResponse)]
pub struct LoraClassGet {}

/// 4.26 CLASS Set
/// Set LoRaWAN class (A, B or C)
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CLASS", LoRaWANClassGetSetResponse)]
pub struct LoraClassSet {
    pub class: String<2>,
}

impl LoraClassSet {
    pub fn class(class: LoraClass) -> Self {
        Self {
            class: class.into(),
        }
    }
}

/// 4.26 CLASS Set and save
/// Set LoRaWAN class (A, B or C) and save config
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CLASS", LoRaWANClassGetSetResponse)]
pub struct LoraClassSetAndSave {
    pub class: String<2>,
    pub save: String<8>,
}

/// 4.28.2 LW ULDL upload download counter get
/// Get the upload and download counter
#[derive(Clone, Debug)]
// #[at_cmd("+LW", UplinkDownlinkCounterGetResponse)]
pub struct LoraUplinkDownlinkCounterGet {
    // ULDL
    // pub command: String<4>,
}

impl AtatCmd for LoraUplinkDownlinkCounterGet {
    type Response = UplinkDownlinkCounterGetResponse;

    const MAX_LEN: usize = 12;

    fn write(&self, buf: &mut [u8]) -> usize {
        buf.copy_from_slice(b"AT+LW=ULDL\r\n");
        12
    }

    fn parse(&self, resp: Result<&[u8], InternalError>) -> Result<Self::Response, Error> {
        match resp {
            Ok(resp) => {
                let resp = core::str::from_utf8(resp).map_err(|_| Error::Parse)?;
                let mut resp = resp.split(',');
                let uplink = resp.nth(1).ok_or(Error::Parse)?;
                let downlink = resp.next().ok_or(Error::Parse)?;
                Ok(Self::Response {
                    uplink: uplink.parse().map_err(|_| Error::Parse)?,
                    downlink: downlink.parse().map_err(|_| Error::Parse)?,
                })
            }
            Err(_err) => Err(Error::Parse),
        }
    }
}

/// 4.28.12 LW Max payload length get
/// Get the max length of the payload at the current data rate
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+LW", MaxPayloadLengthGetResponse)]
pub struct LoraMaxTxLengthGet {
    // LEN
    pub command: String<6>,
}

impl Default for LoraMaxTxLengthGet {
    fn default() -> Self {
        Self {
            command: "LEN".try_into().unwrap(),
        }
    }
}
