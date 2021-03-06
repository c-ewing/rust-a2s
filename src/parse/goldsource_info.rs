use super::*;

use nom::number::complete::le_u8;
use nom::IResult;

// # Structs
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResponseInfo {
    pub address: String,
    pub name: String,
    pub map: String,
    pub folder: String,
    pub game: String,
    pub players: u8,
    pub max_players: u8,
    pub protocol: u8,
    pub server_type: ServerType,
    pub environment: Environment,
    pub visibility: bool,
    pub mod_half_life: bool,
    pub mod_fields: Option<HalfLifeMod>,
    pub vac: bool,
    pub bots: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ServerType {
    Dedicated,
    NonDedicated,
    SourceTV,
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
pub enum Environment {
    Linux,
    Windows,
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
            // Otherwise
            _ => Environment::Other(input),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HalfLifeMod {
    pub link: String,
    pub download_link: String,
    // Null Byte
    pub version: i32,
    pub size: i32,
    pub mod_type: ModType,
    pub dll: ModDLL,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModType {
    SingleAndMultiplayer,
    MultiplayerOnly,
    Other(u8),
}

impl From<u8> for ModType {
    fn from(input: u8) -> Self {
        match input {
            0 => ModType::SingleAndMultiplayer,
            1 => ModType::MultiplayerOnly,
            _ => ModType::Other(input),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModDLL {
    HalfLife,
    Custom,
    Other(u8),
}

impl From<u8> for ModDLL {
    fn from(input: u8) -> Self {
        match input {
            0 => ModDLL::HalfLife,
            1 => ModDLL::Custom,
            _ => ModDLL::Other(input),
        }
    }
}

// # Exposed final parser
// Makes sure that all of the input data was consumed, if not to much data was fed or something
// TODO: comment better
pub fn p_goldsource_info(input: &[u8]) -> IResult<&[u8], goldsource_info::ResponseInfo> {
    all_consuming(goldsource_info)(input)
}

// # Private parsing helper functions
// Does the bulk of the parsing
fn goldsource_info(input: &[u8]) -> IResult<&[u8], goldsource_info::ResponseInfo> {
    let (input, address) = c_string(input)?;
    let (input, name) = c_string(input)?;
    let (input, map) = c_string(input)?;
    let (input, folder) = c_string(input)?;
    let (input, game) = c_string(input)?;
    let (input, players) = le_u8(input)?;
    let (input, max_players) = le_u8(input)?;
    let (input, protocol) = le_u8(input)?;
    let (input, server_type) = server_type(input)?;
    let (input, environment) = environment(input)?;
    let (input, visibility) = bool(input)?;
    let (input, mod_half_life) = bool(input)?;
    let (input, mod_fields) = mod_fields(input, mod_half_life)?;
    let (input, vac) = bool(input)?;
    let (input, bots) = le_u8(input)?;

    Ok((
        input,
        ResponseInfo {
            address,
            name,
            map,
            folder,
            game,
            players,
            max_players,
            protocol,
            server_type,
            environment,
            visibility,
            mod_half_life,
            mod_fields,
            vac,
            bots,
        },
    ))
}

fn server_type(input: &[u8]) -> IResult<&[u8], ServerType> {
    le_u8(input).map(|(next, res)| (next, res.into()))
}

fn environment(input: &[u8]) -> IResult<&[u8], Environment> {
    le_u8(input).map(|(next, res)| (next, res.into()))
}

fn mod_type(input: &[u8]) -> IResult<&[u8], ModType> {
    le_u8(input).map(|(next, res)| (next, res.into()))
}

fn dll(input: &[u8]) -> IResult<&[u8], ModDLL> {
    le_u8(input).map(|(next, res)| (next, res.into()))
}

fn mod_fields(input: &[u8], is_mod: bool) -> IResult<&[u8], Option<HalfLifeMod>> {
    if is_mod {
        let (input, link) = c_string(input)?;
        let (input, download_link) = c_string(input)?;
        let (input, _) = null(input)?;
        let (input, version) = le_i32(input)?;
        let (input, size) = le_i32(input)?;
        let (input, mod_type) = mod_type(input)?;
        let (input, dll) = dll(input)?;

        Ok((
            input,
            Some(HalfLifeMod {
                link,
                download_link,
                version,
                size,
                mod_type,
                dll,
            }),
        ))
    } else {
        Ok((input, None))
    }
}

// # Tests
#[test]
fn info_cs() {
    // Packet from souce wiki
    // Omitts first 5 bytes as parse_source_info assumes the packet data has been combined and the message type determined
    let cs: [u8; 150] = [
        0x37, 0x37, 0x2E, 0x31, 0x31, 0x31, 0x2E, 0x31, 0x39, 0x34, 0x2E, 0x31, 0x31, 0x30, 0x3A,
        0x32, 0x37, 0x30, 0x31, 0x35, 0x00, 0x46, 0x52, 0x20, 0x2D, 0x20, 0x56, 0x65, 0x72, 0x79,
        0x47, 0x61, 0x6D, 0x65, 0x73, 0x2E, 0x6E, 0x65, 0x74, 0x20, 0x2D, 0x20, 0x44, 0x65, 0x61,
        0x74, 0x6D, 0x61, 0x74, 0x63, 0x68, 0x20, 0x2D, 0x20, 0x6F, 0x6E, 0x6C, 0x79, 0x20, 0x73,
        0x75, 0x72, 0x66, 0x5F, 0x73, 0x6B, 0x69, 0x20, 0x2D, 0x20, 0x6E, 0x67, 0x52, 0x00, 0x73,
        0x75, 0x72, 0x66, 0x5F, 0x73, 0x6B, 0x69, 0x00, 0x63, 0x73, 0x74, 0x72, 0x69, 0x6B, 0x65,
        0x00, 0x43, 0x6F, 0x75, 0x6E, 0x74, 0x65, 0x72, 0x2D, 0x53, 0x74, 0x72, 0x69, 0x6B, 0x65,
        0x00, 0x0C, 0x12, 0x2F, 0x64, 0x6C, 0x00, 0x01, 0x77, 0x77, 0x77, 0x2E, 0x63, 0x6F, 0x75,
        0x6E, 0x74, 0x65, 0x72, 0x2D, 0x73, 0x74, 0x72, 0x69, 0x6B, 0x65, 0x2E, 0x6E, 0x65, 0x74,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x9E, 0xF7, 0x0A, 0x00, 0x01, 0x01, 0x00,
    ];

    let response = parse_goldsource_info(&cs).unwrap();

    assert_eq!(
        goldsource_info::ResponseInfo {
            address: "77.111.194.110:27015".to_string(),
            name: "FR - VeryGames.net - Deatmatch - only surf_ski - ngR".to_string(),
            map: "surf_ski".to_string(),
            folder: "cstrike".to_string(),
            game: "Counter-Strike".to_string(),
            players: 12,
            max_players: 18,
            protocol: 47,
            server_type: ServerType::Dedicated,
            environment: Environment::Linux,
            visibility: false,
            mod_half_life: true,
            mod_fields: Some(HalfLifeMod {
                link: "www.counter-strike.net".to_string(),
                download_link: "".to_string(),
                version: 1,
                size: 184000000,
                mod_type: ModType::SingleAndMultiplayer,
                dll: ModDLL::Custom,
            }),
            vac: true,
            bots: 0,
        },
        response
    );
}
