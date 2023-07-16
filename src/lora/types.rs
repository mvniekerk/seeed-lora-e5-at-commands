use core::str::FromStr;
use heapless::String;
use crate::lora::commands::LoraClassSet;
use crate::lora::responses::ModeGetSetResponse;

#[derive(Debug, Clone, PartialEq)]
pub enum LoraJoinMode {
    Test,
    Otaa,
    Abp,
    _Unknown,
}

impl From<ModeGetSetResponse> for LoraJoinMode {
    fn from(value: ModeGetSetResponse) -> Self {
        match value.mode.as_str() {
            "TEST" => Self::Test,
            "LWOTAA" => Self::Otaa,
            "LWABP" => Self::Abp,
            _ => Self::_Unknown
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoraRegion {
    Eu868,
    US915,
    Us915Hybrid,
    Cn779,
    Eu433,
    Au915,
    Au915Old,
    Cn470,
    As923,
    Kr920,
    In865,
    Ru864,
    Cn470Prequel,
    Ste920,
    Jp920,
    Unknown
}

impl From<LoraRegion> for String<24> {
    fn from(value: LoraRegion) -> Self {
        match value {
            LoraRegion::Eu868 => "EU868".into(),
            LoraRegion::US915 => "US915".into(),
            LoraRegion::Us915Hybrid => "US915HYBRID".into(),
            LoraRegion::Cn779 => "CN779".into(),
            LoraRegion::Eu433 => "EU433".into(),
            LoraRegion::Au915 => "AU915".into(),
            LoraRegion::Au915Old => "AU915OLD".into(),
            LoraRegion::Cn470 => "CN470".into(),
            LoraRegion::As923 => "AS923".into(),
            LoraRegion::Kr920 => "KR920".into(),
            LoraRegion::In865 => "IN865".into(),
            LoraRegion::Ru864 => "RU864".into(),
            LoraRegion::Cn470Prequel => "CN470PREQUEL".into(),
            LoraRegion::Ste920 => "STE920".into(),
            LoraRegion::Jp920 => "JP920".into(),
            _ => "UNKNOWN".into()
        }
    }
}

impl FromStr for LoraRegion {

    type Err = ();
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let v = match value {
            "EU868" => LoraRegion::Eu868,
            "US915" => LoraRegion::US915,
            "US915HYBRID" => LoraRegion::Us915Hybrid,
            "CN779" => LoraRegion::Cn779,
            "EU433" => LoraRegion::Eu433,
            "AU915" => LoraRegion::Au915,
            "AU915OLD" => LoraRegion::Au915Old,
            "CN470" => LoraRegion::Cn470,
            "AS923" => LoraRegion::As923,
            "KR920" => LoraRegion::Kr920,
            "IN865" => LoraRegion::In865,
            "RU864" => LoraRegion::Ru864,
            "CN470PREQUEL" => LoraRegion::Cn470Prequel,
            "STE920" => LoraRegion::Ste920,
            "JP920" => LoraRegion::Jp920,
            _ => LoraRegion::Unknown
        };
        Ok(v)
    }
}

impl From<String<24>> for LoraRegion {
    fn from(value: String<24>) -> Self {
        Self::from_str(value.as_str()).unwrap_or(Self::Unknown)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoraClass {
    ClassA,
    ClassB,
    ClassC,
    Unknown,
}

impl From<String<2>> for LoraClass {
    fn from(value: String<2>) -> Self {
        match value.as_str() {
            "A" => Self::ClassA,
            "B" => Self::ClassB,
            "C" => Self::ClassC,
            _ => Self::Unknown,
        }
    }
}

impl From<LoraClass> for String<2> {
    fn from(value: LoraClass) -> Self {
        match value {
            LoraClass::ClassA => "A".into(),
            LoraClass::ClassB => "B".into(),
            LoraClass::ClassC => "C".into(),
            LoraClass::Unknown => "".into(),
        }
    }
}

impl LoraClass {
    pub fn set_cmd(self) -> LoraClassSet {
        LoraClassSet { class: self.into() }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoraJoiningStartingStatus {
    Starting,
    Normal,
    Done(String<12>, String<22>)
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoraJoiningStatus {
    Starting(LoraJoiningStartingStatus),
    Failed,
    Busy,
    Unknown
}