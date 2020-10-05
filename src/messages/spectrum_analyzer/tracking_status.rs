use crate::messages::ParseMessageError;
use num_enum::TryFromPrimitive;
use rfe_message::RfeMessage;
use std::{convert::TryFrom, str::FromStr};

#[derive(Debug, Copy, Clone, RfeMessage)]
#[prefix = "#K"]
pub struct TrackingStatusMessage {
    tracking_status: TrackingStatus,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum TrackingStatus {
    Disabled = 0,
    Enabled,
}

impl TrackingStatusMessage {
    pub fn tracking_status(&self) -> TrackingStatus {
        self.tracking_status
    }
}

impl FromStr for TrackingStatus {
    type Err = ParseMessageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(
            *s.as_bytes()
                .get(0)
                .ok_or_else(|| ParseMessageError::InvalidData)?,
        )
        .map_err(|_| ParseMessageError::InvalidData)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn accept_valid_tracking_status_message() {
        let bytes = [b'#', b'K', 0];
        let tracking_status_message = TrackingStatusMessage::try_from(bytes.as_ref()).unwrap();
        assert_eq!(
            tracking_status_message.tracking_status(),
            TrackingStatus::Disabled
        );
    }
}
