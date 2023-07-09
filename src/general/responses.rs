use atat_derive::AtatResp;
use heapless::String;
use serde_at::HexStr;

/// OK response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct OkResponse {
    pub ok: String<4>,
}

/// VER response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct VerResponse {
    pub ver: String<256>,
}

/// LOWPOWER response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct LowPowerResponse {
    pub message: String<12>
}