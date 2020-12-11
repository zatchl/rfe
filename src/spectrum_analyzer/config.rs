use crate::{
    rf_explorer::{Message, ParseFromBytes},
    spectrum_analyzer::{CalcMode, Mode, RadioModule},
};
use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    character::complete::line_ending,
    combinator::{all_consuming, map_res, opt},
    IResult,
};
use std::str::{self, FromStr};
use uom::si::{
    f64::Frequency,
    frequency::{hertz, kilohertz},
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Config {
    start_freq_khz: f64,
    step_freq_hz: f64,
    max_amp_dbm: i16,
    min_amp_dbm: i16,
    sweep_points: u32,
    active_radio_module: RadioModule,
    mode: Mode,
    min_freq_khz: f64,
    max_freq_khz: f64,
    max_span_khz: f64,
    rbw_khz: Option<f64>,
    amp_offset_db: Option<i16>,
    calculator_mode: Option<CalcMode>,
}

impl Config {
    pub fn start_freq(&self) -> Frequency {
        Frequency::new::<kilohertz>(self.start_freq_khz)
    }

    pub fn stop_freq(&self) -> Frequency {
        self.start_freq() + self.step_freq() * f64::from(self.sweep_points - 1)
    }

    pub fn step_freq(&self) -> Frequency {
        Frequency::new::<hertz>(self.step_freq_hz)
    }

    pub fn min_amp_dbm(&self) -> i16 {
        self.min_amp_dbm
    }

    pub fn max_amp_dbm(&self) -> i16 {
        self.max_amp_dbm
    }

    pub fn sweep_points(&self) -> u32 {
        self.sweep_points
    }

    pub fn active_radio_module(&self) -> RadioModule {
        self.active_radio_module
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn min_freq(&self) -> Frequency {
        Frequency::new::<kilohertz>(self.min_freq_khz)
    }

    pub fn max_freq(&self) -> Frequency {
        Frequency::new::<kilohertz>(self.max_freq_khz)
    }

    pub fn max_span(&self) -> Frequency {
        Frequency::new::<kilohertz>(self.max_span_khz)
    }

    pub fn rbw(&self) -> Option<Frequency> {
        self.rbw_khz
            .map(|rbw_khz| Frequency::new::<kilohertz>(rbw_khz))
    }

    pub fn amp_offset_db(&self) -> Option<i16> {
        self.amp_offset_db
    }

    pub fn calculator_mode(&self) -> Option<CalcMode> {
        self.calculator_mode
    }
}

impl Message for Config {
    const PREFIX: &'static [u8] = b"#C2-F:";
}

impl ParseFromBytes for Config {
    fn parse_from_bytes(bytes: &[u8]) -> IResult<&[u8], Self> {
        // Parse the prefix of the message
        let (bytes, _) = tag(Config::PREFIX)(bytes)?;

        // Parse the start frequency
        let (bytes, start_freq_khz) =
            map_res(map_res(take(7u8), str::from_utf8), str::parse)(bytes)?;

        let (bytes, _) = tag(",")(bytes)?;

        // Parse the stop frequency
        let (bytes, step_freq_hz) = map_res(map_res(take(7u8), str::from_utf8), str::parse)(bytes)?;

        let (bytes, _) = tag(",")(bytes)?;

        // Parse the max amplitude
        let (bytes, max_amp_dbm) =
            map_res(map_res(take(4u8), str::from_utf8), FromStr::from_str)(bytes)?;

        let (bytes, _) = tag(",")(bytes)?;

        // Parse the min amplitude
        let (bytes, min_amp_dbm) =
            map_res(map_res(take(4u8), str::from_utf8), FromStr::from_str)(bytes)?;

        let (bytes, _) = tag(",")(bytes)?;

        // Parse the number of points in a sweep
        // 0-9999 uses 4 bytes and 10000+ uses 5 bytes
        // Try to parse using 5 bytes first and if that doesn't work fall back to 4 bytes
        let (bytes, sweep_points) = alt((
            map_res(map_res(take(5u8), str::from_utf8), FromStr::from_str),
            map_res(map_res(take(4u8), str::from_utf8), FromStr::from_str),
        ))(bytes)?;

        let (bytes, _) = tag(",")(bytes)?;

        // Parse the active radio module
        let (bytes, active_radio_module) =
            map_res(map_res(take(1u8), str::from_utf8), str::parse)(bytes)?;

        let (bytes, _) = tag(",")(bytes)?;

        // Parse the mode
        let (bytes, mode) = map_res(map_res(take(3u8), str::from_utf8), str::parse)(bytes)?;

        let (bytes, _) = tag(",")(bytes)?;

        // Parse the minimum frequency
        let (bytes, min_freq_khz) = map_res(map_res(take(7u8), str::from_utf8), str::parse)(bytes)?;

        let (bytes, _) = tag(",")(bytes)?;

        // Parse the maximum frequency
        let (bytes, max_freq_khz) = map_res(map_res(take(7u8), str::from_utf8), str::parse)(bytes)?;

        let (bytes, _) = tag(",")(bytes)?;

        // Parse the maximum span
        let (bytes, max_span_khz) = map_res(map_res(take(7u8), str::from_utf8), str::parse)(bytes)?;

        let (bytes, _) = opt(tag(","))(bytes)?;

        // Parse the RBW
        // This field is optional because it's not sent by older RF Explorers
        let (bytes, rbw_khz) = opt(map_res(map_res(take(5u8), str::from_utf8), str::parse))(bytes)?;

        let (bytes, _) = opt(tag(","))(bytes)?;

        // Parse the amplitude offset
        // This field is optional because it's not sent by older RF Explorers
        let (bytes, amp_offset_db) = opt(map_res(
            map_res(take(4u8), str::from_utf8),
            FromStr::from_str,
        ))(bytes)?;

        let (bytes, _) = opt(tag(","))(bytes)?;

        // Parse the calculator mode
        // This field is optional because it's not sent by older RF Explorers
        let (bytes, calculator_mode) = opt(map_res(
            map_res(take(3u8), str::from_utf8),
            FromStr::from_str,
        ))(bytes)?;

        // Consume \n or \r\n line endings and make sure there aren't any bytes left afterwards
        let (bytes, _) = all_consuming(opt(line_ending))(bytes)?;

        Ok((
            bytes,
            Config {
                start_freq_khz,
                step_freq_hz,
                max_amp_dbm,
                min_amp_dbm,
                sweep_points,
                active_radio_module,
                mode,
                min_freq_khz,
                max_freq_khz,
                max_span_khz,
                rbw_khz,
                amp_offset_db,
                calculator_mode,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_6g_combo_config() {
        let bytes =
            b"#C2-F:5249000,0196428,-030,-118,0112,0,000,4850000,6100000,0600000,00200,0000,000";
        let config = Config::parse_from_bytes(bytes.as_ref()).unwrap().1;
        assert_eq!(config.start_freq(), Frequency::new::<kilohertz>(5_249_000.));
        assert_eq!(config.step_freq(), Frequency::new::<hertz>(196_428.));
        assert_eq!(config.max_amp_dbm(), -30);
        assert_eq!(config.min_amp_dbm(), -118);
        assert_eq!(config.sweep_points(), 112);
        assert_eq!(config.active_radio_module(), RadioModule::Main);
        assert_eq!(config.mode(), Mode::SpectrumAnalyzer);
        assert_eq!(config.min_freq(), Frequency::new::<kilohertz>(4_850_000.));
        assert_eq!(config.max_freq(), Frequency::new::<kilohertz>(6_100_000.));
        assert_eq!(config.max_span(), Frequency::new::<kilohertz>(600_000.));
        assert_eq!(config.rbw(), Some(Frequency::new::<kilohertz>(200.)));
        assert_eq!(config.amp_offset_db(), Some(0));
        assert_eq!(config.calculator_mode(), Some(CalcMode::Normal));
    }

    #[test]
    fn parse_wsub1g_plus_config() {
        let bytes =
            b"#C2-F:0096000,0090072,-010,-120,0112,0,000,0000050,0960000,0959950,00110,0000,000";
        let config = Config::parse_from_bytes(bytes.as_ref()).unwrap().1;
        assert_eq!(config.start_freq(), Frequency::new::<kilohertz>(96_000.));
        assert_eq!(config.step_freq(), Frequency::new::<hertz>(90072.));
        assert_eq!(config.max_amp_dbm(), -10);
        assert_eq!(config.min_amp_dbm(), -120);
        assert_eq!(config.sweep_points(), 112);
        assert_eq!(config.active_radio_module(), RadioModule::Main);
        assert_eq!(config.mode(), Mode::SpectrumAnalyzer);
        assert_eq!(config.min_freq(), Frequency::new::<kilohertz>(50.));
        assert_eq!(config.max_freq(), Frequency::new::<kilohertz>(960000.));
        assert_eq!(config.max_span(), Frequency::new::<kilohertz>(959950.));
        assert_eq!(config.rbw(), Some(Frequency::new::<kilohertz>(110.)));
        assert_eq!(config.amp_offset_db(), Some(0));
        assert_eq!(config.calculator_mode(), Some(CalcMode::Normal));
    }

    #[test]
    fn parse_config_without_rbw_amp_offset_calc_mode() {
        let bytes = b"#C2-F:5249000,0196428,-030,-118,0112,0,000,4850000,6100000,0600000";
        let config = Config::parse_from_bytes(bytes.as_ref()).unwrap().1;
        assert_eq!(config.rbw(), None);
        assert_eq!(config.amp_offset_db(), None);
        assert_eq!(config.calculator_mode(), None);
    }

    #[test]
    fn fail_to_parse_config_with_incorrect_prefix() {
        let bytes =
            b"#D2-F:0096000,0090072,-010,-120,0112,0,000,0000050,0960000,0959950,00110,0000,000";
        assert!(Config::parse_from_bytes(bytes.as_ref()).is_err());
    }

    #[test]
    fn fail_to_parse_config_with_invalid_start_freq() {
        let bytes =
            b"#C2-F:XX96000,0090072,-010,-120,0112,0,000,0000050,0960000,0959950,00110,0000,000";
        assert!(Config::parse_from_bytes(bytes.as_ref()).is_err());
    }
}
