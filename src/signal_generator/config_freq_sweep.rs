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
pub struct ConfigFreqSweep {
    start_freq_khz: f64,
    total_steps: u32,
    step_freq_khz: f64,
    attenuation: Attenuation,
    power_level: PowerLevel,
    rf_power: RfPower,
    sweep_delay_ms: u16,
}

impl ConfigFreqSweep {
    pub fn start_freq_khz(&self) -> f64 {
        self.start_freq_khz
    }

    pub fn total_steps(&self) -> u32 {
        self.total_steps
    }

    pub fn step_freq_khz(&self) -> f64 {
        self.step_freq_khz
    }

    pub fn attenuation(&self) -> Attenuation {
        self.attenuation
    }

    pub fn power_level(&self) -> PowerLevel {
        self.power_level
    }

    pub fn rf_power(&self) -> RfPower {
        self.rf_power
    }

    pub fn sweep_delay_ms(&self) -> u16 {
        self.sweep_delay_ms
    }
}

impl Message for ConfigFreqSweep {
    const PREFIX: &'static [u8] = b"#C3-F:";
}

impl ParseFromBytes for ConfigFreqSweep {
    fn parse_from_bytes(bytes: &[u8]) -> IResult<&[u8], Self> {
        // Parse the prefix of the message
        let (bytes, _) = tag(Self::PREFIX)(bytes)?;

        // Parse the start frequency
        let (bytes, start_freq_khz) =
            map_res(map_res(take(7u8), str::from_utf8), str::parse)(bytes)?;

        let (bytes, _) = tag(",")(bytes)?;

        // Parse the total steps
        let (bytes, total_steps) =
            map_res(map_res(take(4u8), str::from_utf8), FromStr::from_str)(bytes)?;

        let (bytes, _) = tag(",")(bytes)?;

        // Parse the step frequency
        let (bytes, step_freq_khz) =
            map_res(map_res(take(7u8), str::from_utf8), str::parse)(bytes)?;

        let (bytes, _) = tag(",")(bytes)?;

        // Parse the attenuation
        let (bytes, attenuation) = map_res(nom_u8, TryFrom::try_from)(bytes)?;

        let (bytes, _) = tag(",")(bytes)?;

        // Parse the power level
        let (bytes, power_level) = map_res(nom_u8, TryFrom::try_from)(bytes)?;

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
            ConfigFreqSweep {
                start_freq_khz,
                total_steps,
                step_freq_khz,
                attenuation,
                power_level,
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
    fn parse_config_freq_sweep() {
        let bytes = b"#C3-F:0186525,0005,0001000,0,3,0,00100";
        let config_freq_sweep = ConfigFreqSweep::parse_from_bytes(bytes.as_ref()).unwrap().1;
        assert_eq!(config_freq_sweep.start_freq_khz(), 186_525.);
        assert_eq!(config_freq_sweep.total_steps(), 5);
        assert_eq!(config_freq_sweep.step_freq_khz(), 1000.);
        assert_eq!(config_freq_sweep.attenuation(), Attenuation::On);
        assert_eq!(config_freq_sweep.power_level(), PowerLevel::Highest);
        assert_eq!(config_freq_sweep.rf_power(), RfPower::On);
        assert_eq!(config_freq_sweep.sweep_delay_ms(), 100);
    }
}
