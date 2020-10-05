use crate::messages::RfeMessage;
use std::convert::TryFrom;

#[derive(Clone)]
pub struct ScreenData {
    data: [[u8; 128]; 8],
}

impl ScreenData {
    const WIDTH_PX: u8 = 128;
    const HEIGHT_PX: u8 = 64;

    pub fn data(&self) -> &[[u8; 128]; 8] {
        &self.data
    }
}

impl RfeMessage for ScreenData {
    const PREFIX: &'static [u8] = b"$D";
}

impl TryFrom<&[u8]> for ScreenData {
    type Error = ();

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.starts_with(Self::PREFIX) {
            let screen_data_bytes = bytes.get(2..(128 * 8) + 2).ok_or_else(|| ())?;
            let data = {
                let mut data = [[0; 128]; 8];
                for (row_index, row_bytes) in screen_data_bytes.chunks_exact(128).enumerate() {
                    for column_index in 0..data[row_index].len() {
                        data[row_index][column_index] = row_bytes[column_index];
                    }
                }
                data
            };
            Ok(ScreenData { data })
        } else {
            Err(())
        }
    }
}
