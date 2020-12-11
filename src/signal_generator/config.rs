use crate::{
    rf_explorer::{Message, ParseFromBytes},
    signal_generator::{Attenuation, PowerLevel, RfPower},
};
use nom::{
    bytes::complete::{tag, take},
    character::complete::line_ending,
    combinator::{all_consuming, map_res, opt},
    number::complete::u8 as nom_u8,
    IResult,
};
use std::{
    convert::TryFrom,
    str::{self, FromStr},
};

#[derive(Debug, Copy, Clone)]
pub struct Config {
    start_freq_khz: f64,
    cw_freq_khz: f64,
    total_steps: u32,
    freq_step_khz: f64,
    attenuation: Attenuation,
    power_level: PowerLevel,
    sweep_power_steps: u16,
    start_attenuation: Attenuation,
    start_power_level: PowerLevel,
    stop_attenuation: Attenuation,
    stop_power_level: PowerLevel,
    rf_power: RfPower,
    sweep_delay_ms: u16,
}

impl Config {
    pub fn start_freq_khz(&self) -> f64 {
        self.start_freq_khz
    }

    pub fn cw_freq_khz(&self) -> f64 {
        self.cw_freq_khz
    }

    pub fn total_steps(&self) -> u32 {
        self.total_steps
    }

    pub fn freq_step_khz(&self) -> f64 {
        self.freq_step_khz
    }

    pub fn attenuation(&self) -> Attenuation {
        self.attenuation
    }

    pub fn power_level(&self) -> PowerLevel {
        self.power_level
    }

    pub fn sweep_power_steps(&self) -> u16 {
        self.sweep_power_steps
    }

    pub fn start_attenuation(&self) -> Attenuation {
        self.start_attenuation
    }

    pub fn start_power_level(&self) -> PowerLevel {
        self.start_power_level
    }

    pub fn stop_attenuation(&self) -> Attenuation {
        self.stop_attenuation
    }

    pub fn stop_power_level(&self) -> PowerLevel {
        self.stop_power_level
    }

    pub fn rf_power(&self) -> RfPower {
        self.rf_power
    }

    pub fn sweep_delay_ms(&self) -> u16 {
        self.sweep_delay_ms
    }
}

impl Message for Config {
    const PREFIX: &'static [u8] = b"#C3-*:";
}

impl ParseFromBytes for Config {
    fn parse_from_bytes(bytes: &[u8]) -> IResult<&[u8], Self> {
        // Parse the prefix of the message
        let (bytes, _) = tag(Config::PREFIX)(bytes)?;

        // Parse the start frequency
        let (bytes, start_freq_khz) =
            map_res(map_res(take(7u8), str::from_utf8), str::parse)(bytes)?;

        let (bytes, _) = tag(",")(bytes)?;

        // Parse the cw frequency
        let (bytes, cw_freq_khz) = map_res(map_res(take(7u8), str::from_utf8), str::parse)(bytes)?;

        let (bytes, _) = tag(",")(bytes)?;

        // Parse the total steps
        let (bytes, total_steps) =
            map_res(map_res(take(4u8), str::from_utf8), FromStr::from_str)(bytes)?;

        let (bytes, _) = tag(",")(bytes)?;

        // Parse the step frequency
        let (bytes, freq_step_khz) =
            map_res(map_res(take(7u8), str::from_utf8), str::parse)(bytes)?;

        let (bytes, _) = tag(",")(bytes)?;

        // Parse the attenuation
        let (bytes, attenuation) = map_res(nom_u8, TryFrom::try_from)(bytes)?;

        let (bytes, _) = tag(",")(bytes)?;

        // Parse the power level
        let (bytes, power_level) = map_res(nom_u8, TryFrom::try_from)(bytes)?;

        let (bytes, _) = tag(",")(bytes)?;

        // Parse the sweep power steps
        let (bytes, sweep_power_steps) =
            map_res(map_res(take(4u8), str::from_utf8), FromStr::from_str)(bytes)?;

        let (bytes, _) = tag(",")(bytes)?;

        // Parse the start attenuation
        let (bytes, start_attenuation) = map_res(nom_u8, TryFrom::try_from)(bytes)?;

        let (bytes, _) = tag(",")(bytes)?;

        // Parse the start power level
        let (bytes, start_power_level) = map_res(nom_u8, TryFrom::try_from)(bytes)?;

        let (bytes, _) = tag(",")(bytes)?;

        // Parse the stop attenuation
        let (bytes, stop_attenuation) = map_res(nom_u8, TryFrom::try_from)(bytes)?;

        let (bytes, _) = tag(",")(bytes)?;

        // Parse the stop power level
        let (bytes, stop_power_level) = map_res(nom_u8, TryFrom::try_from)(bytes)?;

        let (bytes, _) = tag(",")(bytes)?;

        // Parse the rf power
        let (bytes, rf_power) = map_res(nom_u8, TryFrom::try_from)(bytes)?;

        let (bytes, _) = tag(",")(bytes)?;

        // Parse the sweep delay
        let (bytes, sweep_delay_ms) =
            map_res(map_res(take(5u8), str::from_utf8), FromStr::from_str)(bytes)?;

        // Consume any \r or \r\n line endings and make sure there aren't any bytes left
        let (bytes, _) = all_consuming(opt(line_ending))(bytes)?;

        Ok((
            bytes,
            Config {
                start_freq_khz,
                cw_freq_khz,
                total_steps,
                freq_step_khz,
                attenuation,
                power_level,
                sweep_power_steps,
                start_attenuation,
                start_power_level,
                stop_attenuation,
                stop_power_level,
                rf_power,
                sweep_delay_ms,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_config() {
        let bytes = b"#C3-*:0510000,0186525,0005,0001000,0,3,0000,0,0,1,3,0,00100\r\n";
        let config = Config::parse_from_bytes(bytes.as_ref()).unwrap().1;
        assert_eq!(config.start_freq_khz(), 510_000.);
        assert_eq!(config.cw_freq_khz(), 186_525.);
        assert_eq!(config.total_steps(), 5);
        assert_eq!(config.freq_step_khz(), 1000.);
        assert_eq!(config.attenuation(), Attenuation::On);
        assert_eq!(config.power_level(), PowerLevel::Highest);
        assert_eq!(config.sweep_power_steps(), 0);
        assert_eq!(config.start_attenuation(), Attenuation::On);
        assert_eq!(config.start_power_level(), PowerLevel::Lowest);
        assert_eq!(config.stop_attenuation(), Attenuation::Off);
        assert_eq!(config.stop_power_level(), PowerLevel::Highest);
        assert_eq!(config.rf_power(), RfPower::On);
        assert_eq!(config.sweep_delay_ms(), 100);
    }
}
