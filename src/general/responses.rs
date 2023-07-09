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
    pub ver: String<256>,
}

/// LOWPOWER response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct LowPowerResponse {
    pub message: String<12>,
}
