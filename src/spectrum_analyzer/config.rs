use crate::rf_explorer::ParseMessageError;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use rfe_message::Message;
use std::convert::TryFrom;
use std::str::FromStr;
use uom::si::{
    f64::Frequency,
    frequency::{hertz, kilohertz},
};

#[derive(Debug, Copy, Clone, PartialEq, Message)]
#[prefix = "#C2-F:"]
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
    #[optional]
    rbw_khz: Option<f64>,
    #[optional]
    amp_offset_db: Option<i16>,
    #[optional]
    calculator_mode: Option<CalcMode>,
}

#[derive(Debug, Copy, Clone, TryFromPrimitive, Eq, PartialEq)]
#[repr(u8)]
pub enum RadioModule {
    Main = 0,
    Expansion,
}

#[derive(Debug, Copy, Clone, TryFromPrimitive, Eq, PartialEq)]
#[repr(u8)]
pub enum Mode {
    SpectrumAnalyzer = 0,
    RfGenerator = 1,
    WifiAnalyzer = 2,
    AnalyzerTracking = 5,
    RfSniffer = 6,
    CwTransmitter = 60,
    SweepFrequency = 61,
    SweepAmplitude = 62,
    GeneratorTracking = 63,
    Unknown = 255,
}

#[derive(Debug, Copy, Clone, TryFromPrimitive, IntoPrimitive, Eq, PartialEq)]
#[repr(u8)]
pub enum CalcMode {
    Normal = 0,
    Max,
    Avg,
    Overwrite,
    MaxHold,
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

impl FromStr for RadioModule {
    type Err = ParseMessageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(u8::from_str(s)?).map_err(|_| ParseMessageError::InvalidData)
    }
}

impl FromStr for Mode {
    type Err = ParseMessageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(u8::from_str(s)?).map_err(|_| ParseMessageError::InvalidData)
    }
}

impl FromStr for CalcMode {
    type Err = ParseMessageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(u8::from_str(s)?).map_err(|_| ParseMessageError::InvalidData)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_6g_combo_config() {
        let bytes =
            b"#C2-F:5249000,0196428,-030,-118,0112,0,000,4850000,6100000,0600000,00200,0000,000";
        let config = Config::try_from(bytes.as_ref()).unwrap();
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
        let config = Config::try_from(bytes.as_ref()).unwrap();
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
        let config = Config::try_from(bytes.as_ref()).unwrap();
        assert_eq!(config.rbw(), None);
        assert_eq!(config.amp_offset_db(), None);
        assert_eq!(config.calculator_mode(), None);
    }

    #[test]
    fn fail_to_parse_config_with_incorrect_prefix() {
        let bytes =
            b"#D2-F:0096000,0090072,-010,-120,0112,0,000,0000050,0960000,0959950,00110,0000,000";
        assert!(Config::try_from(bytes.as_ref()).is_err());
    }

    #[test]
    fn fail_to_parse_config_with_invalid_start_freq() {
        let bytes =
            b"#C2-F:XX96000,0090072,-010,-120,0112,0,000,0000050,0960000,0959950,00110,0000,000";
        assert!(Config::try_from(bytes.as_ref()).is_err());
    }
}
