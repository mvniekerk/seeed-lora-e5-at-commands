//! # URC parser implementation
//!
//! This is just used internally, but needs to be public for passing [URCMessages] as a generic to
//! [AtDigester](atat::digest::AtDigester): `AtDigester<URCMessages>`.

use crate::client::asynch::JoinStatus;
use crate::lora::urc::{JoinUrc, MessageHexSend, MessageReceived};
use crate::signal::Signal;
use atat::digest::ParseError;
use atat::{
    nom::{branch, bytes, combinator, sequence},
    AtatUrc, Parser,
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

#[cfg(feature = "debug")]
use embassy_sync::pipe::Pipe;

/// URC definitions, needs to passed as generic of [AtDigester](atat::digest::AtDigester): `AtDigester<URCMessages>`
#[derive(Debug, PartialEq, Clone)]
pub enum URCMessages {
    /// Unknown URC message
    Unknown,
    /// Join
    Join(JoinUrc),
    /// Message Hex Sen
    MessageHexSend(MessageHexSend),
    /// Message received
    MessageReceived(MessageReceived),
}

pub struct ReceivedMessage {
    pub port: u8,
    pub payload: [u8; 243],
    pub length: usize,
}

pub struct MessageStats {
    pub rxwin: u8,
    pub rssi: i8,
    pub snr: f32,
}

pub enum SentMessage {
    Failed,
    Success(MessageStats),
}

pub static LAST_LORA_MESSAGE_RECEIVED: Signal<CriticalSectionRawMutex, ReceivedMessage> =
    Signal::new();
pub static LORA_MESSAGE_RECEIVED_COUNT: Signal<CriticalSectionRawMutex, u32> = Signal::new();
pub static LORA_MESSAGE_RECEIVED_STATS: Signal<CriticalSectionRawMutex, MessageStats> =
    Signal::new();
pub static LAST_LORA_MESSAGE_SENT: Signal<CriticalSectionRawMutex, MessageStats> = Signal::new();
pub static LORA_JOIN_STATUS: Signal<CriticalSectionRawMutex, JoinStatus> = Signal::new();

#[cfg(feature = "debug")]
pub static LORA_LATEST_BUF: Pipe<CriticalSectionRawMutex, 50> = Pipe::new();

impl URCMessages {}

impl AtatUrc for URCMessages {
    type Response = Self;

    fn parse(resp: &[u8]) -> Option<Self::Response> {
        match resp {
            b if b.starts_with(b"+JOIN: ") => JoinUrc::parse(resp).ok().map(URCMessages::Join),
            b if b.starts_with(b"+MSGHEX: ") || b.starts_with(b"+CMSGHEX: ") => {
                MessageHexSend::parse(resp)
                    .ok()
                    .map(URCMessages::MessageHexSend)
            }
            b if b.starts_with(b"+MSG: ") => MessageReceived::parse(resp)
                .ok()
                .map(URCMessages::MessageReceived),
            _ => None,
        }
    }
}

impl Parser for URCMessages {
    fn parse(buf: &[u8]) -> Result<(&[u8], usize), ParseError> {
        // Check if this is a join started message
        match buf {
            b if b.starts_with(b"+JOIN: Auto-Join ") => return Err(ParseError::NoMatch),
            b if b.starts_with(b"+JOIN: Start") => return Err(ParseError::NoMatch),
            _ => {}
        }

        let (_reminder, (head, data, tail)) = branch::alt((
            // Join messages
            sequence::tuple((
                combinator::success(&b""[..]),
                combinator::recognize(sequence::tuple((
                    bytes::streaming::tag("+JOIN: "),
                    bytes::streaming::take_until("\r\n"),
                ))),
                bytes::streaming::tag("\r\n"),
            )),
            // Message Hex Send
            sequence::tuple((
                combinator::success(&b""[..]),
                combinator::recognize(sequence::tuple((
                    branch::alt((
                        bytes::streaming::tag("+MSGHEX: "),
                        bytes::streaming::tag("+CMSGHEX: "),
                    )),
                    bytes::streaming::take_until("\r\n"),
                ))),
                bytes::streaming::tag("\r\n"),
            )),
            // Message Hex Receive
            sequence::tuple((
                combinator::success(&b""[..]),
                combinator::recognize(sequence::tuple((
                    bytes::streaming::tag("+MSG: "),
                    bytes::streaming::take_until("\r\n"),
                ))),
                bytes::streaming::tag("\r\n"),
            )),
        ))(buf)?;
        Ok((data, head.len() + data.len() + tail.len()))
    }
}
