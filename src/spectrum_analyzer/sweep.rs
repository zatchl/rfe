use chrono::{DateTime, Utc};
use std::{cmp::Ordering, convert::TryFrom};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct Sweep {
    amplitudes_dbm: Vec<f32>,
    timestamp: DateTime<Utc>,
}

#[derive(Error, Debug, Eq, PartialEq)]
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

impl Sweep {
    fn new(amp_bytes: Option<&[u8]>, expected_len: usize) -> Result<Sweep> {
        let amp_bytes = amp_bytes.ok_or_else(|| ParseSweepError::InvalidFormatError)?;
        match amp_bytes.len().cmp(&expected_len) {
            Ordering::Equal => Ok(Sweep {
                amplitudes_dbm: amplitudes_from_bytes(amp_bytes),
                timestamp: Utc::now(),
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

impl TryFrom<&[u8]> for Sweep {
    type Error = ParseSweepError;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        match bytes.get(0..2) {
            Some(b"$S") => Sweep::new(bytes.get(3..), parse_sweep_data_len(bytes)?),
            Some(b"$s") => Sweep::new(bytes.get(3..), parse_sweep_data_ext_len(bytes)?),
            Some(b"$z") => Sweep::new(bytes.get(4..), parse_sweep_data_large_len(bytes)?),
            _ => Err(ParseSweepError::InvalidFormatError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sweep() {
        let length = 112;
        let bytes = [
            b'$', b'S', length, 15, 136, 218, 52, 155, 233, 246, 235, 135, 113, 130, 74, 70, 251,
            124, 186, 231, 115, 199, 203, 64, 112, 146, 24, 170, 197, 77, 105, 121, 139, 134, 91,
            157, 44, 19, 167, 140, 65, 188, 86, 28, 244, 191, 26, 164, 55, 241, 16, 5, 154, 57,
            109, 253, 211, 62, 47, 111, 152, 196, 73, 119, 178, 147, 88, 41, 250, 238, 247, 40, 97,
            230, 102, 169, 151, 249, 116, 66, 4, 80, 234, 3, 183, 71, 107, 237, 198, 175, 179, 36,
            21, 195, 243, 30, 90, 176, 37, 81, 153, 117, 51, 122, 83, 7, 189, 227, 20, 92, 6, 229,
            120, 125, 239,
        ];
        let sweep = Sweep::try_from(&bytes[..]).unwrap();
        assert_eq!(
            sweep.amplitudes_dbm(),
            &[
                -7.5, -68.0, -109.0, -26.0, -77.5, -116.5, -123.0, -117.5, -67.5, -56.5, -65.0,
                -37.0, -35.0, -125.5, -62.0, -93.0, -115.5, -57.5, -99.5, -101.5, -32.0, -56.0,
                -73.0, -12.0, -85.0, -98.5, -38.5, -52.5, -60.5, -69.5, -67.0, -45.5, -78.5, -22.0,
                -9.5, -83.5, -70.0, -32.5, -94.0, -43.0, -14.0, -122.0, -95.5, -13.0, -82.0, -27.5,
                -120.5, -8.0, -2.5, -77.0, -28.5, -54.5, -126.5, -105.5, -31.0, -23.5, -55.5,
                -76.0, -98.0, -36.5, -59.5, -89.0, -73.5, -44.0, -20.5, -125.0, -119.0, -123.5,
                -20.0, -48.5, -115.0, -51.0, -84.5, -75.5, -124.5, -58.0, -33.0, -2.0, -40.0,
                -117.0, -1.5, -91.5, -35.5, -53.5, -118.5, -99.0, -87.5, -89.5, -18.0, -10.5,
                -97.5, -121.5, -15.0, -45.0, -88.0, -18.5, -40.5, -76.5, -58.5, -25.5, -61.0,
                -41.5, -3.5, -94.5, -113.5, -10.0, -46.0, -3.0, -114.5, -60.0, -62.5, -119.5
            ]
        );
    }

    #[test]
    fn parse_sweep_ext() {
        let length = (112 / 16) - 1;
        let bytes = [
            b'$', b's', length, 15, 136, 218, 52, 155, 233, 246, 235, 135, 113, 130, 74, 70, 251,
            124, 186, 231, 115, 199, 203, 64, 112, 146, 24, 170, 197, 77, 105, 121, 139, 134, 91,
            157, 44, 19, 167, 140, 65, 188, 86, 28, 244, 191, 26, 164, 55, 241, 16, 5, 154, 57,
            109, 253, 211, 62, 47, 111, 152, 196, 73, 119, 178, 147, 88, 41, 250, 238, 247, 40, 97,
            230, 102, 169, 151, 249, 116, 66, 4, 80, 234, 3, 183, 71, 107, 237, 198, 175, 179, 36,
            21, 195, 243, 30, 90, 176, 37, 81, 153, 117, 51, 122, 83, 7, 189, 227, 20, 92, 6, 229,
            120, 125, 239,
        ];
        let sweep = Sweep::try_from(&bytes[..]).unwrap();
        assert_eq!(
            sweep.amplitudes_dbm(),
            &[
                -7.5, -68.0, -109.0, -26.0, -77.5, -116.5, -123.0, -117.5, -67.5, -56.5, -65.0,
                -37.0, -35.0, -125.5, -62.0, -93.0, -115.5, -57.5, -99.5, -101.5, -32.0, -56.0,
                -73.0, -12.0, -85.0, -98.5, -38.5, -52.5, -60.5, -69.5, -67.0, -45.5, -78.5, -22.0,
                -9.5, -83.5, -70.0, -32.5, -94.0, -43.0, -14.0, -122.0, -95.5, -13.0, -82.0, -27.5,
                -120.5, -8.0, -2.5, -77.0, -28.5, -54.5, -126.5, -105.5, -31.0, -23.5, -55.5,
                -76.0, -98.0, -36.5, -59.5, -89.0, -73.5, -44.0, -20.5, -125.0, -119.0, -123.5,
                -20.0, -48.5, -115.0, -51.0, -84.5, -75.5, -124.5, -58.0, -33.0, -2.0, -40.0,
                -117.0, -1.5, -91.5, -35.5, -53.5, -118.5, -99.0, -87.5, -89.5, -18.0, -10.5,
                -97.5, -121.5, -15.0, -45.0, -88.0, -18.5, -40.5, -76.5, -58.5, -25.5, -61.0,
                -41.5, -3.5, -94.5, -113.5, -10.0, -46.0, -3.0, -114.5, -60.0, -62.5, -119.5
            ]
        );
    }

    #[test]
    fn parse_sweep_large() {
        let length = 112u16.to_be_bytes();
        let bytes = [
            b'$', b'z', length[0], length[1], 15, 136, 218, 52, 155, 233, 246, 235, 135, 113, 130,
            74, 70, 251, 124, 186, 231, 115, 199, 203, 64, 112, 146, 24, 170, 197, 77, 105, 121,
            139, 134, 91, 157, 44, 19, 167, 140, 65, 188, 86, 28, 244, 191, 26, 164, 55, 241, 16,
            5, 154, 57, 109, 253, 211, 62, 47, 111, 152, 196, 73, 119, 178, 147, 88, 41, 250, 238,
            247, 40, 97, 230, 102, 169, 151, 249, 116, 66, 4, 80, 234, 3, 183, 71, 107, 237, 198,
            175, 179, 36, 21, 195, 243, 30, 90, 176, 37, 81, 153, 117, 51, 122, 83, 7, 189, 227,
            20, 92, 6, 229, 120, 125, 239,
        ];
        let sweep = Sweep::try_from(&bytes[..]).unwrap();
        assert_eq!(
            sweep.amplitudes_dbm(),
            &[
                -7.5, -68.0, -109.0, -26.0, -77.5, -116.5, -123.0, -117.5, -67.5, -56.5, -65.0,
                -37.0, -35.0, -125.5, -62.0, -93.0, -115.5, -57.5, -99.5, -101.5, -32.0, -56.0,
                -73.0, -12.0, -85.0, -98.5, -38.5, -52.5, -60.5, -69.5, -67.0, -45.5, -78.5, -22.0,
                -9.5, -83.5, -70.0, -32.5, -94.0, -43.0, -14.0, -122.0, -95.5, -13.0, -82.0, -27.5,
                -120.5, -8.0, -2.5, -77.0, -28.5, -54.5, -126.5, -105.5, -31.0, -23.5, -55.5,
                -76.0, -98.0, -36.5, -59.5, -89.0, -73.5, -44.0, -20.5, -125.0, -119.0, -123.5,
                -20.0, -48.5, -115.0, -51.0, -84.5, -75.5, -124.5, -58.0, -33.0, -2.0, -40.0,
                -117.0, -1.5, -91.5, -35.5, -53.5, -118.5, -99.0, -87.5, -89.5, -18.0, -10.5,
                -97.5, -121.5, -15.0, -45.0, -88.0, -18.5, -40.5, -76.5, -58.5, -25.5, -61.0,
                -41.5, -3.5, -94.5, -113.5, -10.0, -46.0, -3.0, -114.5, -60.0, -62.5, -119.5
            ]
        );
    }

    #[test]
    fn reject_sweep_with_too_many_amplitudes() {
        let length = 112;
        let bytes = [
            b'$', b'S', length, 15, 136, 218, 52, 155, 233, 246, 235, 135, 113, 130, 74, 70, 251,
            124, 186, 231, 115, 199, 203, 64, 112, 146, 24, 170, 197, 77, 105, 121, 139, 134, 91,
            157, 44, 19, 167, 140, 65, 188, 86, 28, 244, 191, 26, 164, 55, 241, 16, 5, 154, 57,
            109, 253, 211, 62, 47, 111, 152, 196, 73, 119, 178, 147, 88, 41, 250, 238, 247, 40, 97,
            230, 102, 169, 151, 249, 116, 66, 4, 80, 234, 3, 183, 71, 107, 237, 198, 175, 179, 36,
            21, 195, 243, 30, 90, 176, 37, 81, 153, 117, 51, 122, 83, 7, 189, 227, 20, 92, 6, 229,
            120, 125, 239, 100,
        ];
        let sweep = Sweep::try_from(&bytes[..]);
        assert!(matches!(sweep.unwrap_err(), ParseSweepError::TooManyAmplitudes{..}));
    }

    #[test]
    fn reject_sweep_with_too_few_amplitudes() {
        let length = 112;
        let bytes = [
            b'$', b'S', length, 15, 136, 218, 52, 155, 233, 246, 235, 135, 113, 130, 74, 70, 251,
            124, 186, 231, 115, 199, 203, 64, 112, 146, 24, 170, 197, 77, 105, 121, 139, 134, 91,
            157, 44, 19, 167, 140, 65, 188, 86, 28, 244, 191, 26, 164, 55, 241, 16, 5, 154, 57,
            109, 253, 211, 62, 47, 111, 152, 196, 73, 119, 178, 147, 88, 41, 250, 238, 247, 40, 97,
            230, 102, 169, 151, 249, 116, 66, 4, 80, 234, 3, 183, 71, 107, 237, 198, 175, 179, 36,
            21, 195, 243, 30, 90, 176, 37, 81, 153, 117, 51, 122, 83, 7, 189, 227, 20, 92, 6, 229,
            120, 125,
        ];
        let sweep_error = Sweep::try_from(&bytes[..]).unwrap_err();
        assert!(matches!(sweep_error, ParseSweepError::TooFewAmplitudes{..}));
    }
}
