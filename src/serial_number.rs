use std::{convert::TryFrom, str};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct RfExplorerSerialNumber {
    serial_number: String,
}

#[derive(Error, Debug)]
pub enum ParseSerialNumberError {
    #[error(
        "Invalid RfExplorerSerialNumber: expected bytes to start with #Sn and be 19 bytes long"
    )]
    InvalidFormat,

    #[error(transparent)]
    InvalidUtf8(#[from] str::Utf8Error),
}

impl RfExplorerSerialNumber {
    const MESSAGE_LENGTH: usize = 19;

    pub fn serial_number(&self) -> &str {
        &self.serial_number
    }
}

impl TryFrom<&[u8]> for RfExplorerSerialNumber {
    type Error = ParseSerialNumberError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.starts_with(b"#Sn") && bytes.len() == RfExplorerSerialNumber::MESSAGE_LENGTH {
            Ok(RfExplorerSerialNumber {
                serial_number: str::from_utf8(&bytes[3..])?.to_string(),
            })
        } else {
            Err(ParseSerialNumberError::InvalidFormat)
        }
    }
}
