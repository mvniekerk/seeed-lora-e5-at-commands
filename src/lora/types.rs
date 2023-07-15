use heapless::String;
use crate::lora::commands::LoraClassSet;
use crate::lora::responses::ModeGetSetResponse;

#[derive(Debug, Clone, PartialEq)]
pub enum LoraMode {
    Test,
    Otaa,
    Abp,
    _Unknown,
}

impl From<ModeGetSetResponse> for LoraMode {
    fn from(value: ModeGetSetResponse) -> Self {
        match value.mode.as_str() {
            "TEST" => Self::Test,
            "LWOTAA" => Self::Otaa,
            "LWABP" => Self::Abp,
            _ => Self::_Unknown
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoraRegion {
    EU868,
    US915,
    US915HYBRID,
    CN779,
    EU433,
    AU915,
    AU915OLD,
    CN470,
    AS923,
    KR920,
    IN865,
    RU864,
    CN470PREQUEL,
    STE920,
    JP920,
    Unknown
}

impl From<LoraRegion> for String<24> {
    fn from(value: LoraRegion) -> Self {
        match value {
            LoraRegion::EU868 => "EU868".into(),
            LoraRegion::US915 => "US915".into(),
            LoraRegion::US915HYBRID => "US915HYBRID".into(),
            LoraRegion::CN779 => "CN779".into(),
            LoraRegion::EU433 => "EU433".into(),
            LoraRegion::AU915 => "AU915".into(),
            LoraRegion::AU915OLD => "AU915OLD".into(),
            LoraRegion::CN470 => "CN470".into(),
            LoraRegion::AS923 => "AS923".into(),
            LoraRegion::KR920 => "KR920".into(),
            LoraRegion::IN865 => "IN865".into(),
            LoraRegion::RU864 => "RU864".into(),
            LoraRegion::CN470PREQUEL => "CN470PREQUEL".into(),
            LoraRegion::STE920 => "STE920".into(),
            LoraRegion::JP920 => "JP920".into(),
            _ => "UNKNOWN".into()
        }
    }
}

impl From<String<24>> for LoraRegion {
    fn from(value: String<24>) -> Self {
        match value.as_str() {
            "EU868" => LoraRegion::EU868,
            "US915" => LoraRegion::US915,
            "US915HYBRID" => LoraRegion::US915HYBRID,
            "CN779" => LoraRegion::CN779,
            "EU433" => LoraRegion::EU433,
            "AU915" => LoraRegion::AU915,
            "AU915OLD" => LoraRegion::AU915OLD,
            "CN470" => LoraRegion::CN470,
            "AS923" => LoraRegion::AS923,
            "KR920" => LoraRegion::KR920,
            "IN865" => LoraRegion::IN865,
            "RU864" => LoraRegion::RU864,
            "CN470PREQUEL" => LoraRegion::CN470PREQUEL,
            "STE920" => LoraRegion::STE920,
            "JP920" => LoraRegion::JP920,
            _ => LoraRegion::EU868
        }
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