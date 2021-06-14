use nom::{
    bytes::complete::take_till,
    character::complete::char,
    combinator::opt,
    number::complete::{le_i32, le_u8},
    sequence::terminated,
    IResult,
};

/// Parses a C style String
/// Reads all bytes until a null terminator is reached.
/// All data transmitted by the protocol should be UTF-8. from_utf8_lossy is used as it can take a slice.
pub(crate) fn c_string(input: &[u8]) -> IResult<&[u8], String> {
    terminated(take_till(|c| c == 0x00u8), char(0x00 as char))(input)
        .map(|(next, res)| (next, String::from_utf8_lossy(res).into_owned()))
}

/// Attempts to parse a byte, if the parser fails None is returned
pub(crate) fn opt_le_u8(input: &[u8]) -> IResult<&[u8], Option<u8>> {
    opt(le_u8)(input)
}

/// Attempts to parse a little endian i32, if the parser fails None is returned
pub(crate) fn opt_le_i32(input: &[u8]) -> IResult<&[u8], Option<i32>> {
    opt(le_i32)(input)
}

/// Reads one null byte (0x00) from input. If the byte is not null an Error is returned.
pub(crate) fn parse_null(input: &[u8]) -> IResult<&[u8], char> {
    char(0x00 as char)(input)
}

/// Reads one byte from the input and returns false if it is equal to 0, 1 otherwise.
pub(crate) fn parse_bool(input: &[u8]) -> IResult<&[u8], bool> {
    le_u8(input).map(|(next, res)| (next, res != 0))
}
