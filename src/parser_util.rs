use nom::{
    bytes::complete::take_till,
    character::complete::char,
    combinator::opt,
    number::complete::le_u8,
    sequence::terminated,
    IResult,
};

// # Struct / Enums
#[derive(Clone, Debug, PartialEq, Eq)]
/// Indicates the type of the server  
/// Gold Source uses the capital (uppercase?) version of the characters  
/// Used in [`info_goldsource`](crate::info_goldsource), [`info_source`](crate::info_source)
pub enum ServerType {
    /// Dedicated (Gold)Source server -> 'd' (0x44) or 'D' (0x64)
    Dedicated,
    /// Non Dedicated (Gold)Source server -> 'l' (0x4C) or 'L' (0x6C) - TODO: Is this used anywhere?
    NonDedicated,
    /// SourceTV relay server -> 'p' (0x50) or 'P' (0x70)
    SourceTV,
    /// Holds the value of any other parsed value. In theory this should be unused, however there may be some odd games
    Other(u8),
}

impl From<u8> for ServerType {
    fn from(input: u8) -> Self {
        // Docs say uppercase but the example has lower. Maybe it uses either?
        match input {
            // 'd' or 'D'
            0x44 => ServerType::Dedicated,
            0x64 => ServerType::Dedicated,
            // 'l' or 'L'
            0x4C => ServerType::NonDedicated,
            0x6C => ServerType::NonDedicated,
            // 'p' or 'P'
            0x50 => ServerType::SourceTV,
            0x70 => ServerType::SourceTV,
            // Otherwise
            _ => ServerType::Other(input),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Indicates the Operating System the server is running on  
/// Gold Source uses the capital (uppercase?) version of the characters  
/// Used in [`info_goldsource`](crate::info_goldsource), [`info_source`](crate::info_source)
pub enum Environment {
    /// Linux -> 'l' (0x4C) or 'L' (0x6C)
    Linux,
    /// Windows -> 'w' (0x57) or 'W' (0x77)
    Windows,
    /// MacOS -> 'm' (0x6D) or 'o' (0x6F), uppercase equivalents are included as well TODO: Test if the capital letter varients exist in any source engine game.
    MacOS,
    /// Any other operating system value, Should never hit this in theory, however there may be some odd games
    Other(u8),
}

impl From<u8> for Environment {
    fn from(input: u8) -> Self {
        match input {
            // 'l' or 'L'
            0x4C => Environment::Linux,
            0x6C => Environment::Linux,
            // 'w' or 'W'
            0x57 => Environment::Windows,
            0x77 => Environment::Windows,
            // 'm' or 'M' or 'o' or 'O'
            0x4D => Environment::MacOS,
            0x6D => Environment::MacOS,
            0x4F => Environment::MacOS,
            0x6F => Environment::MacOS,
            // Otherwise
            _ => Environment::Other(input),
        }
    }
}

// TODO: Tests
// # General Helper functions used across several parsers
/// Reads one byte from the input slice and returns the ServerType
pub(crate) fn server_type(input: &[u8]) -> IResult<&[u8], ServerType> {
    le_u8(input).map(|(next, res)| (next, res.into()))
}

/// Reads one byte from the input slice and returns the Environment
pub(crate) fn environment(input: &[u8]) -> IResult<&[u8], Environment> {
    le_u8(input).map(|(next, res)| (next, res.into()))
}

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

/// Reads one null byte (0x00) from input. If the next byte is not null an Error is returned.
pub(crate) fn parse_null(input: &[u8]) -> IResult<&[u8], char> {
    char(0x00 as char)(input)
}

/// Reads one byte from the input and returns false if it is equal to 0, 1 otherwise.
pub(crate) fn parse_bool(input: &[u8]) -> IResult<&[u8], bool> {
    le_u8(input).map(|(next, res)| (next, res != 0))
}
