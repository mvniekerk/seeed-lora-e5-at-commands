//! # URC parser implementation
//!
//! This is just used internally, but needs to be public for passing [URCMessages] as a generic to
//! [AtDigester](atat::digest::AtDigester): `AtDigester<URCMessages>`.

use crate::lora::types::LoraRegion;
use crate::urc::URCMessages::SystemStart;
use atat::digest::ParseError;
#[cfg(feature = "debug")]
use atat::helpers::LossyStr;
use atat::{
    nom::{branch, bytes, character, combinator, sequence},
    AtatUrc, Parser,
};
#[cfg(feature = "debug")]
use defmt::error;

/// URC definitions, needs to passed as generic of [AtDigester](atat::digest::AtDigester): `AtDigester<URCMessages>`
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum URCMessages {
    /// Unknown URC message
    Unknown,
    AteOn,
    AteOff,
    SystemStart,
    SoftwareVersion(u8, u8, u8),
    LoraVersion(u32),
    LoraRegion(LoraRegion),
    NextTxInSeconds(u16)
}

impl URCMessages {
    pub(crate) fn parse_software_version(buf: &[u8]) -> Result<URCMessages, ParseError> {
        let (_, (_, major, _, minor, _, patch)) = sequence::tuple((
            bytes::streaming::tag("SOFT VERSION:"),
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

    pub(crate) fn parse_lora_version(buf: &[u8]) -> Result<URCMessages, ParseError> {
        let (_, (_, version)) = sequence::tuple((
            bytes::streaming::tag(b"LORA VERSION:"),
            bytes::streaming::take_while(character::is_digit),
        ))(buf)?;
        let version = core::str::from_utf8(version)
            .map_err(|_e| {
                #[cfg(feature = "debug")]
                error!(
                    "Failed to parse lora version string [{:?}]",
                    LossyStr(version)
                );
                ParseError::NoMatch
            })?
            .parse::<u32>()
            .map_err(|_e| {
                #[cfg(feature = "debug")]
                error!(
                    "Failed to parse lora version integer {:?}",
                    LossyStr(version)
                );
                ParseError::NoMatch
            })?;
        Ok(URCMessages::LoraVersion(version))
    }

    pub(crate) fn parse_lora_region(buf: &[u8]) -> Result<URCMessages, ParseError> {
        let (_, (_, region)) = sequence::tuple((
            bytes::streaming::tag(b"LORA REGION:"),
            bytes::streaming::take_while(character::is_alphanumeric),
        ))(buf)?;
        let region = core::str::from_utf8(region)
            .map_err(|_e| {
                #[cfg(feature = "debug")]
                error!("Failed to parse lora region string, {:?}", LossyStr(region));
                ParseError::NoMatch
            })?
            .parse::<LoraRegion>()
            .map_err(|_| {
                #[cfg(feature = "debug")]
                error!("Failed to parse lora region from string");
                ParseError::NoMatch
            })?;
        Ok(URCMessages::LoraRegion(region))
    }

    pub(crate) fn parse_next_tx(buf: &[u8]) -> Result<URCMessages, ParseError> {
        let (_, (_, seconds)) = sequence::tuple((
            bytes::streaming::tag(b"NEXT TX after(s):"),
            bytes::streaming::take_while(character::is_digit),
        ))(buf)?;
        let seconds = core::str::from_utf8(seconds)
            .map_err(|_e| {
                #[cfg(feature = "debug")]
                error!("Failed to parse next tx seconds, {:?}", LossyStr(seconds));
                ParseError::NoMatch
            })?
            .parse::<u16>()
            .map_err(|_| {
                #[cfg(feature = "debug")]
                error!("Failed to parse next tx seconds from string");
                ParseError::NoMatch
            })?;
        Ok(URCMessages::NextTxInSeconds(seconds))
    }
}

impl AtatUrc for URCMessages {
    type Response = Self;

    fn parse(resp: &[u8]) -> Option<Self::Response> {
        match resp {
            b"SYSTEM START" => Some(SystemStart),
            b if b.starts_with(b"SOFT VERSION:") => URCMessages::parse_software_version(resp).ok(),
            b if b.starts_with(b"LORA VERSION:") => URCMessages::parse_lora_version(resp).ok(),
            b if b.starts_with(b"LORA REGION:") => URCMessages::parse_lora_region(resp).ok(),
            b if b.starts_with(b"NEXT TX after(s):") => URCMessages::parse_next_tx(resp).ok(),
            _ => None,
        }
    }
}

impl Parser for URCMessages {
    fn parse(buf: &[u8]) -> Result<(&[u8], usize), ParseError> {
        let (_reminder, (head, data, tail)) = branch::alt((
            // System start
            sequence::tuple((
                bytes::streaming::tag("====================="),
                bytes::streaming::tag("SYSTEM START"),
                bytes::streaming::tag("=====================\r\n\r\n"),
            )),
            // Software version
            sequence::tuple((
                bytes::streaming::tag("==================="),
                combinator::recognize(sequence::tuple((
                    bytes::streaming::tag("SOFT VERSION:"),
                    bytes::streaming::take_while(character::is_digit),
                    bytes::streaming::tag("."),
                    bytes::streaming::take_while(character::is_digit),
                    bytes::streaming::tag("."),
                    bytes::streaming::take_while(character::is_digit),
                ))),
                bytes::streaming::tag("==============\r\n\r\n"),
            )),
            // Lora version
            sequence::tuple((
                bytes::streaming::tag("==================="),
                combinator::recognize(sequence::tuple((
                    bytes::streaming::tag("LORA VERSION:"),
                    bytes::streaming::take_while(character::is_digit),
                ))),
                bytes::streaming::tag("===============\r\n\r\n"),
            )),
            // Lora region
            sequence::tuple((
                bytes::streaming::tag("==================="),
                combinator::recognize(sequence::tuple((
                    bytes::streaming::tag("LORA REGION:"),
                    bytes::streaming::take_while(character::is_alphanumeric),
                ))),
                bytes::streaming::tag("==================\r\n"),
            )),
            // Next TX
            sequence::tuple((
                combinator::success(&b""[..]),
                combinator::recognize(sequence::tuple((
                    bytes::streaming::tag("NEXT TX after(s):"),
                    bytes::streaming::take_while(character::is_digit),
                ))),
                bytes::streaming::tag("\r\n"),
            ))
        ))(buf)?;
        Ok((data, head.len() + data.len() + tail.len()))
    }
}
