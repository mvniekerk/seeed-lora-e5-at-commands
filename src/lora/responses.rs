use atat_derive::AtatResp;
use heapless::String;
use serde_at::HexStr;

/// ID ABP DevAddr Get/Set Response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct AbpDevAddrResponse {
    pub dev_addr_text: String<14>,
    pub dev_addr: HexStr<u32>,
}

/// ID OTAA DevEui Get/Set Response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct OtaaDevEuiResponse {
    pub dev_eui_text: String<12>,
    pub dev_eui: HexStr<u64>,
}

/// ID OTAA AppEui Get/Set Response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct OtaaAppEuiResponse {
    pub app_eui_text: String<12>,
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
    pub on: String<6>
}

/// Data rate get/set response
/// Example return US915 DR0 SF10 BW125K
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct DataRateGetSetResponse {
    pub rate: String<42>
}

/// LoRaWAN class get/set response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct LoRaWANClassGetSetResponse {
    pub class: String<2>
}