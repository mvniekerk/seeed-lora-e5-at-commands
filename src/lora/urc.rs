use atat::digest::ParseError;
use atat::helpers::LossyStr;
use atat::nom::{branch, bytes, character, combinator, sequence};
use defmt::info;
use heapless::String;
use crate::urc::URCMessages;

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
    Success(String<12>, String<22>),
    Done
}

impl From<JoinUrc> for URCMessages {
    fn from(value: JoinUrc) -> Self {
        Self::Join(value)
    }
}

impl JoinUrc {
    pub(crate) fn parse(buf: &[u8]) -> Result<Self, ParseError> {
        let (val, _) = sequence::tuple((
            bytes::streaming::tag("+JOIN: "),
        ))(buf)?;
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
                x if x.starts_with("NetID") => {
                    let (_, (_, net_id, _, dev_addr)) = sequence::tuple((
                        bytes::streaming::tag("NetID "),
                        bytes::streaming::take_until(" "),
                        bytes::streaming::tag(" DevAddr"),
                        combinator::success(&b""[..]),
                    ))(buf)?;
                    match (core::str::from_utf8(net_id), core::str::from_utf8(dev_addr)) {
                        (Ok(dev_eui), Ok(app_eui)) => {
                            Ok(JoinUrc::Success(dev_eui.into(), app_eui.into()))
                        }
                        _ => Err(ParseError::NoMatch),
                    }
                },
                x if x.starts_with("Done") => Ok(JoinUrc::Done),
                _ => Err(ParseError::NoMatch),
            },
            _ => Err(ParseError::NoMatch),
        }
    }
}