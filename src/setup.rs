use crate::RfExplorerModel;
use std::{convert::TryFrom, str, str::FromStr};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct RfExplorerSetup {
    main_model: RfExplorerModel,
    expansion_model: Option<RfExplorerModel>,
    firmware_version: String,
}

#[derive(Error, Debug)]
pub enum ParseSetupError {
    #[error(transparent)]
    InvalidRfExplorerModel(#[from] <RfExplorerModel as TryFrom<u8>>::Error),

    #[error("Invalid RfExplorerSetup: expected bytes to start with #C2-M:")]
    InvalidFormat,

    #[error("A required field is missing from the bytes")]
    MissingField,

    #[error(transparent)]
    InvalidInt(#[from] std::num::ParseIntError),

    #[error(transparent)]
    InvalidUtf8(#[from] std::str::Utf8Error),

    #[error(transparent)]
    InvalidString(#[from] std::string::ParseError),
}

impl RfExplorerSetup {
    pub fn main_model(&self) -> RfExplorerModel {
        self.main_model
    }

    pub fn expansion_model(&self) -> Option<RfExplorerModel> {
        self.expansion_model
    }

    pub fn firmware_version(&self) -> &str {
        self.firmware_version.as_str()
    }
}

impl TryFrom<&[u8]> for RfExplorerSetup {
    type Error = ParseSetupError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.starts_with(b"#C2-M:") {
            let mut fields = value
                .get(6..)
                .ok_or_else(|| ParseSetupError::MissingField)?
                .split(|&byte| byte == b',');

            Ok(RfExplorerSetup {
                main_model: RfExplorerModel::try_from(parse_field::<u8>(fields.next())?)?,
                expansion_model: RfExplorerModel::try_from(parse_field::<u8>(fields.next())?).ok(),
                firmware_version: parse_field::<String>(fields.next())?,
            })
        } else {
            Err(ParseSetupError::InvalidFormat)
        }
    }
}

fn parse_field<T>(field: Option<&[u8]>) -> Result<T, ParseSetupError>
where
    T: FromStr,
    ParseSetupError: From<T::Err>,
{
    Ok(T::from_str(
        str::from_utf8(field.ok_or_else(|| ParseSetupError::MissingField)?)?.trim(),
    )?)
}

#[cfg(test)]
mod tests {}
