//! # URC parser implementation
//!
//! This is just used internally, but needs to be public for passing [URCMessages] as a generic to
//! [AtDigester](atat::digest::AtDigester): `AtDigester<URCMessages>`.

use crate::lora::types::LoraRegion;
use atat::digest::ParseError;
#[cfg(feature = "debug")]
use atat::helpers::LossyStr;
use atat::{
    nom::{branch, bytes, character, combinator, sequence},
    AtatUrc, Parser,
};
#[cfg(feature = "debug")]
use defmt::error;
use crate::lora::urc::JoinUrc;

/// URC definitions, needs to passed as generic of [AtDigester](atat::digest::AtDigester): `AtDigester<URCMessages>`
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum URCMessages {
    /// Unknown URC message
    Unknown,
    SoftwareVersion(u8, u8, u8),
    Join(JoinUrc),
}

impl URCMessages {
    pub(crate) fn parse_software_version(buf: &[u8]) -> Result<URCMessages, ParseError> {
        let (_, (_, major, _, minor, _, patch)) = sequence::tuple((
            bytes::streaming::tag("+VER: "),
            bytes::streaming::take_while(character::is_digit),
            bytes::streaming::tag("."),
            bytes::streaming::take_while(character::is_digit),
            bytes::streaming::tag("."),
            bytes::streaming::take_while(character::is_digit),
        ))(buf)?;

        match (
            core::str::from_utf8(major),
            core::str::from_utf8(minor),
            core::str::from_utf8(patch),
        ) {
            (Ok(major), Ok(minor), Ok(patch)) => match (
                major.parse::<u8>(),
                minor.parse::<u8>(),
                patch.parse::<u8>(),
            ) {
                (Ok(major), Ok(minor), Ok(patch)) => {
                    Ok(URCMessages::SoftwareVersion(major, minor, patch))
                }
                _ => {
                    #[cfg(feature = "debug")]
                    error!("Failed to parse u8 values for software version");
                    Err(ParseError::NoMatch)
                }
            },
            _ => {
                #[cfg(feature = "debug")]
                error!(
                    "Failed to parse software version strings [{:?}, {:?}, {:?}]",
                    LossyStr(major),
                    LossyStr(minor),
                    LossyStr(patch)
                );
                Err(ParseError::NoMatch)
            }
        }
    }
}

impl AtatUrc for URCMessages {
    type Response = Self;

    fn parse(resp: &[u8]) -> Option<Self::Response> {
        match resp {
            b if b.starts_with(b"+VER: ") => URCMessages::parse_software_version(resp).ok(),
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
            // Software version
            sequence::tuple((
                combinator::success(&b""[..]),
                combinator::recognize(sequence::tuple((
                    bytes::streaming::tag("+VER: "),
                    bytes::streaming::take_while(character::is_digit),
                    bytes::streaming::tag("."),
                    bytes::streaming::take_while(character::is_digit),
                    bytes::streaming::tag("."),
                    bytes::streaming::take_while(character::is_digit),
                ))),
                bytes::streaming::tag("\r\n"),
            )),
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
