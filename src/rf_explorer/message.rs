use nom::IResult;

pub trait ParseFromBytes: Sized {
    fn parse_from_bytes(bytes: &[u8]) -> IResult<&[u8], Self>;
}

pub trait Message: ParseFromBytes {
    const PREFIX: &'static [u8];
}
