use crate::rf_explorer::{parsers::*, Message, ParseFromBytes};
use nom::{bytes::complete::tag, combinator::map_res, IResult};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::convert::TryFrom;

#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum DspMode {
    Auto = 0,
    Filter,
    Fast,
}

impl Message for DspMode {
    const PREFIX: &'static [u8] = b"DSP:";
}

impl ParseFromBytes for DspMode {
    fn parse_from_bytes(bytes: &[u8]) -> IResult<&[u8], Self> {
        // Parse the prefix of the message
        let (bytes, _) = tag(DspMode::PREFIX)(bytes)?;

        // Parse the DSP mode
        let (bytes, dsp_mode) = map_res(parse_num::<u8>(1u8), DspMode::try_from)(bytes)?;

        // Consume \r or \r\n line ending and make sure there aren't any bytes left
        let (bytes, _) = parse_opt_line_ending(bytes)?;

        Ok((bytes, dsp_mode))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accept_valid_dsp_mode_message() {
        let bytes = b"DSP:0";
        let dsp_mode = DspMode::parse_from_bytes(bytes.as_ref()).unwrap().1;
        assert_eq!(dsp_mode, DspMode::Auto);
    }
}
