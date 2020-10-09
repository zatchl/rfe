use crate::messages::{ParseMessageError, RfeMessage};
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;

#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum TrackingStatus {
    Disabled = 0,
    Enabled,
}

impl RfeMessage for TrackingStatus {
    const PREFIX: &'static [u8] = b"#K";
}

impl TryFrom<&[u8]> for TrackingStatus {
    type Error = ParseMessageError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if let [b'#', b'K', tracking_byte] = bytes {
            TrackingStatus::try_from(*tracking_byte).map_err(|_| ParseMessageError::InvalidData)
        } else if let [tracking_byte] = bytes {
            TrackingStatus::try_from(*tracking_byte).map_err(|_| ParseMessageError::InvalidData)
        } else {
            Err(ParseMessageError::InvalidData)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accept_valid_tracking_status_message() {
        let bytes = [b'#', b'K', 0];
        let tracking_status = TrackingStatus::try_from(bytes.as_ref()).unwrap();
        assert_eq!(tracking_status, TrackingStatus::Disabled);
    }
}
