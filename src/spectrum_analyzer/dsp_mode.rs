use crate::rf_explorer::Message;
use nom::{
    bytes::complete::tag,
    character::complete::line_ending,
    combinator::{all_consuming, map_res, opt},
    number::complete::u8 as nom_u8,
    IResult,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::convert::TryFrom;

#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum DspMode {
    Auto = b'0',
    Filter = b'1',
    Fast = b'2',
}

impl DspMode {
    const PREFIX: &'static [u8] = b"DSP:";
}

impl Message for DspMode {
    fn from_bytes(bytes: &[u8]) -> IResult<&[u8], Self> {
        // Parse the prefix of the message
        let (bytes, _) = tag(DspMode::PREFIX)(bytes)?;

        // Parse the DSP mode
        let (bytes, dsp_mode) = map_res(nom_u8, DspMode::try_from)(bytes)?;

        // Consume \r or \r\n line ending and make sure there aren't any bytes left
        let (bytes, _) = all_consuming(opt(line_ending))(bytes)?;

        Ok((bytes, dsp_mode))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accept_valid_dsp_mode_message() {
        let bytes = b"DSP:0";
        let dsp_mode = DspMode::from_bytes(bytes.as_ref()).unwrap().1;
        assert_eq!(dsp_mode, DspMode::Auto);
    }
}
