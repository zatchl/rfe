use super::{
    Config, ConfigAmpSweep, ConfigAmpSweepExp, ConfigCw, ConfigCwExp, ConfigExp, ConfigFreqSweep,
    ConfigFreqSweepExp, Model, Temperature,
};
use crate::common::{ScreenData, SerialNumber, SetupInfo};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Message {
    Config(Config),
    ConfigAmpSweep(ConfigAmpSweep),
    ConfigCw(ConfigCw),
    ConfigFreqSweep(ConfigFreqSweep),
    ScreenData(ScreenData),
    SerialNumber(SerialNumber),
    SetupInfo(SetupInfo<Model>),
    Temperature(Temperature),
}

impl crate::common::Message for Message {
    fn parse(bytes: &[u8]) -> Result<Message, crate::common::MessageParseError> {
        if bytes.starts_with(Config::PREFIX) {
            Ok(Message::Config(Config::parse(bytes)?.1))
        } else if bytes.starts_with(ConfigAmpSweep::PREFIX) {
            Ok(Message::ConfigAmpSweep(ConfigAmpSweep::parse(bytes)?.1))
        } else if bytes.starts_with(ConfigCw::PREFIX) {
            Ok(Message::ConfigCw(ConfigCw::parse(bytes)?.1))
        } else if bytes.starts_with(ConfigFreqSweep::PREFIX) {
            Ok(Message::ConfigFreqSweep(ConfigFreqSweep::parse(bytes)?.1))
        } else if bytes.starts_with(ScreenData::PREFIX) {
            Ok(Message::ScreenData(ScreenData::parse(bytes)?.1))
        } else if bytes.starts_with(SerialNumber::PREFIX) {
            Ok(Message::SerialNumber(SerialNumber::parse(bytes)?.1))
        } else if bytes.starts_with(SetupInfo::<Model>::PREFIX) {
            Ok(Message::SetupInfo(SetupInfo::<Model>::parse(bytes)?.1))
        } else if bytes.starts_with(Temperature::PREFIX) {
            Ok(Message::Temperature(Temperature::parse(bytes)?.1))
        } else {
            Err(crate::common::MessageParseError::UnknownMessageType)
        }
    }
}
