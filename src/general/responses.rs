use atat::Error;
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

impl VerResponse {
    pub fn major_minor_patch(&self) -> Result<(u16, u16, u16), Error> {
        let mut parts = self.ver.split('.');
        let major = parts.next().ok_or(Error::Parse)?.parse::<u16>().map_err(|_| Error::Parse)?;
        let minor = parts.next().ok_or(Error::Parse)?.parse::<u16>().map_err(|_| Error::Parse)?;
        let patch = parts.next().ok_or(Error::Parse)?.parse::<u16>().map_err(|_| Error::Parse)?;
        Ok((major, minor, patch))
    }
}

/// LOWPOWER response
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct LowPowerResponse {
    pub message: String<12>,
}
