use super::{Attenuation, PowerLevel, RfPower};
use crate::rf_explorer::parsers::*;
use nom::{combinator::map_res, IResult};
use std::convert::TryFrom;

pub(super) fn parse_attenuation(bytes: &[u8]) -> IResult<&[u8], Attenuation> {
    map_res(parse_num::<u8>(1u8), Attenuation::try_from)(bytes)
}

pub(super) fn parse_power_level(bytes: &[u8]) -> IResult<&[u8], PowerLevel> {
    map_res(parse_num::<u8>(1u8), PowerLevel::try_from)(bytes)
}

pub(super) fn parse_sweep_delay_ms(bytes: &[u8]) -> IResult<&[u8], u16> {
    parse_num(5u8)(bytes)
}

pub(super) fn parse_rf_power(bytes: &[u8]) -> IResult<&[u8], RfPower> {
    map_res(parse_num::<u8>(1u8), RfPower::try_from)(bytes)
}
