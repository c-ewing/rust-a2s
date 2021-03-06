use super::*;

use nom::number::complete::{le_i16, le_u64, le_u8};
use nom::IResult;

// # Structs
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResponseInfo {
    pub protocol: u8,
    pub name: String,
    pub map: String,
    pub folder: String,
    pub game: String,
    pub app_id: i16,
    pub players: u8,
    pub max_players: u8,
    pub bots: u8,
    pub server_type: ServerType,
    pub environment: Environment,
    pub visibility: bool,
    pub vac: bool,
    pub the_ship: Option<TheShipFields>,
    pub version: String,
    pub extra_data_flag: u8,
    pub extra_data_fields: ExtraDataFields,
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
    MacOS,
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
            // 'm' or 'M'
            0x4D => Environment::MacOS,
            0x6D => Environment::MacOS,
            // Otherwise
            _ => Environment::Other(input),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TheShipGameMode {
    Hunt,
    Elimination,
    Duel,
    Deathmatch,
    VIP_Team,
    Team_Elimination,
    Other(u8),
}

impl From<u8> for TheShipGameMode {
    fn from(input: u8) -> Self {
        match input {
            0 => TheShipGameMode::Hunt,
            1 => TheShipGameMode::Elimination,
            2 => TheShipGameMode::Duel,
            3 => TheShipGameMode::Deathmatch,
            4 => TheShipGameMode::VIP_Team,
            5 => TheShipGameMode::Team_Elimination,
            _ => TheShipGameMode::Other(input),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TheShipFields {
    pub mode: TheShipGameMode,
    pub witnesses: u8,
    pub duration: u8,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtraDataFields {
    pub port: Option<i16>,
    pub steam_id: Option<u64>,
    pub source_tv_port: Option<i16>,
    pub source_tv_name: Option<String>,
    pub keywords: Option<String>,
    pub game_id: Option<u64>,
}

// # Exposed final parser
// Makes sure that all of the input data was consumed, if not to much data was fed or something
// TODO: comment better
pub fn p_source_info(input: &[u8]) -> IResult<&[u8], source_info::ResponseInfo> {
    all_consuming(source_info)(input)
}

// # Private parsing helper functions
// Does the bulk of the parsing
fn source_info(input: &[u8]) -> IResult<&[u8], source_info::ResponseInfo> {
    let (input, protocol) = le_u8(input)?;
    let (input, name) = c_string(input)?;
    let (input, map) = c_string(input)?;
    let (input, folder) = c_string(input)?;
    let (input, game) = c_string(input)?;
    let (input, app_id) = le_i16(input)?;
    let (input, players) = le_u8(input)?;
    let (input, max_players) = le_u8(input)?;
    let (input, bots) = le_u8(input)?;
    let (input, server_type) = server_type(input)?;
    let (input, environment) = environment(input)?;
    let (input, visibility) = bool(input)?;
    let (input, vac) = bool(input)?;
    let (input, the_ship) = the_ship(input, app_id == 2400)?;

    // The version is either the last data in the input, or there is the extra data flag
    let (input, version) = c_string(input)?;

    // Doesn't always exist, need to make optional
    let (input, extra_data_flag) = opt_le_u8(input)?;
    // Unwrap, 0 means no data flags
    let extra_data_flag: u8 = extra_data_flag.unwrap_or(0);

    // TODO: This is not optimal, should skip trying to parse all of the values if the flag is 0
    let (input, extra_data_fields) = extra_data_fields(input, extra_data_flag)?;

    Ok((
        input,
        ResponseInfo {
            protocol,
            name,
            map,
            folder,
            game,
            app_id,
            players,
            max_players,
            bots,
            server_type,
            environment,
            visibility,
            vac,
            the_ship,
            version,
            extra_data_flag,
            extra_data_fields,
        },
    ))
}

fn server_type(input: &[u8]) -> IResult<&[u8], ServerType> {
    le_u8(input).map(|(next, res)| (next, res.into()))
}

fn environment(input: &[u8]) -> IResult<&[u8], Environment> {
    le_u8(input).map(|(next, res)| (next, res.into()))
}

fn the_ship(input: &[u8], is_ship: bool) -> IResult<&[u8], Option<TheShipFields>> {
    if is_ship {
        let (input, mode) = le_u8(input).map(|(next, res)| (next, res.into()))?;
        let (input, witnesses) = le_u8(input)?;
        let (input, duration) = le_u8(input)?;

        Ok((
            input,
            Some(TheShipFields {
                mode,
                witnesses,
                duration,
            }),
        ))
    } else {
        Ok((input, None))
    }
}

fn extra_data_fields(input: &[u8], extra_data_flag: u8) -> IResult<&[u8], ExtraDataFields> {
    let (input, port) = port(input, extra_data_flag)?;
    let (input, steam_id) = steam_id(input, extra_data_flag)?;
    let (input, source_tv_port) = source_tv_port(input, extra_data_flag)?;
    let (input, source_tv_name) = source_tv_name(input, extra_data_flag)?;
    let (input, keywords) = keywords(input, extra_data_flag)?;
    let (input, game_id) = game_id(input, extra_data_flag)?;

    Ok((
        input,
        ExtraDataFields {
            port,
            steam_id,
            source_tv_port,
            source_tv_name,
            keywords,
            game_id,
        },
    ))
}

fn port(input: &[u8], flag: u8) -> IResult<&[u8], Option<i16>> {
    if flag & 0x80 != 0 {
        let (input, port) = le_i16(input)?;

        Ok((input, Some(port)))
    } else {
        Ok((input, None))
    }
}

fn steam_id(input: &[u8], flag: u8) -> IResult<&[u8], Option<u64>> {
    if flag & 0x10 != 0 {
        let (input, steam_id) = le_u64(input)?;

        Ok((input, Some(steam_id)))
    } else {
        Ok((input, None))
    }
}

fn source_tv_port(input: &[u8], flag: u8) -> IResult<&[u8], Option<i16>> {
    if flag & 0x40 != 0 {
        let (input, port) = le_i16(input)?;

        Ok((input, Some(port)))
    } else {
        Ok((input, None))
    }
}

fn source_tv_name(input: &[u8], flag: u8) -> IResult<&[u8], Option<String>> {
    if flag & 0x40 != 0 {
        let (input, name) = c_string(input)?;

        Ok((input, Some(name)))
    } else {
        Ok((input, None))
    }
}

fn keywords(input: &[u8], flag: u8) -> IResult<&[u8], Option<String>> {
    if flag & 0x20 != 0 {
        let (input, keywords) = c_string(input)?;

        Ok((input, Some(keywords)))
    } else {
        Ok((input, None))
    }
}

fn game_id(input: &[u8], flag: u8) -> IResult<&[u8], Option<u64>> {
    if flag & 0x20 != 0 {
        let (input, game_id) = le_u64(input)?;

        Ok((input, Some(game_id)))
    } else {
        Ok((input, None))
    }
}

// # Tests
#[test]
fn info_css() {
    // Packet from souce wiki
    // Omitts first 5 bytes as parse_source_info assumes the packet data has been combined and the message type determined
    let css: [u8; 95] = [
        0x02, 0x67, 0x61, 0x6D, 0x65, 0x32, 0x78, 0x73, 0x2E, 0x63, 0x6F, 0x6D, 0x20, 0x43, 0x6F,
        0x75, 0x6E, 0x74, 0x65, 0x72, 0x2D, 0x53, 0x74, 0x72, 0x69, 0x6B, 0x65, 0x20, 0x53, 0x6F,
        0x75, 0x72, 0x63, 0x65, 0x20, 0x23, 0x31, 0x00, 0x64, 0x65, 0x5F, 0x64, 0x75, 0x73, 0x74,
        0x00, 0x63, 0x73, 0x74, 0x72, 0x69, 0x6B, 0x65, 0x00, 0x43, 0x6F, 0x75, 0x6E, 0x74, 0x65,
        0x72, 0x2D, 0x53, 0x74, 0x72, 0x69, 0x6B, 0x65, 0x3A, 0x20, 0x53, 0x6F, 0x75, 0x72, 0x63,
        0x65, 0x00, 0xF0, 0x00, 0x05, 0x10, 0x04, 0x64, 0x6C, 0x00, 0x00, 0x31, 0x2E, 0x30, 0x2E,
        0x30, 0x2E, 0x32, 0x32, 0x00,
    ];

    let response = parse_source_info(&css).unwrap();

    assert_eq!(
        ResponseInfo {
            protocol: 2,
            name: "game2xs.com Counter-Strike Source #1".to_string(),
            map: "de_dust".to_string(),
            folder: "cstrike".to_string(),
            game: "Counter-Strike: Source".to_string(),
            app_id: 240,
            players: 5,
            max_players: 16,
            bots: 4,
            server_type: ServerType::Dedicated,
            environment: Environment::Linux,
            visibility: false,
            vac: false,
            the_ship: None,
            version: "1.0.0.22".to_string(),
            extra_data_flag: 0,
            extra_data_fields: ExtraDataFields {
                port: None,
                steam_id: None,
                source_tv_port: None,
                source_tv_name: None,
                keywords: None,
                game_id: None
            },
        },
        response
    );
}

#[test]
fn info_the_ship() {
    // Omitts first 5 bytes as parse_source_info assumes the packet data has been combined and the message type determined
    let ship: [u8; 56] = [
        0x07, 0x53, 0x68, 0x69, 0x70, 0x20, 0x53, 0x65, 0x72, 0x76, 0x65, 0x72, 0x00, 0x62, 0x61,
        0x74, 0x61, 0x76, 0x69, 0x65, 0x72, 0x00, 0x73, 0x68, 0x69, 0x70, 0x00, 0x54, 0x68, 0x65,
        0x20, 0x53, 0x68, 0x69, 0x70, 0x00, 0x60, 0x09, 0x01, 0x05, 0x00, 0x6C, 0x77, 0x00, 0x00,
        0x01, 0x03, 0x03, 0x31, 0x2E, 0x30, 0x2E, 0x30, 0x2E, 0x34, 0x00,
    ];

    let response = parse_source_info(&ship).unwrap();

    assert_eq!(
        ResponseInfo {
            protocol: 7,
            name: "Ship Server".to_string(),
            map: "batavier".to_string(),
            folder: "ship".to_string(),
            game: "The Ship".to_string(),
            app_id: 2400,
            players: 1,
            max_players: 5,
            bots: 0,
            server_type: ServerType::NonDedicated,
            environment: Environment::Windows,
            visibility: false,
            vac: false,
            the_ship: Some(TheShipFields {
                mode: TheShipGameMode::Elimination,
                witnesses: 3,
                duration: 3,
            }),
            version: "1.0.0.4".to_string(),
            extra_data_flag: 0,
            extra_data_fields: ExtraDataFields {
                port: None,
                steam_id: None,
                source_tv_port: None,
                source_tv_name: None,
                keywords: None,
                game_id: None
            },
        },
        response
    );
}
