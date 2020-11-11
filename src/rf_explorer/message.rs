use nom::IResult;

pub trait Message: Sized {
    fn from_bytes(bytes: &[u8]) -> IResult<&[u8], Self>;
}
