use crate::urc::URCMessages;
use atat::digest::ParseError;
use atat::helpers::LossyStr;
use atat::nom::{bytes, sequence};
#[cfg(feature = "debug")]
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
        #[cfg(feature = "debug")]
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

#[derive(Debug, PartialEq, Clone)]
pub enum MessageHexSend {
    Start,
    Pending,
    RxWinRssiSnr(u8, i8, f32),
    Done
}

impl From<MessageHexSend> for URCMessages {
    fn from(value: MessageHexSend) -> Self {
        Self::MessageHexSend(value)
    }
}

impl MessageHexSend {
    pub(crate) fn parse(buf: &[u8]) -> Result<Self, ParseError> {
        let (val, _) = sequence::tuple((bytes::streaming::tag("+MSGHEX: "),))(buf)?;
        let v = LossyStr(val);
        #[cfg(feature = "debug")]
        info!("+MSGHEX PARSE: {}", v);
        match core::str::from_utf8(val) {
            Ok(val) => match val {
                // TODO more values
                x if x.starts_with("Start") => Ok(MessageHexSend::Start),
                x if x.starts_with("FPENDING") => Ok(MessageHexSend::Pending),
                x if x.starts_with("RXWIN") => {
                    let x = &x[5..];
                    let mut s = x.split(",");
                    let rxwin = s.next().ok_or(ParseError::NoMatch)?;
                    let rssi = s.next().ok_or(ParseError::NoMatch)?;
                    let rssi = rssi.split(' ').nth(2).ok_or(ParseError::NoMatch)?;
                    let snr = s.next().ok_or(ParseError::NoMatch)?;
                    let snr = snr.split(' ').nth(2).ok_or(ParseError::NoMatch)?;
                    info!("rxwin: {}, rssi: {}, snr: {}", rxwin, rssi, snr);
                    Ok(MessageHexSend::RxWinRssiSnr(rxwin.parse().unwrap(), rssi.parse().unwrap(), snr.parse().unwrap()))
                }
                x if x.starts_with("Done") => Ok(MessageHexSend::Done),
                _ => Err(ParseError::NoMatch),
            },
            _ => Err(ParseError::NoMatch),
        }
    }
}