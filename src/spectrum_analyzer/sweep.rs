use std::fmt::Debug;
use std::ops::{Add, AddAssign};

use chrono::{DateTime, Utc};
use nom::{
    bytes::complete::tag,
    combinator::map,
    multi::length_data,
    number::complete::{be_u16, u8 as nom_u8},
};

use super::{Config, Model};
use crate::common::{parsers::*, MessageParseError, SetupInfo};

#[derive(Clone, PartialEq)]
pub enum Sweep {
    Standard(SweepDataStandard),
    Ext(SweepDataExt),
    Large(SweepDataLarge),
}

impl Sweep {
    const EEOT_BYTES: [u8; 5] = [255, 254, 255, 254, 0];

    pub fn amplitudes_dbm(&self) -> &[f32] {
        match self {
            Sweep::Standard(sweep_data) => sweep_data.amplitudes_dbm.as_slice(),
            Sweep::Ext(sweep_data) => sweep_data.amplitudes_dbm.as_slice(),
            Sweep::Large(sweep_data) => sweep_data.amplitudes_dbm.as_slice(),
        }
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Sweep::Standard(sweep_data) => sweep_data.timestamp,
            Sweep::Ext(sweep_data) => sweep_data.timestamp,
            Sweep::Large(sweep_data) => sweep_data.timestamp,
        }
    }
}

macro_rules! impl_sweep_data {
    ($sweep_data:ident, $prefix:expr, $amp_parser:expr) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $sweep_data {
            amplitudes_dbm: Vec<f32>,
            timestamp: DateTime<Utc>,
        }

        impl $sweep_data {
            pub const PREFIX: &'static [u8] = $prefix;
        }

        impl<'a> TryFrom<&'a [u8]> for $sweep_data {
            type Error = MessageParseError<'a>;

            fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
                // Parse the prefix of the message
                let (bytes, _) = tag(Self::PREFIX)(bytes)?;

                // Determine whether or not the Sweep is 'truncated' by looking for the EEOT byte
                // sequence as well as Config and SetupInfo messages
                if let Some(index) = bytes.windows(5).enumerate().find_map(|(i, window)| {
                    if Sweep::EEOT_BYTES.starts_with(window) {
                        Some(i + Sweep::EEOT_BYTES.len())
                    } else if Config::PREFIX.starts_with(window)
                        || SetupInfo::<Model>::PREFIX.starts_with(window)
                    {
                        Some(i)
                    } else {
                        None
                    }
                }) {
                    return Err(MessageParseError::Truncated {
                        remainder: bytes.get(index..),
                    });
                }

                // Get the slice containing the amplitudes in the sweep data
                let (bytes, amps) = $amp_parser(bytes)?;

                // Convert the amplitude bytes into dBm by dividing them by -2
                let amplitudes_dbm = amps.iter().map(|&byte| f32::from(byte) / -2.).collect();

                // Consume any \r or \r\n line endings and make sure there aren't any bytes left
                let _ = parse_opt_line_ending(bytes)?;

                Ok($sweep_data {
                    amplitudes_dbm,
                    timestamp: Utc::now(),
                })
            }
        }

        impl Add for $sweep_data {
            type Output = $sweep_data;

            fn add(mut self, mut rhs: Self) -> Self::Output {
                self.amplitudes_dbm.append(&mut rhs.amplitudes_dbm);
                self
            }
        }

        impl AddAssign for $sweep_data {
            fn add_assign(&mut self, mut rhs: Self) {
                self.amplitudes_dbm.append(&mut rhs.amplitudes_dbm);
            }
        }
    };
}

impl_sweep_data!(SweepDataStandard, b"$S", length_data(nom_u8));
impl_sweep_data!(
    SweepDataExt,
    b"$s",
    length_data(map(nom_u8, |len| (usize::from(len) + 1) * 16))
);
impl_sweep_data!(SweepDataLarge, b"$z", length_data(be_u16));

impl Default for Sweep {
    fn default() -> Self {
        Sweep::Standard(SweepDataStandard {
            amplitudes_dbm: Vec::default(),
            timestamp: DateTime::default(),
        })
    }
}

