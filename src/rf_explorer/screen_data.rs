use crate::rf_explorer::{Message, ParseMessageError};
use std::convert::{TryFrom, TryInto};

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

impl TryFrom<&[u8]> for ScreenData {
    type Error = ParseMessageError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.starts_with(Self::PREFIX) {
            let screen_data_array: &[u8; ScreenData::ROWS * ScreenData::COLUMNS] = bytes
                .get(ScreenData::PREFIX.len()..)
                .ok_or_else(|| ParseMessageError::InvalidData)?
                .try_into()
                .map_err(|_| ParseMessageError::InvalidData)?;
            let screen_data_matrix = {
                let mut matrix = [[0; ScreenData::COLUMNS]; ScreenData::ROWS];
                for (row_index, row_bytes) in screen_data_array
                    .chunks_exact(ScreenData::COLUMNS)
                    .enumerate()
                {
                    matrix[row_index].clone_from_slice(row_bytes);
                }
                matrix
            };
            Ok(ScreenData { screen_data_matrix })
        } else {
            Err(ParseMessageError::InvalidData)
        }
    }
}
