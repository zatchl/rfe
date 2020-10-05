use crate::messages::ParseMessageError;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use rfe_message::RfeMessage;
use std::{convert::TryFrom, str::FromStr};

#[derive(Debug, Copy, Clone, RfeMessage)]
#[prefix = "DSP:"]
pub struct DspModeMessage {
    dsp_mode: DspMode,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum DspMode {
    Auto = b'0',
    Filter = b'1',
    Fast = b'2',
}

impl DspModeMessage {
    pub fn dsp_mode(&self) -> DspMode {
        self.dsp_mode
    }
}

impl FromStr for DspMode {
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
    fn accept_valid_dsp_mode_message() {
        let bytes = b"DSP:0";
        let dsp_mode_message = DspModeMessage::try_from(bytes.as_ref()).unwrap();
        assert_eq!(dsp_mode_message.dsp_mode(), DspMode::Auto);
    }
}
