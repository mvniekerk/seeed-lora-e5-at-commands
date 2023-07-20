//! # URC parser implementation
//!
//! This is just used internally, but needs to be public for passing [URCMessages] as a generic to
//! [AtDigester](atat::digest::AtDigester): `AtDigester<URCMessages>`.

use crate::lora::urc::JoinUrc;
use atat::digest::ParseError;
use atat::{
    nom::{branch, bytes, combinator, sequence},
    AtatUrc, Parser,
};

/// URC definitions, needs to passed as generic of [AtDigester](atat::digest::AtDigester): `AtDigester<URCMessages>`
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum URCMessages {
    /// Unknown URC message
    Unknown,
    SoftwareVersion(u8, u8, u8),
    Join(JoinUrc),
}

impl URCMessages {}

impl AtatUrc for URCMessages {
    type Response = Self;

    fn parse(resp: &[u8]) -> Option<Self::Response> {
        match resp {
            b if b.starts_with(b"+JOIN: ") => JoinUrc::parse(resp).ok().map(URCMessages::Join),
            _ => None,
        }
    }
}

impl Parser for URCMessages {
    fn parse(buf: &[u8]) -> Result<(&[u8], usize), ParseError> {
        // Check if this is a join started message
        match buf {
            b if b.starts_with(b"+JOIN: Start\r\n") => return Err(ParseError::NoMatch),
            b if b.starts_with(b"+JOIN: Auto-Join ") => return Err(ParseError::NoMatch),
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
        ))(buf)?;
        Ok((data, head.len() + data.len() + tail.len()))
    }
}
