use crate::{RfExplorerCalcMode, RfExplorerMode};
use std::{convert::TryFrom, str, str::FromStr};
use thiserror::Error;

#[derive(Debug, Copy, Clone)]
pub struct RfExplorerConfig {
    start_freq_khz: f64,
    freq_step_hz: f64,
    amp_top_dbm: i16,
    amp_bottom_dbm: i16,
    sweep_points: u32,
    expansion_module_active: bool,
    mode: RfExplorerMode,
    min_freq_khz: f64,
    max_freq_khz: f64,
    max_span_khz: f64,
    rbw_khz: Option<f64>,
    amp_offset_db: Option<i16>,
    calculator_mode: Option<RfExplorerCalcMode>,
}

#[derive(Error, Debug)]
pub enum ParseConfigError {
    #[error(transparent)]
    InvalidRfExplorerMode(#[from] <RfExplorerMode as TryFrom<u8>>::Error),

    #[error("Invalid RfExplorerConfig: expected bytes to start with #C2-F:")]
    InvalidFormat,

    #[error("A required field is missing from the bytes")]
    MissingField,

    #[error(transparent)]
    InvalidFloat(#[from] std::num::ParseFloatError),

    #[error(transparent)]
    InvalidInt(#[from] std::num::ParseIntError),

    #[error(transparent)]
    InvalidUtf8(#[from] std::str::Utf8Error),
}

fn parse_field<T>(field: Option<&[u8]>) -> Result<T, ParseConfigError>
where
    T: FromStr,
    ParseConfigError: From<T::Err>,
{
    Ok(T::from_str(
        str::from_utf8(field.ok_or_else(|| ParseConfigError::MissingField)?)?.trim(),
    )?)
}

impl RfExplorerConfig {
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

    pub fn expansion_module_active(&self) -> bool {
        self.expansion_module_active
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

impl TryFrom<&[u8]> for RfExplorerConfig {
    type Error = ParseConfigError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.starts_with(b"#C2-F:") {
            let mut fields = value
                .get(6..)
                .ok_or_else(|| ParseConfigError::MissingField)?
                .split(|&byte| byte == b',');

            // rbw_khz, amp_offset_db, and calculator_mode are optional fields that were introduced in firmware updates
            // If there's any sort of error parsing those fields, discard the error and set the field to None
            Ok(RfExplorerConfig {
                start_freq_khz: parse_field(fields.next())?,
                freq_step_hz: parse_field(fields.next())?,
                amp_top_dbm: parse_field(fields.next())?,
                amp_bottom_dbm: parse_field(fields.next())?,
                sweep_points: parse_field(fields.next())?,
                expansion_module_active: parse_field::<u8>(fields.next())? == 1u8,
                mode: RfExplorerMode::try_from(parse_field::<u8>(fields.next())?)?,
                min_freq_khz: parse_field(fields.next())?,
                max_freq_khz: parse_field(fields.next())?,
                max_span_khz: parse_field(fields.next())?,
                rbw_khz: parse_field(fields.next()).ok(),
                amp_offset_db: parse_field(fields.next()).ok(),
                calculator_mode: {
                    if let Ok(field) = parse_field::<u8>(fields.next()) {
                        RfExplorerCalcMode::try_from(field).ok()
                    } else {
                        None
                    }
                },
            })
        } else {
            Err(ParseConfigError::InvalidFormat)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_6g_combo_config() {
        let bytes =
            b"#C2-F:5249000,0196428,-030,-118,0112,0,000,4850000,6100000,0600000,00200,0000,000";
        let config = RfExplorerConfig::try_from(bytes.as_ref()).unwrap();
        assert_eq!(config.start_freq_khz(), 5_249_000f64);
        assert_eq!(config.freq_step_hz(), 196_428f64);
        assert_eq!(config.amp_top_dbm(), -30);
        assert_eq!(config.amp_bottom_dbm(), -118);
        assert_eq!(config.sweep_points(), 112);
        assert_eq!(config.expansion_module_active(), false);
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
        let config = RfExplorerConfig::try_from(bytes.as_ref()).unwrap();
        assert_eq!(config.start_freq_khz(), 96_000f64);
        assert_eq!(config.freq_step_hz(), 90072f64);
        assert_eq!(config.amp_top_dbm(), -10);
        assert_eq!(config.amp_bottom_dbm(), -120);
        assert_eq!(config.sweep_points(), 112);
        assert_eq!(config.expansion_module_active(), false);
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
        let config = RfExplorerConfig::try_from(bytes.as_ref()).unwrap();
        assert_eq!(config.rbw_khz(), None);
        assert_eq!(config.amp_offset_db(), None);
        assert_eq!(config.calculator_mode(), None);
    }

    #[test]
    fn fail_to_parse_config_with_incorrect_prefix() {
        let bytes =
            b"#D2-F:0096000,0090072,-010,-120,0112,0,000,0000050,0960000,0959950,00110,0000,000";
        assert!(RfExplorerConfig::try_from(bytes.as_ref()).is_err());
    }

    #[test]
    fn fail_to_parse_config_with_invalid_start_freq() {
        let bytes =
            b"#C2-F:XX96000,0090072,-010,-120,0112,0,000,0000050,0960000,0959950,00110,0000,000";
        assert!(RfExplorerConfig::try_from(bytes.as_ref()).is_err());
    }
}