impl Debug for Sweep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sweep")
            .field("amplitudes", &self.amplitudes_dbm())
            .field("timestamp", &self.timestamp())
            .finish()
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
        let sweep_data = SweepDataStandard::try_from(&bytes[..]).unwrap();
        assert_eq!(
            sweep_data.amplitudes_dbm,
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
        let sweep_data = SweepDataExt::try_from(&bytes[..]).unwrap();
        assert_eq!(
            sweep_data.amplitudes_dbm,
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
        let sweep_data = SweepDataLarge::try_from(&bytes[..]).unwrap();
        assert_eq!(
            sweep_data.amplitudes_dbm,
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
        let sweep_data_error = SweepDataStandard::try_from(&bytes[..]).unwrap_err();
        assert_eq!(sweep_data_error, MessageParseError::Invalid);
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
        let sweep_data_error = SweepDataStandard::try_from(&bytes[..]).unwrap_err();
        assert_eq!(sweep_data_error, MessageParseError::Incomplete);
    }

    #[test]
    fn reject_sweep_with_eeot_bytes() {
        let length = 112;
        let bytes = [
            b'$', b'S', length, 255, 254, 255, 254, 0, 233, 246, 235, 135, 113, 130, 74, 70, 251,
            124, 186, 231, 115, 199, 203, 64, 112, 146, 24, 170, 197, 77, 105, 121, 139, 134, 91,
            157, 44, 19, 167, 140, 65, 188, 86, 28, 244, 191, 26, 164, 55, 241, 16, 5, 154, 57,
            109, 253, 211, 62, 47, 111, 152, 196, 73, 119, 178, 147, 88, 41, 250, 238, 247, 40, 97,
            230, 102, 169, 151, 249, 116, 66, 4, 80, 234, 3, 183, 71, 107, 237, 198, 175, 179, 36,
            21, 195, 243, 30, 90, 176, 37, 81, 153, 117, 51, 122, 83, 7, 189, 227, 20, 92, 6, 229,
            120, 125, 239,
        ];
        assert_eq!(
            SweepDataStandard::try_from(bytes.as_slice()).unwrap_err(),
            MessageParseError::Truncated {
                remainder: Some(&bytes[8..])
            }
        );
    }

    #[test]
    fn reject_sweep_with_config_at_the_end() {
        let bytes = [
            36, 83, 112, 215, 210, 214, 212, 212, 216, 212, 210, 214, 213, 212, 215, 212, 212, 212,
            212, 220, 211, 215, 212, 212, 217, 213, 208, 214, 216, 210, 210, 213, 215, 216, 213,
            213, 217, 209, 216, 214, 217, 206, 210, 13, 10, 35, 67, 50, 45, 77, 58, 48, 48, 54, 44,
            48, 48, 52, 44, 48, 49, 46, 49, 50, 66, 50, 48, 13, 10,
        ];
        assert_eq!(
            SweepDataStandard::try_from(bytes.as_slice()).unwrap_err(),
            MessageParseError::Truncated {
                remainder: Some(&bytes[45..])
            }
        );
    }

    #[test]
    fn add_sweeps() {
        let sweep1 = SweepDataStandard {
            amplitudes_dbm: vec![-120., -110.],
            timestamp: Utc::now(),
        };

        let sweep2 = SweepDataStandard {
            amplitudes_dbm: vec![-100., -90.],
            timestamp: Utc::now(),
        };

        let sweep3 = SweepDataStandard {
            amplitudes_dbm: vec![-80., -70.],
            timestamp: Utc::now(),
        };

        let sweep = sweep1 + sweep2 + sweep3;

        assert_eq!(
            sweep.amplitudes_dbm,
            &[-120., -110., -100., -90., -80., -70.]
        );
    }

    #[test]
    fn add_assign_sweeps() {
        let mut sweep = SweepDataStandard {
            amplitudes_dbm: vec![-120., -110.],
            timestamp: Utc::now(),
        };

        sweep += sweep.clone();

        assert_eq!(sweep.amplitudes_dbm, &[-120., -110., -120., -110.]);
    }
}
