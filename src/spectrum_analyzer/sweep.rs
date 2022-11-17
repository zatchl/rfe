use crate::rf_explorer::{Message, ParseFromBytes};
use chrono::{DateTime, Utc};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::line_ending,
    combinator::{all_consuming, map, opt, peek},
    error::{Error, ErrorKind},
    multi::length_data,
    number::complete::{be_u16, u8 as nom_u8},
    IResult,
};
use std::ops::{Add, AddAssign};

#[derive(Debug, Clone, PartialEq)]
pub enum Sweep {
    Standard(SweepDataStandard),
    Ext(SweepDataExt),
    Large(SweepDataLarge),
}

impl Sweep {
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

impl ParseFromBytes for Sweep {
    fn parse_from_bytes(bytes: &[u8]) -> IResult<&[u8], Self> {
        // Parse the prefix of the message
        let (bytes, prefix) = peek(alt((tag("$S"), tag("$s"), tag("$z"))))(bytes)?;
        let sweep = match prefix {
            b"$S" => Sweep::Standard(SweepDataStandard::parse_from_bytes(bytes)?.1),
            b"$s" => Sweep::Ext(SweepDataExt::parse_from_bytes(bytes)?.1),
            b"$z" => Sweep::Large(SweepDataLarge::parse_from_bytes(bytes)?.1),
            _ => return Err(nom::Err::Failure(Error::new(bytes, ErrorKind::Tag))),
        };

        Ok((bytes, sweep))
    }
}

macro_rules! impl_sweep_data {
    ($sweep_data:ident, $prefix:expr, $amp_parser:expr) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $sweep_data {
            amplitudes_dbm: Vec<f32>,
            timestamp: DateTime<Utc>,
        }

        impl Message for $sweep_data {
            const PREFIX: &'static [u8] = $prefix;
        }

        impl ParseFromBytes for $sweep_data {
            fn parse_from_bytes(bytes: &[u8]) -> IResult<&[u8], Self> {
                // Parse the prefix of the message
                let (bytes, _) = tag(Self::PREFIX)(bytes)?;

                // Get the slice containing the amplitudes in the sweep data
                let (bytes, amps) = $amp_parser(bytes)?;

                // Convert the amplitude bytes into dBm by dividing them by -2
                let amplitudes_dbm = amps.iter().map(|&byte| f32::from(byte) / -2.).collect();

                // Consume any \r or \r\n line endings and make sure there aren't any bytes left
                let (bytes, _) = all_consuming(opt(line_ending))(bytes)?;

                Ok((
                    bytes,
                    $sweep_data {
                        amplitudes_dbm,
                        timestamp: Utc::now(),
                    },
                ))
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
        let sweep_data = SweepDataStandard::parse_from_bytes(&bytes[..]).unwrap().1;
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
        let sweep_data = SweepDataExt::parse_from_bytes(&bytes[..]).unwrap().1;
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
        let sweep_data = SweepDataLarge::parse_from_bytes(&bytes[..]).unwrap().1;
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
        let sweep_data_error = SweepDataStandard::parse_from_bytes(&bytes[..]).unwrap_err();
        assert!(matches!(sweep_data_error, nom::Err::Error(..)));
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
        let sweep_data_error = SweepDataStandard::parse_from_bytes(&bytes[..]).unwrap_err();
        assert!(matches!(sweep_data_error, nom::Err::Incomplete(..)));
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
