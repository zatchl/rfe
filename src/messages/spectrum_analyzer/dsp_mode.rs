use crate::messages::{ParseMessageError, RfeMessage};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::convert::TryFrom;

#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum DspMode {
    Auto = b'0',
    Filter = b'1',
    Fast = b'2',
}

impl RfeMessage for DspMode {
    const PREFIX: &'static [u8] = b"DSP:";
}

impl TryFrom<&[u8]> for DspMode {
    type Error = ParseMessageError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if let [b'D', b'S', b'P', b':', dsp_byte] = bytes {
            DspMode::try_from(*dsp_byte).map_err(|_| ParseMessageError::InvalidData)
        } else if let [dsp_byte] = bytes {
            DspMode::try_from(*dsp_byte).map_err(|_| ParseMessageError::InvalidData)
        } else {
            Err(ParseMessageError::InvalidData)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accept_valid_dsp_mode_message() {
        let bytes = b"DSP:0";
        let dsp_mode = DspMode::try_from(bytes.as_ref()).unwrap();
        assert_eq!(dsp_mode, DspMode::Auto);
    }
}
