use atat_derive::AtatResp;
use heapless::String;

/// OK response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct OkResponse {
    pub ok: String<4>,
}

impl OkResponse {
    pub fn is_ok(&self) -> bool {
        self.ok.as_str().eq("OK")
    }
}

/// VER response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct VerResponse {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

/// LOWPOWER response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct LowPowerResponse {
    pub message: String<12>,
}
