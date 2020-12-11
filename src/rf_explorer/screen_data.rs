use crate::rf_explorer::{Message, ParseFromBytes};
use nom::{
    bytes::complete::tag,
    bytes::streaming::take,
    character::complete::line_ending,
    combinator::{all_consuming, map_res, opt},
    IResult,
};
use std::convert::TryInto;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ScreenData {
    screen_data_matrix: [[u8; ScreenData::COLUMNS]; ScreenData::ROWS],
}

impl ScreenData {
    pub const ROWS: usize = 8;
    pub const COLUMNS: usize = 128;
    pub const VERTICAL_PX_PER_ROW: usize = 8;

    pub fn as_byte_matrix(&self) -> &[[u8; ScreenData::COLUMNS]; ScreenData::ROWS] {
        &self.screen_data_matrix
    }
}

impl Message for ScreenData {
    const PREFIX: &'static [u8] = b"$D";
}

impl ParseFromBytes for ScreenData {
    fn parse_from_bytes(bytes: &[u8]) -> IResult<&[u8], Self> {
        // Parse the prefix of the message
        let (bytes, _) = tag(Self::PREFIX)(bytes)?;

        // Parse the screen data
        let (bytes, screen_data): (&[u8], &[u8; Self::ROWS * Self::COLUMNS]) =
            map_res(take(Self::ROWS * Self::COLUMNS), TryInto::try_into)(bytes)?;

        // Consume any \r or \r\n line endings and make sure there aren't any bytes left
        let (bytes, _) = all_consuming(opt(line_ending))(bytes)?;

        // Convert the slice of bytes representing the screen data into a matrix
        let screen_data_matrix = {
            let mut matrix = [[0; ScreenData::COLUMNS]; ScreenData::ROWS];
            for (row_index, row_bytes) in screen_data.chunks_exact(ScreenData::COLUMNS).enumerate()
            {
                matrix[row_index].clone_from_slice(row_bytes);
            }
            matrix
        };

        Ok((bytes, ScreenData { screen_data_matrix }))
    }
}
