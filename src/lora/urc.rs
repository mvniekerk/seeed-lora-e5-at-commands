use crate::client::asynch::JoinStatus;
use crate::urc::{
    MessageStats, ReceivedMessage, URCMessages, LAST_LORA_MESSAGE_RECEIVED, LORA_JOIN_STATUS,
    LORA_MESSAGE_RECEIVED_COUNT, LORA_MESSAGE_RECEIVED_STATS,
};
use atat::digest::ParseError;
use atat::helpers::LossyStr;
use atat::nom::{branch, bytes, character, sequence};
#[cfg(feature = "debug")]
use defmt::{debug, error, trace};
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
        trace!("+JOIN PARSE: {}", v);
        let ret = match core::str::from_utf8(val) {
            Ok(val) => match val {
                x if x.starts_with("Start") => Ok(JoinUrc::Start),
                x if x.starts_with("Auto-Join") => Ok(JoinUrc::AutoJoin(AutoJoin::Off)),
                x if x.starts_with("Start") => Ok(JoinUrc::Start),
                x if x.starts_with("NORMAL") => Ok(JoinUrc::Normal),
                x if x.starts_with("Join failed") => Ok(JoinUrc::Failed),
                x if x.starts_with("Joined already") => Ok(JoinUrc::JoinedAlready),
                x if x.starts_with("Network joined") => Ok(JoinUrc::NetworkJoined),
                x if x.starts_with("NetID") => {
                    let mut s = x.split(' ');
                    let net_id = s.nth(1).ok_or(ParseError::NoMatch)?;
                    let dev_addr = s.nth(1).ok_or(ParseError::NoMatch)?;
                    Ok(JoinUrc::Success(net_id.into(), dev_addr.into()))
                }
                x if x.starts_with("Done") => Ok(JoinUrc::Done),
                _ => Err(ParseError::NoMatch),
            },
            _ => Err(ParseError::NoMatch),
        };
        match &ret {
            Ok(JoinUrc::Start | JoinUrc::Failed) => LORA_JOIN_STATUS.signal(JoinStatus::Joining),
            Ok(JoinUrc::NetworkJoined | JoinUrc::JoinedAlready | JoinUrc::Success(_, _)) => {
                LORA_JOIN_STATUS.signal(JoinStatus::Success)
            }
            _ => {}
        }
        ret
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum MessageHexSend {
    Start,
    Pending,
    AckReceived,
    WaitAck,
    RxWinRssiSnr(u8, i8, f32),
    Done,
}

impl From<MessageHexSend> for URCMessages {
    fn from(value: MessageHexSend) -> Self {
        Self::MessageHexSend(value)
    }
}

