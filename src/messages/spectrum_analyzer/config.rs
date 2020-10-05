use crate::messages::ParseMessageError;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use rfe_message::RfeMessage;
use std::convert::TryFrom;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, RfeMessage)]
#[prefix = "#C2-F:"]
pub struct Config {
    start_freq_khz: f64,
    freq_step_hz: f64,
    amp_top_dbm: i16,
    amp_bottom_dbm: i16,
    sweep_points: u32,
    active_module: RfExplorerActiveModule,
    mode: RfExplorerMode,
    min_freq_khz: f64,
    max_freq_khz: f64,
    max_span_khz: f64,
    #[optional]
    rbw_khz: Option<f64>,
    #[optional]
    amp_offset_db: Option<i16>,
    #[optional]
    calculator_mode: Option<RfExplorerCalcMode>,
}

#[derive(Debug, Copy, Clone, TryFromPrimitive, Eq, PartialEq)]
#[repr(u8)]
pub enum RfExplorerActiveModule {
    Main = 0,
    Expansion,
}

#[derive(Debug, Copy, Clone, TryFromPrimitive, Eq, PartialEq)]
#[repr(u8)]
pub enum RfExplorerMode {
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
pub enum RfExplorerCalcMode {
    Normal = 0,
    Max,
    Avg,
    Overwrite,
    MaxHold,
}

impl Config {
    pub fn start_freq_khz(&self) -> f64 {
        self.start_freq_khz
    }

    pub fn end_freq_khz(&self) -> f64 {
        self.start_freq_khz + f64::from(self.sweep_points - 1) * (self.freq_step_hz / 1000f64)
    }

    pub fn freq_step_hz(&self) -> f64 {
        self.freq_step_hz
    }

    pub fn amp_top_dbm(&self) -> i16 {
        self.amp_top_dbm
    }

    pub fn amp_bottom_dbm(&self) -> i16 {
        self.amp_bottom_dbm
    }

    pub fn sweep_points(&self) -> u32 {
        self.sweep_points
    }

    pub fn active_module(&self) -> RfExplorerActiveModule {
        self.active_module
    }

    pub fn mode(&self) -> RfExplorerMode {
        self.mode
    }

    pub fn min_freq_khz(&self) -> f64 {
        self.min_freq_khz
    }

    pub fn max_freq_khz(&self) -> f64 {
        self.max_freq_khz
    }

    pub fn max_span_khz(&self) -> f64 {
        self.max_span_khz
    }

    pub fn rbw_khz(&self) -> Option<f64> {
        self.rbw_khz
    }

    pub fn amp_offset_db(&self) -> Option<i16> {
        self.amp_offset_db
    }

    pub fn calculator_mode(&self) -> Option<RfExplorerCalcMode> {
        self.calculator_mode
    }
}

impl FromStr for RfExplorerActiveModule {
    type Err = ParseMessageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(u8::from_str(s)?).map_err(|_| ParseMessageError::InvalidData)
    }
}

impl FromStr for RfExplorerMode {
    type Err = ParseMessageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(u8::from_str(s)?).map_err(|_| ParseMessageError::InvalidData)
    }
}

impl FromStr for RfExplorerCalcMode {
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
        assert_eq!(config.start_freq_khz(), 5_249_000f64);
        assert_eq!(config.freq_step_hz(), 196_428f64);
        assert_eq!(config.amp_top_dbm(), -30);
        assert_eq!(config.amp_bottom_dbm(), -118);
        assert_eq!(config.sweep_points(), 112);
        assert_eq!(config.active_module(), RfExplorerActiveModule::Main);
        assert_eq!(config.mode(), RfExplorerMode::SpectrumAnalyzer);
        assert_eq!(config.min_freq_khz(), 4_850_000f64);
        assert_eq!(config.max_freq_khz(), 6_100_000f64);
        assert_eq!(config.max_span_khz(), 600_000f64);
        assert_eq!(config.rbw_khz(), Some(200f64));
        assert_eq!(config.amp_offset_db(), Some(0));
        assert_eq!(config.calculator_mode(), Some(RfExplorerCalcMode::Normal));
    }

    #[test]
    fn parse_wsub1g_plus_config() {
        let bytes =
            b"#C2-F:0096000,0090072,-010,-120,0112,0,000,0000050,0960000,0959950,00110,0000,000";
        let config = Config::try_from(bytes.as_ref()).unwrap();
        assert_eq!(config.start_freq_khz(), 96_000f64);
        assert_eq!(config.freq_step_hz(), 90072f64);
        assert_eq!(config.amp_top_dbm(), -10);
        assert_eq!(config.amp_bottom_dbm(), -120);
        assert_eq!(config.sweep_points(), 112);
        assert_eq!(config.active_module(), RfExplorerActiveModule::Main);
        assert_eq!(config.mode(), RfExplorerMode::SpectrumAnalyzer);
        assert_eq!(config.min_freq_khz(), 50f64);
        assert_eq!(config.max_freq_khz(), 960000f64);
        assert_eq!(config.max_span_khz(), 959950f64);
        assert_eq!(config.rbw_khz(), Some(110f64));
        assert_eq!(config.amp_offset_db(), Some(0));
        assert_eq!(config.calculator_mode(), Some(RfExplorerCalcMode::Normal));
    }

    #[test]
    fn parse_config_without_rbw_amp_offset_calc_mode() {
        let bytes = b"#C2-F:5249000,0196428,-030,-118,0112,0,000,4850000,6100000,0600000";
        let config = Config::try_from(bytes.as_ref()).unwrap();
        assert_eq!(config.rbw_khz(), None);
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
