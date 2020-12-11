use crate::{rf_explorer::Message, Model};
use nom::{
    bytes::complete::{tag, take},
    character::complete::{line_ending, not_line_ending},
    combinator::{all_consuming, map, map_res, opt},
    IResult,
};
use std::{str, str::FromStr};

pub trait SetupInfo: Message + Sized {
    fn new(main_model: Model, exp_model: Option<Model>, fw_version: String) -> Self;

    fn parse_from_bytes(bytes: &[u8]) -> IResult<&[u8], Self> {
        // Parse the prefix of the message
        let (bytes, _) = tag(Self::PREFIX)(bytes)?;

        // Parse the main model
        let (bytes, main_model) =
            map_res(map_res(take(3u8), str::from_utf8), Model::from_str)(bytes)?;

        let (bytes, _) = tag(",")(bytes)?;

        // Parse the expansion model
        let (bytes, exp_model) = map(map_res(take(3u8), str::from_utf8), |s| {
            Model::from_str(s).ok()
        })(bytes)?;

        let (bytes, _) = tag(",")(bytes)?;

        // Parse the firmware version
        let (bytes, fw_version) =
            map(map_res(not_line_ending, str::from_utf8), str::to_string)(bytes)?;

        // Consume \r or \r\n line ending and make sure there aren't any bytes left
        let (bytes, _) = all_consuming(opt(line_ending))(bytes)?;

        Ok((bytes, Self::new(main_model, exp_model, fw_version)))
    }
}
