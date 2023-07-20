use crate::urc::URCMessages;
use atat::digest::ParseError;
use atat::helpers::LossyStr;
use atat::nom::{bytes, sequence};
use defmt::info;
use heapless::String;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AutoJoin {
    Off,
    Mode0(u32),
    Mode1(u32),
    Mode2(u32),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum JoinUrc {
    AutoJoin(AutoJoin),
    Start,
    Normal,
    Failed,
    JoinedAlready,
    NetworkJoined,
    Success(String<12>, String<22>),
    Done,
}

impl From<JoinUrc> for URCMessages {
    fn from(value: JoinUrc) -> Self {
        Self::Join(value)
    }
}

impl JoinUrc {
    pub(crate) fn parse(buf: &[u8]) -> Result<Self, ParseError> {
        let (val, _) = sequence::tuple((bytes::streaming::tag("+JOIN: "),))(buf)?;
        let v = LossyStr(val);
        info!("+JOIN PARSE: {}", v);
        match core::str::from_utf8(val) {
            Ok(val) => match val {
                // TODO more values
                x if x.starts_with("Auto-Join") => Ok(JoinUrc::AutoJoin(AutoJoin::Off)),
                x if x.starts_with("Start") => Ok(JoinUrc::Start),
                x if x.starts_with("NORMAL") => Ok(JoinUrc::Normal),
                x if x.starts_with("Join failed") => Ok(JoinUrc::Failed),
                x if x.starts_with("Joined already") => Ok(JoinUrc::JoinedAlready),
                x if x.starts_with("Network joined") => Ok(JoinUrc::NetworkJoined),
                x if x.starts_with("NetID") => {
                    let mut s = x.split(" ");
                    let net_id = s.nth(1).ok_or(ParseError::NoMatch)?;
                    let dev_addr = s.nth(1).ok_or(ParseError::NoMatch)?;
                    Ok(JoinUrc::Success(net_id.into(), dev_addr.into()))
                }
                x if x.starts_with("Done") => Ok(JoinUrc::Done),
                _ => Err(ParseError::NoMatch),
            },
            _ => Err(ParseError::NoMatch),
        }
    }
}
