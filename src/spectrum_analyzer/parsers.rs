use super::{CalcMode, Mode, RadioModule};
use crate::rf_explorer::parsers::*;
use nom::{combinator::map_res, IResult};
use std::convert::TryFrom;

pub(super) fn parse_amplitude(bytes: &[u8]) -> IResult<&[u8], i16> {
    parse_num(4u8)(bytes)
}

pub(super) fn parse_calc_mode(bytes: &[u8]) -> IResult<&[u8], CalcMode> {
    map_res(parse_num::<u8>(3u8), CalcMode::try_from)(bytes)
}

pub(super) fn parse_mode(bytes: &[u8]) -> IResult<&[u8], Mode> {
    map_res(parse_num::<u8>(3u8), Mode::try_from)(bytes)
}

pub(super) fn parse_radio_module(bytes: &[u8]) -> IResult<&[u8], RadioModule> {
    map_res(parse_num::<u8>(1u8), RadioModule::try_from)(bytes)
}
