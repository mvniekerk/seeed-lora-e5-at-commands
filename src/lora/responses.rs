use crate::lora::types::{LoraJoinMode, LoraJoiningStartingStatus, LoraJoiningStatus};
use atat::{AtatLen, AtatResp, AtatUrc};
use atat_derive::{AtatLen, AtatResp};
use core::str::FromStr;
#[cfg(feature = "debug")]
use defmt::error;
use heapless::{String, Vec};
use serde_at::serde::Deserializer;
use serde_at::{HexStr, serde};

/// MODE Get/Set Response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct ModeGetSetResponse {
    pub mode: String<24>,
}

impl ModeGetSetResponse {
    pub fn mode(self) -> LoraJoinMode {
        self.into()
    }
}

/// ID ABP DevAddr Get/Set Response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct AbpDevAddrResponse {
    pub dev_addr_text: String<14>,
    pub dev_addr: HexStr<u32>,
}

/// ID OTAA DevEui Get/Set Response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct OtaaDevEuiResponse {
    pub dev_eui: HexStr<u64>,
}

/// ID OTAA AppEui Get/Set Response
#[derive(Debug, Clone, PartialEq, AtatLen)]
pub struct OtaaAppEuiResponse {
    pub app_eui: HexStr<u64>,
}

impl AtatResp for OtaaAppEuiResponse {}
impl AtatUrc for OtaaAppEuiResponse {
    type Response = Self;

    fn parse(resp: &[u8]) -> Option<Self::Response> {
        let resp = core::str::from_utf8(resp).ok()?;
        let mut resp = resp.split(' ');
        let _app_eui_text = resp.next();
        let app_eui = resp.next()?;
        let app_eui: HexStr<u64> = serde_at::from_str(app_eui).ok()?;
        Some(Self { app_eui })
    }
}

impl<'de> serde::Deserialize<'de> for OtaaAppEuiResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::<{ OtaaAppEuiResponse::LEN + 20 }>::deserialize(deserializer)?;
        let s = s.as_bytes();
        OtaaAppEuiResponse::parse(s).ok_or(serde::de::Error::custom(
            "Failed to parse OtaaAppEuiResponse",
        ))
    }
}

#[derive(Debug, Clone, PartialEq, AtatLen)]
pub struct AppKeySetResponse {
    pub app_key: HexStr<u128>,
}

impl AtatResp for AppKeySetResponse {}
impl AtatUrc for AppKeySetResponse {
    type Response = AppKeySetResponse;

    fn parse(resp: &[u8]) -> Option<Self::Response> {
        let resp = core::str::from_utf8(resp).ok()?;
        let mut resp = resp.split(',');
        let _app_key_text = resp.next();
        let app_key = resp.next()?;
        let app_key: HexStr<u128> = serde_at::from_str(app_key).ok()?;
        Some(Self { app_key })
    }
}

impl<'de> serde::Deserialize<'de> for AppKeySetResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::<{ AppKeySetResponse::LEN + 20 }>::deserialize(deserializer)?;
        let s = s.as_bytes();
        AppKeySetResponse::parse(s).ok_or(serde::de::Error::custom(
            "Failed to parse AppKeySetResponse",
        ))
    }
}

/// Port get/set response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct PortGetSetResponse {
    pub port: u8,
}

/// ADR get/set response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct AdrGetSetResponse {
    pub on: String<6>,
}

impl AdrGetSetResponse {
    pub fn is_on(&self) -> bool {
        self.on.as_str().eq("ON")
    }
}

/// Data rate get/set response
/// Example return US915 DR0 SF10 BW125K
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct DataRateGetSetResponse {
    pub rate: String<42>,
}

/// LoRaWAN class get/set response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct LoRaWANClassGetSetResponse {
    pub class: String<2>,
}

/// Join response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct LoraOtaaJoinResponse {
    pub response: String<26>,
}

/// Auto join response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct LoraOtaaAutoJoinResponse {
    pub response: String<26>,
}

impl From<String<26>> for LoraJoiningStatus {
    fn from(value: String<26>) -> Self {
        match value.as_str() {
            "Starting" => LoraJoiningStatus::Starting(LoraJoiningStartingStatus::Starting),
            "NORMAL" => LoraJoiningStatus::Starting(LoraJoiningStartingStatus::Normal),
            "Join failed" => LoraJoiningStatus::Failed,
            "LoRaWAN modem is busy" => LoraJoiningStatus::Busy,
            x if x.starts_with("NetId") => {
                let mut parts = x.split(' ').skip(1);
                let net_id = parts.next();
                let dev_addr = parts.nth(1);
                match (net_id, dev_addr) {
                    (Some(net_id), Some(dev_addr)) => {
                        let net_id = net_id.try_into().unwrap();
                        let dev_addr = dev_addr.try_into().unwrap();
                        LoraJoiningStatus::Starting(LoraJoiningStartingStatus::Done(
                            net_id, dev_addr,
                        ))
                    }
                    _ => LoraJoiningStatus::Unknown,
                }
            }
            _ => LoraJoiningStatus::Unknown,
        }
    }
}

/// POWER force response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct TxPowerForceSetResponse {
    pub db_m: u8,
}

/// POWER table
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct TxPowerTable {
    pub table: String<80>,
}

impl TxPowerTable {
    pub fn db_m_list(&self) -> Result<Vec<u8, 12>, atat::Error> {
        let mut ret = Vec::new();
        for i in self.table.as_str().split(' ').map(u8::from_str) {
            ret.push(i.map_err(|_e| {
                #[cfg(feature = "debug")]
                error!("Could not parse u8");
                atat::Error::Parse
            })?)
            .map_err(|e| {
                #[cfg(feature = "debug")]
                {
                    error!("Could not add u8 to return of tx power tables: {}", e);
                }
                atat::Error::Parse
            })?;
        }
        Ok(ret)
    }
}

/// REPEAT response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct RepeatGetSetResponse {
    pub repeat: u8,
}

/// RETRY response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct RetryGetSetResponse {
    pub retry: u8,
}

/// Max payload length response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct MaxPayloadLengthGetResponse {
    // LEN
    pub command: String<6>,
    pub max: u8,
}

/// Uplink/Downlink counter response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct UplinkDownlinkCounterGetResponse {
    pub uplink: u32,
    pub downlink: u32,
}

impl UplinkDownlinkCounterGetResponse {
    pub fn uplink(&self) -> u32 {
        self.uplink
    }

    pub fn downlink(&self) -> u32 {
        self.downlink
    }
}
