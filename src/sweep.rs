use std::{cmp::Ordering, convert::TryFrom};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct RfExplorerSweep {
    amplitudes_dbm: Vec<f32>,
}

#[derive(Error, Debug)]
pub enum ParseSweepError {
    #[error("Invalid RfExplorerSweep")]
    InvalidFormatError,

    #[error("Fewer amplitudes than expected. Expected {expected} but received {actual}.")]
    TooFewAmplitudes { expected: usize, actual: usize },

    #[error("More amplitudes than expected. Expected {expected} but received {actual}.")]
    TooManyAmplitudes { expected: usize, actual: usize },
}

type Result<T> = std::result::Result<T, ParseSweepError>;

fn amplitudes_from_bytes(bytes: &[u8]) -> Vec<f32> {
    // Divide each byte by -2 to get each amplitude in dBm
    bytes.iter().map(|&byte| f32::from(byte) / -2.0).collect()
}

fn parse_sweep_data_len(bytes: &[u8]) -> Result<usize> {
    Ok(usize::from(
        *bytes
            .get(2)
            .ok_or_else(|| ParseSweepError::InvalidFormatError)?,
    ))
}

fn parse_sweep_data_ext_len(bytes: &[u8]) -> Result<usize> {
    Ok((usize::from(
        *bytes
            .get(2)
            .ok_or_else(|| ParseSweepError::InvalidFormatError)?,
    ) + 1)
        * 16)
}

fn parse_sweep_data_large_len(bytes: &[u8]) -> Result<usize> {
    Ok(usize::from(u16::from_be_bytes([
        *bytes
            .get(2)
            .ok_or_else(|| ParseSweepError::InvalidFormatError)?,
        *bytes
            .get(3)
            .ok_or_else(|| ParseSweepError::InvalidFormatError)?,
    ])))
}

impl RfExplorerSweep {
    fn new(amp_bytes: Option<&[u8]>, expected_len: usize) -> Result<RfExplorerSweep> {
        let amp_bytes = amp_bytes.ok_or_else(|| ParseSweepError::InvalidFormatError)?;
        match amp_bytes.len().cmp(&expected_len) {
            Ordering::Equal => Ok(RfExplorerSweep {
                amplitudes_dbm: amplitudes_from_bytes(amp_bytes),
            }),
            Ordering::Less => Err(ParseSweepError::TooFewAmplitudes {
                expected: expected_len,
                actual: amp_bytes.len(),
            }),
            Ordering::Greater => Err(ParseSweepError::TooManyAmplitudes {
                expected: expected_len,
                actual: amp_bytes.len(),
            }),
        }
    }

    pub fn amplitudes_dbm(&self) -> &[f32] {
        &self.amplitudes_dbm
    }
}

impl TryFrom<&[u8]> for RfExplorerSweep {
    type Error = ParseSweepError;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        match bytes.get(0..2) {
            Some(b"$S") => RfExplorerSweep::new(bytes.get(3..), parse_sweep_data_len(bytes)?),
            Some(b"$s") => RfExplorerSweep::new(bytes.get(3..), parse_sweep_data_ext_len(bytes)?),
            Some(b"$z") => RfExplorerSweep::new(bytes.get(4..), parse_sweep_data_large_len(bytes)?),
            _ => Err(ParseSweepError::InvalidFormatError),
        }
    }
}
