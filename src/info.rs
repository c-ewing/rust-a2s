// # Imports
use nom::{
    combinator::all_consuming,
    error::Error,
    number::complete::{le_i32, le_u8, le_i16, le_u64},
    Finish, IResult,
};

use crate::parser_util::{
    c_string,  parse_bool, parse_null, opt_le_u8
};
// # Enums

#[derive(Clone, Debug, PartialEq, Eq)]
/// Indicates the type of the server  
/// Gold Source uses the capital (uppercase?) version of the characters  
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

// TODO: Specialize this in the parsing functions as Gold / Source use different vals
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

// TODO: Specialize for Gold / Source
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

#[derive(Clone, Debug, PartialEq, Eq)]
/// Parsed Half-Life mod type
pub enum ModType {
    /// Single and Multiplayer mod
    SingleAndMultiplayer,
    /// Multiplayer only mod
    MultiplayerOnly,
    /// Any other mod type (this should be unused!)
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
/// Custom or standard Half-Life DLL for the mod
pub enum ModDLL {
    /// Mod uses the base Half-Life DLL
    HalfLife,
    /// MOD uses a custom DLL
    Custom,
    /// Any other response type (should be unused!)
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

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq, Eq)]
/// Possible gamemodes for The Ship
pub enum TheShipGameMode {
    /// 0 -> Hunt Gamemode
    Hunt,
    /// 1 -> Elimination Gamemode
    Elimination,
    /// 2 -> Duel Gamemode
    Duel,
    /// 3 -> Deathmatch Gamemode
    Deathmatch,
    /// 4 -> VIP_Team Gamemode
    VIP_Team,
    /// 5 -> Team Elimination Gamemode
    Team_Elimination,
    /// Any other game mode (Should not occur)
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

// # Structs

#[derive(Clone, Debug, PartialEq, Eq)]
/// Contains parsed Half-Life mod data
pub struct HalfLifeMod {
    /// Website for the mod
    pub link: String,
    /// Download link for the mod
    pub download_link: String,
    /// Mod Version
    pub version: i32,
    /// Size of the mod in bytes
    pub size: i32,
    /// Single player and multiplayer mod or multiplayer only mod
    pub mod_type: ModType,
    /// If the mod uses a custom DLL or the Half-Life DLL
    pub dll: ModDLL,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Optionally transmitted data about the configuration of The Ship (only used by one game)
pub struct TheShipFields {
    /// Gamemode
    pub mode: TheShipGameMode,
    /// Number of witnesses needed to arrest a player
    pub witnesses: u8,
    /// Time in seconds before the player is arrested while witnessed
    pub duration: u8,
}
#[derive(Clone, Debug, PartialEq, Eq)]
/// Optional Extra Data Fields
/// if `EDF & 0x80` then the servers port is also transmitted
/// if `EDF & 0x10` then servers steam ID is transmitted
/// if `EDF & 0x40` then the spectator port number and name of the spectator server for SourceTV are contained
/// if `EDF & 0x20` then tags that describe the game are transmitted
/// if `EDF & 0x01` then the full game ID and untruncated App ID are contained.  
pub struct ExtraDataFields {
    /// Servers port
    pub port: Option<i16>,
    /// Server SteamID
    pub steam_id: Option<u64>,
    /// Port for Source TV
    pub source_tv_port: Option<i16>,
    /// Name of the Spectator server for Source TV
    pub source_tv_name: Option<String>,
    /// Tags that describe the game
    pub keywords: Option<String>,
    /// 64bit GameID, if present then the lower 24bits are a more accurate AppID as it may have been truncated to fit in 16bits previously
    pub game_id: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Data contained within an [GoldSource A2S_INFO Response](https://developer.valvesoftware.com/wiki/Server_queries#Obsolete_GoldSource_Response)
pub struct GoldSourceResponseInfo {
    /// Server IP address IPV4:PORT
    pub address: String,
    /// Name of the Server
    pub name: String,
    /// Map currently loaded
    pub map: String,
    /// Folder name containing game files
    pub folder: String,
    /// Name of the game(mode)
    pub game: String,
    /// Number of currently connected (and connecting) players
    pub players: u8,
    /// Maximum number of players
    pub max_players: u8,
    /// Protocol version used by the server
    pub protocol: u8,
    /// Hosting type of the server
    pub server_type: ServerType,
    /// Operating system of the server
    pub environment: Environment,
    /// Is the server private
    pub visibility: bool,
    /// Is the server a Half Life Mod
    pub mod_half_life: bool,
    /// If it is a mod, HalfLifeMod contains the mod data
    pub mod_fields: Option<HalfLifeMod>,
    /// Is the server secured by VAC
    pub vac: bool,
    /// Number of bots currently connected to the server
    pub bots: u8,
}

// # Structs
#[derive(Clone, Debug, PartialEq, Eq)]
/// Data contained within an [Source A2S_INFO Response](https://developer.valvesoftware.com/wiki/Server_queries#Response_Format)
pub struct SourceResponseInfo {
    /// Procool version used by the server
    pub protocol: u8,
    /// Name of the server
    pub name: String,
    /// Current map name
    pub map: String,
    /// Name of the folder containing the game files
    pub folder: String,
    /// Full name of the game(mode)
    pub game: String,
    /// [Steam Application ID] (https://developer.valvesoftware.com/wiki/Steam_Application_IDs) for the game
    pub app_id: i16,
    /// Number of connected and connecting players
    pub players: u8,
    /// Maximum number of connected players
    pub max_players: u8,
    /// Number of connected bots
    pub bots: u8,
    /// Hosting type of the server
    pub server_type: ServerType,
    /// Operating system the server is running on
    pub environment: Environment,
    /// Is the server private
    pub visibility: bool,
    /// Is the server secured with VAC
    pub vac: bool,
    /// Optional data transmitted by [The Ship](https://developer.valvesoftware.com/wiki/The_Ship)
    pub the_ship: Option<TheShipFields>,
    /// Version of the game installed on the server
    pub version: String,
    /// Extra Data Flag according to the [wiki](https://developer.valvesoftware.com/wiki/Server_queries#Response_Format)
    pub extra_data_flag: u8,
    /// Optional Data signalled by the EDF flag
    /// if `EDF & 0x80` then the servers port is also transmitted
    /// if `EDF & 0x10` then servers steam ID is transmitted
    /// if `EDF & 0x40` then the spectator port number and name of the spectator server for SourceTV are contained
    /// if `EDF & 0x20` then tags that describe the game are transmitted
    /// if `EDF & 0x01` then the full game ID and untruncated App ID are contained. 
    pub extra_data_fields: ExtraDataFields,
}

// # Public Parsers
pub fn parse_goldsource_info(input: &[u8]) -> Result<GoldSourceResponseInfo, Error<&[u8]>> {
    match p_goldsource_info(input).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}

pub fn parse_source_info(input: &[u8]) -> Result<SourceResponseInfo, Error<&[u8]>> {
    match p_source_info(input).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}

// # Support Parsers
// Does the bulk of the parsing
fn goldsource_info(input: &[u8]) -> IResult<&[u8], GoldSourceResponseInfo> {
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
    let (input, visibility) = parse_bool(input)?;
    let (input, mod_half_life) = parse_bool(input)?;
    let (input, mod_fields) = mod_fields(input, mod_half_life)?;
    let (input, vac) = parse_bool(input)?;
    let (input, bots) = le_u8(input)?;

    Ok((
        input,
        GoldSourceResponseInfo {
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

// Make sure the parser ate all the data
// TODO: move into main parsing function
fn p_goldsource_info(input: &[u8]) -> IResult<&[u8], GoldSourceResponseInfo> {
    all_consuming(goldsource_info)(input)
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
        let (input, _) = parse_null(input)?;
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

/// Reads one byte from the input slice and returns the ServerType
fn server_type(input: &[u8]) -> IResult<&[u8], ServerType> {
    le_u8(input).map(|(next, res)| (next, res.into()))
}

/// Reads one byte from the input slice and returns the Environment
fn environment(input: &[u8]) -> IResult<&[u8], Environment> {
    le_u8(input).map(|(next, res)| (next, res.into()))
}

fn p_source_info(input: &[u8]) -> IResult<&[u8], SourceResponseInfo> {
    all_consuming(source_info)(input)
}
// Does the bulk of the parsing
fn source_info(input: &[u8]) -> IResult<&[u8], SourceResponseInfo> {
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
    let (input, visibility) = parse_bool(input)?;
    let (input, vac) = parse_bool(input)?;
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
        SourceResponseInfo {
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
        GoldSourceResponseInfo {
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
        SourceResponseInfo {
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
        SourceResponseInfo {
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