impl MessageHexSend {
    pub(crate) fn parse(buf: &[u8]) -> Result<Self, ParseError> {
        let (val, _) = branch::alt((
            bytes::streaming::tag("+MSGHEX: "),
            bytes::streaming::tag("+CMSGHEX: "),
        ))(buf)?;
        let v = LossyStr(val);
        #[cfg(feature = "debug")]
        trace!("+(C)MSGHEX PARSE: {}", v);
        match val {
            x if x.starts_with(b"Start") => Ok(MessageHexSend::Start),
            x if x.starts_with(b"ACK Received") => Ok(MessageHexSend::AckReceived),
            x if x.starts_with(b"Wait ACK") => Ok(MessageHexSend::WaitAck),
            x if x.starts_with(b"FPENDING") => Ok(MessageHexSend::Pending),
            x if x.starts_with(b"RXWIN") => {
                let (_, (_, rxwin, _, rssi, _, snr)) = sequence::tuple((
                    bytes::streaming::tag(b"RXWIN"),
                    bytes::streaming::take_until(","),
                    bytes::streaming::tag(b", RSSI "),
                    bytes::streaming::take_until(","),
                    bytes::streaming::tag(b", SNR "),
                    bytes::streaming::take_while(character::is_alphanumeric),
                ))(x)?;
                let rxwin = core::str::from_utf8(rxwin).map_err(|_| ParseError::NoMatch)?;
                let rssi = core::str::from_utf8(rssi).map_err(|_| ParseError::NoMatch)?;
                let snr = core::str::from_utf8(snr).map_err(|_| ParseError::NoMatch)?;
                #[cfg(feature = "debug")]
                trace!("rxwin: {}, rssi: {}, snr: {}", rxwin, rssi, snr);
                let rxwin = rxwin.parse().unwrap_or(0);
                let rssi = rssi.parse().unwrap_or(0);
                let snr = snr.parse().unwrap_or(0.0);
                Ok(MessageHexSend::RxWinRssiSnr(rxwin, rssi, snr))
            }
            x if x.starts_with(b"Done") => Ok(MessageHexSend::Done),
            _ => Err(ParseError::NoMatch),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Payload {
    pub port: u8,
    pub length: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum MessageReceived {
    Payload(Payload),
    RxWinRssiSnr(u8, i8, f32),
    FPending,
    Done,
}

impl From<MessageReceived> for URCMessages {
    fn from(value: MessageReceived) -> Self {
        Self::MessageReceived(value)
    }
}

impl MessageReceived {
    pub(crate) fn parse(buf: &[u8]) -> Result<Self, ParseError> {
        let (val, _) = sequence::tuple((bytes::streaming::tag("+MSG: "),))(buf)?;
        let v = LossyStr(val);
        #[cfg(feature = "debug")]
        trace!("+MSG PARSE: {}", v);
        match val {
            x if x.starts_with(b"PORT: ") => {
                let (_, (_, port, _, payload_str, _)) = sequence::tuple((
                    bytes::streaming::tag("PORT: "),
                    bytes::streaming::take_until(";"),
                    bytes::streaming::tag("; RX: \""),
                    bytes::streaming::take_until("\""),
                    bytes::streaming::tag("\""),
                ))(x)
                .map_err(|e| {
                    #[cfg(feature = "debug")]
                    error!("Error on +MSG Port parse");
                    e
                })?;
                let payload_str_len = payload_str.len();
                #[cfg(feature = "debug")]
                debug!("Payload str [{}]{}", payload_str_len, LossyStr(payload_str));
                let length = payload_str_len / 2 + if payload_str_len % 2 != 0 { 1 } else { 0 };
                let mut payload = [0u8; 243];

                for (index, val) in payload_str.iter().enumerate().take(payload_str_len) {
                    let val_bytes = [*val];
                    let val_bytes =
                        core::str::from_utf8(&val_bytes).map_err(|_| ParseError::NoMatch)?;
                    let val = u8::from_str_radix(val_bytes, 16).map_err(|_| ParseError::NoMatch)?;
                    let val = if index % 2 == 0 { val << 4 } else { val };
                    payload[index / 2] += val;
                }

                let port = core::str::from_utf8(port)
                    .map_err(|_| ParseError::NoMatch)?
                    .parse()
                    .map_err(|_| ParseError::NoMatch)?;
                LORA_MESSAGE_RECEIVED_STATS.reset();

                LAST_LORA_MESSAGE_RECEIVED.signal(ReceivedMessage {
                    payload,
                    length,
                    port,
                });

                let count = match LORA_MESSAGE_RECEIVED_COUNT.try_signaled_value() {
                    Some(v) => {
                        if v == u32::MAX {
                            0
                        } else {
                            v + 1
                        }
                    }
                    None => 1,
                };
                LORA_MESSAGE_RECEIVED_COUNT.signal(count);

                Ok(MessageReceived::Payload(Payload { length, port }))
            }
            x if x.starts_with(b"RXWIN") => {
                let (_, (_, rxwin, _, rssi, _, snr)) = sequence::tuple((
                    bytes::streaming::tag(b"RXWIN"),
                    bytes::streaming::take_until(","),
                    bytes::streaming::tag(b", RSSI "),
                    bytes::streaming::take_until(","),
                    bytes::streaming::tag(b", SNR "),
                    bytes::streaming::take_while(character::is_alphanumeric),
                ))(x)?;

                let rxwin = core::str::from_utf8(rxwin).map_err(|_| ParseError::NoMatch)?;
                let rssi = core::str::from_utf8(rssi).map_err(|_| ParseError::NoMatch)?;
                let snr = core::str::from_utf8(snr).map_err(|_| ParseError::NoMatch)?;
                #[cfg(feature = "debug")]
                trace!("rxwin: {}, rssi: {}, snr: {}", rxwin, rssi, snr);
                let rxwin = rxwin.parse().map_err(|_| ParseError::NoMatch)?;
                let rssi = rssi.parse().map_err(|_| ParseError::NoMatch)?;
                let snr = snr.parse().map_err(|_| ParseError::NoMatch)?;

                LORA_MESSAGE_RECEIVED_STATS.signal(MessageStats { rxwin, rssi, snr });

                Ok(MessageReceived::RxWinRssiSnr(rxwin, rssi, snr))
            }
            x if x.starts_with(b"Done") => Ok(MessageReceived::Done),
            x if x.starts_with(b"FPENDING") => Ok(MessageReceived::FPending),
            _ => Err(ParseError::NoMatch),
        }
    }
}
