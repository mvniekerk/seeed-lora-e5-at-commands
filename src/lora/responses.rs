use atat_derive::AtatResp;
use heapless::String;
use serde_at::HexStr;
use crate::lora::types::{LoraJoiningStartingStatus, LoraJoiningStatus, LoraJoinMode};

/// MODE Get/Set Response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct ModeGetSetResponse {
    pub mode: String<24>
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
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct OtaaAppEuiResponse {
    pub app_eui: HexStr<u64>,
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

/// AppKey Set response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct AppKeySetResponse {
    // APPKEY <32 char> = 41 char = 82 bytes
    pub response: String<82>
}

/// Join response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct LoraOtaaJoinResponse {
    pub response: String<26>
}

/// Auto join response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct LoraOtaaAutoJoinResponse {
    pub response: String<26>
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
                let dev_addr = parts.skip(1).next();
                match (net_id, dev_addr) {
                    (Some(net_id), Some(dev_addr)) => {
                        let net_id = net_id.into();
                        let dev_addr = dev_addr.into();
                        LoraJoiningStatus::Starting(LoraJoiningStartingStatus::Done(net_id, dev_addr))
                    },
                    _ => LoraJoiningStatus::Unknown
                }
            }
            _ => LoraJoiningStatus::Unknown
        }
    }
}

/// REPEAT response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct RepeatGetSetResponse {
    pub repeat: u8
}

/// RETRY response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct RetryGetSetResponse {
    pub retry: u8
}

/// Max payload length response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct MaxPayloadLengthGetResponse {
    // LEN
    pub command: String<6>,
    pub max: u8
}

/// Uplink/Downlink counter response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct UplinkDownlinkCounterGetResponse {
    // ULDL 4294967295,
    pub command: String<30>,
    pub downlink: u32
}

impl UplinkDownlinkCounterGetResponse {
    pub fn uplink(&self) -> u32 {
        let s = self.command.as_str().split(' ').skip(1).next().unwrap();
        s.parse().unwrap()
    }

    pub fn downlink(&self) -> u32 {
        self.downlink
    }
}
