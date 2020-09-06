use crate::{CalculatorMode, RfExplorerMode};
use std::convert::TryFrom;
use std::str::{self, FromStr};
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
    calculator_mode: Option<CalculatorMode>,
}

#[derive(Error, Debug)]
pub enum ParseConfigError {
    #[error(transparent)]
    ConvertToModeError(#[from] <RfExplorerMode as TryFrom<u8>>::Error),

    #[error("Invalid RfExplorerConfig: expected bytes to start with #C2-F:")]
    InvalidFormatError,

    #[error("A required field is missing from the bytes")]
    MissingFieldError,

    #[error(transparent)]
    ParseFloatError(#[from] std::num::ParseFloatError),

    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),
}

impl RfExplorerConfig {
    pub fn start_freq_khz(&self) -> f64 {
        self.start_freq_khz
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

    pub fn calculator_mode(&self) -> Option<CalculatorMode> {
        self.calculator_mode
    }
}

impl TryFrom<&[u8]> for RfExplorerConfig {
    type Error = ParseConfigError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.starts_with("#C2-F:".as_bytes()) {
            let mut fields = value
                .get(6..)
                .ok_or_else(|| ParseConfigError::MissingFieldError)?
                .split(|byte| *byte == ',' as u8);

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
                calculator_mode: CalculatorMode::try_from(parse_field::<u8>(fields.next())?).ok(),
            })
        } else {
            Err(ParseConfigError::InvalidFormatError)
        }
    }
}

fn parse_field<T>(field: Option<&[u8]>) -> Result<T, ParseConfigError>
where
    T: FromStr,
    ParseConfigError: From<T::Err>,
{
    Ok(T::from_str(
        str::from_utf8(field.ok_or_else(|| ParseConfigError::MissingFieldError)?)?.trim(),
    )?)
}

#[cfg(test)]
mod tests {}
