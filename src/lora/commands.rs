use super::responses::{
    AbpDevAddrResponse, AdrGetSetResponse, DataRateGetSetResponse, LoRaWANClassGetSetResponse,
    OtaaAppEuiResponse, OtaaDevEuiResponse, PortGetSetResponse,
};
use atat_derive::AtatCmd;
use heapless::String;
use serde_at::HexStr;

/// 4.3 ABP DevAddr Get
/// Get the ABP mode DevAddr
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("ID", AbpDevAddrResponse)]
pub struct AbpDevAddrGet {
    pub dev_addr_text: String<14>,
}

/// 4.3 ABP DevAddr Set
/// Set the ABP DevAddr
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("ID", AbpDevAddrResponse)]
pub struct AbpDevAddSet {
    pub dev_addr_text: String<14>,
    pub dev_addr: HexStr<u32>,
}

/// 4.3 OTAA DevEUI Get
/// Get the OTAA DevEUI
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("ID=DevEui", OtaaDevEuiResponse)]
pub struct OtaaDevEuiGet {
    pub dev_eui_text: String<12>,
}

/// 4.3 OTAA DevEUI Set
/// Set the OTAA DevEUI
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("ID", OtaaDevEuiResponse)]
pub struct OtaaDevEuiSet {
    pub dev_eui_text: String<12>,
    pub dev_eui: HexStr<u64>,
}

/// 4.3 OTAA AppEUI Get
/// Get the OTAA DevEUI
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("ID", OtaaAppEuiResponse)]
pub struct OtaaAppEuiGet {
    pub app_eui_text: String<12>,
}

/// 4.3 OTAA AppEUI Set
/// Set the OTAA DevEUI
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("ID", OtaaAppEuiResponse)]
pub struct OtaaAppEuiSet {
    pub app_eui_text: String<12>,
    pub app_eui: HexStr<u64>,
}

/// 4.4 RESET
/// Reset the module
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("RESET", NoResponse)]
pub struct Reset {}

/// 4.5 MSG
/// Send unchecked message
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("MSG", NoResponse)]
pub struct MessageStringUnconfirmed {
    pub message: String<128>,
}

/// 4.5.1 Link check
/// Send an empty string message in order to get the
/// link status.
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("MSG", NoResponse)]
pub struct LinkCheck {}

/// 4.6 CMSG
/// Send a string that needs to be confirmed by the server
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("CMSG", NoResponse)]
pub struct MessageStringConfirmed {
    pub message: String<128>,
}

/// 4.7 MSGHEX
/// Send hex format data frame that doesn't need to be confirmed by the server
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("MSGHEX", NoResponse)]
pub struct MessageHexUnconfirmed {
    pub message: HexStr<[u8; 256]>,
}

/// 4.7.1 MSGHEX empty
/// Send server unconfirmed payload with zero length
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("MSGHEX", NoResponse)]
pub struct MessageHexUnconfirmedEmpty {}

/// 4.8 CMSGHEX
/// Send hex format data that needs to be confirmed by the server
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("CMSGHEX", NoResponse)]
pub struct MessageHexConfirmed {
    pub message: HexStr<[u8; 256]>,
}

/// 4.8.1 CMSGHEX empty
/// Send server confirmed payload with zero length
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("CMSHEX", NoResponse)]
pub struct MessageHexConfirmedEmpty {}

/// 4.9 PMSG
/// Send string format propriety LoRaWAN frames
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("PMSG", NoResponse)]
pub struct ProprietyMessageString {
    pub message: String<128>,
}

/// 4.10 PMSGHEX
/// Send hex format data propriety LoRaWAN frames
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("PMSGHEX", NoResponse)]
pub struct ProprietyMessageHex {
    pub message: HexStr<[u8; 256]>,
}

/// 4.11 PORT Get
/// Get PORT number that is to be used by MSG/CMSG/MSGHEX/CMSGHEX
/// Range from 1 to 255
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("PORT=?", PortGetSetResponse)]
pub struct PortGet {}

/// 4.11 PORT Set
/// Set PORT number that is to be used by MSG/CMSG/MSGHEX/CMSGHEX
/// Range from 1 to 255
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("PORT", PortGetSetResponse)]
pub struct PortSet {
    pub port: u8,
}

/// 4.12 ADR Get
/// Get ADR function of LoRaWAN module
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("ADR=?", AdrGetSetResponse)]
pub struct AdrGet {}

/// 4.12 ADR Set
/// Set ADR function of LoRaWAN module. Either ON or OFF
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("ADR", AdrGetSetResponse)]
pub struct AdrSet {
    pub on: String<6>,
}

/// 4.13.1 DR get
/// Get the data rate
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("DR", NoResponse)]
pub struct DataRateGet {}

/// 4.13.1 DR set
/// Set the data rate
/// dr0 .. dr15
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("DR", NoResponse)]
pub struct DataRateSet {
    pub on: String<8>,
}

/// 4.13.2 DR scheme get
/// Get the data rate scheme
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("DR=SCHEME", NoResponse)]
pub struct DataRateSchemeGet {}

/// 4.13.2 DR scheme set
/// Set the data rate scheme
/// One of EU868 US915 US915HYBRID CN779 EU433 AU915 AU915OLD CN470 AS923 KR920 IN865 RU864 CN470PREQUEL STE920 JP920
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("DR", DataRateGetSetResponse)]
pub struct DataRateSchemeSet {
    pub scheme: String<24>,
}

/// 4.24 OTAA Join
/// Join a network using OTAA
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("JOIN", NoResponse)]
pub struct OtaaJoin {}

/// 4.24 OTAA Join force
/// Force join a network using OTAA
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("JOIN=FORCE", NoResponse)]
pub struct OtaaJoinForce {}

/// 4.24.1 OTAA Join at data rate
/// Join a network using OTAA at a data rate DR0 .. DR15
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("JOIN=FORCE", NoResponse)]
pub struct OtaaJoinAtDataRate {
    pub data_rate: String<8>,
}

/// 4.24.2 OTAA disable auto join
/// Disable auto joining
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("JOIN=0", NoResponse)]
pub struct OtaaAutoJoinDisable {}

/// 4.24.2 OTAA auto join 0
/// Setup auto join using its interval as per auto join mode 0
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("JOIN=AUTO, ", NoResponse, cmd_prefix = "")]
pub struct OtaaAutoJoinMode0 {
    pub interval: u32,
}

/// 4.24.2 OTAA auto join
/// Setup auto join using the setup provided
/// If min_period is 0, then auto join mode is OFF
/// If max_period is 0, then it is in auto join mode 0
/// If steps is 0, then it is in auto join mode 1
/// Otherwise, it is in auto join mode 2
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+JOIN=AUTO, ", NoResponse, cmd_prefix = "")]
pub struct OtaaAutoJoinMode {
    pub min_period: u32,
    pub max_period: u32,
    pub steps: u32,
}

/// 4.26 CLASS Get
/// Get LoRaWAN class (A, B or C)
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CLASS=?", LoRaWANClassGetSetResponse)]
pub struct LoRaWANClassGet {}

/// 4.26 CLASS Set
/// Set LoRaWAN class (A, B or C)
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CLASS", LoRaWANClassGetSetResponse)]
pub struct LoRaWANClassSet {
    pub class: String<2>,
}

/// 4.26 CLASS Set and save
/// Set LoRaWAN class (A, B or C) and save config
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CLASS", LoRaWANClassGetSetResponse)]
pub struct LoRaWANClassSetAndSave {
    pub class: String<2>,
    pub save: String<8>,
}
