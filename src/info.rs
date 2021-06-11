// # Imports
use nom::{
    error::Error,
    number::complete::{le_i16, le_i32, le_u64, le_u8},
    Finish, IResult,
};

use crate::parser_util::{c_string, opt_le_u8, parse_bool, parse_null};
// # Enums

#[derive(Clone, Debug, PartialEq, Eq)]
/// Indicates the type of the server  
/// Gold Source uses the capital (uppercase?) version of the characters  
pub enum ServerType {
    /// Dedicated (Gold)Source server -> 'd' (0x44) or 'D' (0x64)
    Dedicated,
    /// Non Dedicated (Gold)Source server -> 'l' (0x4C) or 'L' (0x6C)
    NonDedicated,
    /// SourceTV relay server -> 'p' (0x50) or 'P' (0x70)
    SourceTV,
    /// Rag Doll Kung Fu always returns 0
    RagDollKungFu,
    /// Holds the value of any other parsed value. In theory this should be unused, however there may be some odd games
    Invalid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Indicates the Operating System the server is running on  
/// Gold Source uses the capital (uppercase?) version of the characters  
pub enum Environment {
    /// Linux -> 'l' (0x4C) or 'L' (0x6C)
    Linux,
    /// Windows -> 'w' (0x57) or 'W' (0x77)
    Windows,
    /// MacOS -> 'm' (0x6D) or 'o' (0x6F), uppercase equivalents are included as well
    MacOS,
    /// Any other operating system value, Should never hit this in theory, however there may be some odd games
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Parsed Half-Life mod type
pub enum ModType {
    /// Single and Multiplayer mod
    SingleAndMultiplayer,
    /// Multiplayer only mod
    MultiplayerOnly,
    /// Should only be one of the above options
    Invalid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Custom or standard Half-Life DLL for the mod
pub enum ModDLL {
    /// Mod uses the base Half-Life DLL
    HalfLife,
    /// MOD uses a custom DLL
    Custom,
    /// Any other response type should be invalid
    Invalid,
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
    /// Any other game mode should be invalid
    Invalid,
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
/// Data contained within an [GoldSource A2S_INFO Response](https://developer.valvesoftware.com/wiki/Server_queries#Obsolete_GoldSource_Response)
pub struct PreGoldSourceResponseInfo {
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

// # Public Parsers
/// Takes a slice of bytes and attempts to parse a PreGoldSource Server info response out of it
/// The parsing itself occurs withing p_goldsource_info, this just converts the IResult to a Result
pub fn parse_pregoldsource_info(input: &[u8]) -> Result<PreGoldSourceResponseInfo, Error<&[u8]>> {
    match p_pregoldsource_info(input).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}
/// Takes a slice of bytes and attempts to parse a Source info response out of it
/// The parsing itself occurs withing p_goldsource_info, this just converts the IResult to a Result
pub fn parse_source_info(input: &[u8]) -> Result<SourceResponseInfo, Error<&[u8]>> {
    match p_source_info(input).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}

// # Support Parsers
// Does the parsing for pregoldsource server info responses
fn p_pregoldsource_info(input: &[u8]) -> IResult<&[u8], PreGoldSourceResponseInfo> {
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

    let (input, mod_fields) = match mod_half_life {
        true => mod_fields(input)?,
        false => (input, None),
    };

    let (input, vac) = parse_bool(input)?;
    let (input, bots) = le_u8(input)?;

    // If the input is not empty there is extra data that shouldn't be there, raise a soft error so other parsers can be tried
    // if !input.is_empty() {
    //     return Err(nom::Err::Error(nom::error::Error {
    //         input,
    //         code: nom::error::ErrorKind::TooLarge,
    //     }));
    // }

    Ok((
        input,
        PreGoldSourceResponseInfo {
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

fn mod_fields(input: &[u8]) -> IResult<&[u8], Option<HalfLifeMod>> {
    let (input, link) = c_string(input)?;
    let (input, download_link) = c_string(input)?;
    let (input, _) = parse_null(input)?;
    let (input, version) = le_i32(input)?;
    let (input, size) = le_i32(input)?;
    let (input, mod_value) = le_u8(input)?;
    let (input, dll_value) = le_u8(input)?;

    let mod_type = match mod_value {
        0 => ModType::SingleAndMultiplayer,
        1 => ModType::MultiplayerOnly,
        _ => ModType::Invalid,
    };

    let dll = match dll_value {
        0 => ModDLL::HalfLife,
        1 => ModDLL::Custom,
        _ => ModDLL::Invalid,
    };

    // Make sure the type is not invalid and the dll is not invalid
    if mod_type == ModType::Invalid || dll == ModDLL::Invalid {
        return Err(nom::Err::Error(nom::error::Error {
            input,
            code: nom::error::ErrorKind::IsNot,
        }));
    }

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
}

/// Does the parsing for source info responses
fn p_source_info(input: &[u8]) -> IResult<&[u8], SourceResponseInfo> {
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

    // Only if the app_id matches on of The Ships ids should we try and parse ship data
    let (input, the_ship) = match app_id {
        // The Ship AppIds
        2400 | 2401 | 2402 | 2412 => the_ship(input)?,
        // The Ship Tutorial AppIds
        2430 | 2405 | 2406 => the_ship(input)?,
        // All other AppIds shouldn't have The Ship data
        _ => (input, None),
    };

    let (input, version) = c_string(input)?;

    // Optional, only is present when there is more data provided
    let (input, extra_data_flag) = opt_le_u8(input)?;
    // Unwrap, 0 means no data flags
    let extra_data_flag: u8 = extra_data_flag.unwrap_or(0);

    // Parse the extra data fields if the flag is not 0
    // if `EDF & 0x80` then the servers port is also transmitted
    let (input, port) = if extra_data_flag & 0x80 == 0x80 {
        le_i16(input).map(|(next, val)| (next, Some(val)))?
    } else {
        (input, None)
    };

    // if `EDF & 0x10` then servers steam ID is transmitted
    let (input, steam_id) = if extra_data_flag & 0x10 == 0x10 {
        le_u64(input).map(|(next, val)| (next, Some(val)))?
    } else {
        (input, None)
    };

    // if `EDF & 0x40` then the spectator port number and name of the spectator server for SourceTV are contained
    let (input, source_tv_port) = if extra_data_flag & 0x40 == 0x40 {
        le_i16(input).map(|(next, val)| (next, Some(val)))?
    } else {
        (input, None)
    };

    let (input, source_tv_name) = if extra_data_flag & 0x40 == 0x40 {
        c_string(input).map(|(next, val)| (next, Some(val)))?
    } else {
        (input, None)
    };

    // if `EDF & 0x20` then tags that describe the game are transmitted
    let (input, keywords) = if extra_data_flag & 0x20 == 0x20 {
        c_string(input).map(|(next, val)| (next, Some(val)))?
    } else {
        (input, None)
    };

    // if `EDF & 0x01` then the full game ID and untruncated App ID are contained.
    let (input, game_id) = if extra_data_flag & 0x01 == 0x01 {
        le_u64(input).map(|(next, val)| (next, Some(val)))?
    } else {
        (input, None)
    };

    //If the input is not empty there is extra data that shouldn't be there, raise a soft error so other parsers can be tried
    if !input.is_empty() {
        return Err(nom::Err::Error(nom::error::Error {
            input,
            code: nom::error::ErrorKind::TooLarge,
        }));
    }

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
            port,
            steam_id,
            source_tv_port,
            source_tv_name,
            keywords,
            game_id,
        },
    ))
}

fn the_ship(input: &[u8]) -> IResult<&[u8], Option<TheShipFields>> {
    let (input, mode_value) = le_u8(input)?;

    let mode = match mode_value {
        0 => TheShipGameMode::Hunt,
        1 => TheShipGameMode::Elimination,
        2 => TheShipGameMode::Duel,
        3 => TheShipGameMode::Deathmatch,
        4 => TheShipGameMode::VIP_Team,
        5 => TheShipGameMode::Team_Elimination,
        _ => TheShipGameMode::Invalid,
    };

    // Make sure the gamemode is not invalid
    if mode == TheShipGameMode::Invalid {
        return Err(nom::Err::Error(nom::error::Error {
            input,
            code: nom::error::ErrorKind::IsNot,
        }));
    }

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
}

/// Reads one byte from the input slice and returns the ServerType for goldsource
fn server_type(input: &[u8]) -> IResult<&[u8], ServerType> {
    let (input, value) = le_u8(input)?;
    let server_type = match value {
        // 'D' or 'd'
        0x44 => ServerType::Dedicated,
        0x64 => ServerType::Dedicated,
        // 'L' or 'l'
        0x4C => ServerType::NonDedicated,
        0x6C => ServerType::NonDedicated,
        // 'P' or 'p'
        0x50 => ServerType::SourceTV,
        0x70 => ServerType::SourceTV,
        // 0
        0x00 => ServerType::RagDollKungFu,
        // Any other value is invalid
        _ => ServerType::Invalid,
    };

    // recoverable error so that we can try alternative parsers if we desire later
    if server_type == ServerType::Invalid {
        return Err(nom::Err::Error(nom::error::Error {
            input,
            code: nom::error::ErrorKind::IsNot,
        }));
    }

    Ok((input, server_type))
}

/// Reads one byte from the input slice and returns the Environment
fn environment(input: &[u8]) -> IResult<&[u8], Environment> {
    let (input, value) = le_u8(input)?;

    let server_env = match value {
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
        _ => Environment::Other,
    };

    if server_env == Environment::Other {
        return Err(nom::Err::Error(nom::error::Error {
            input,
            code: nom::error::ErrorKind::IsNot,
        }));
    }

    Ok((input, server_env))
}

// # Tests

#[test]
fn info_garrysmod() {
    // Contains extra data flags
    let info_bytes = include_bytes!("../test_bytes/chaoticTTT.info");
    // Skip the first byte, its the header and is used to pick which parser is run
    let response = parse_source_info(&info_bytes[1..]).unwrap();

    assert_eq!(
        SourceResponseInfo {
            protocol: 0x11,
            name: "Chaotic TTT | Mostly Vanilla".to_string(),
            map: "ttt_submarine".to_string(),
            folder: "garrysmod".to_string(),
            game: "Trouble in Terrorist Town".to_string(),
            app_id: 4000,
            players: 0,
            max_players: 24,
            bots: 0,
            server_type: ServerType::Dedicated,
            environment: Environment::Windows,
            visibility: false,
            vac: true,
            the_ship: None,
            version: "2020.10.14".to_string(),

            extra_data_flag: 0xB1,
            port: Some(27035),
            steam_id: Some(0x1300000003A5F1E),
            source_tv_port: None,
            source_tv_name: None,
            keywords: Some(" gm:terrortown gmc:pvp loc:us ver:210402".to_string()),
            game_id: Some(4000)
        },
        response
    );
}

#[test]
fn info_cs16() {
    let info_bytes = include_bytes!("../test_bytes/cblaCS16.info");
    // Skip the first byte, its the header and is used to pick which parser is run
    let response = parse_source_info(&info_bytes[1..]).unwrap();

    assert_eq!(
        SourceResponseInfo {
            protocol: 48,
            name: "CBLA at Vimy CS 1.6".to_string(),
            map: "cs_italy".to_string(),
            folder: "cstrike".to_string(),
            game: "Counter-Strike".to_string(),
            app_id: 10,
            players: 9,
            max_players: 10,
            bots: 9,
            server_type: ServerType::Dedicated,
            environment: Environment::Linux,
            visibility: true,
            vac: true,
            the_ship: None,
            version: "1.1.2.7/Stdio".to_string(),

            extra_data_flag: 177,
            port: Some(27015),
            steam_id: Some(90147598802991112),
            source_tv_port: None,
            source_tv_name: None,
            keywords: Some("".to_string()),
            game_id: Some(10),
        },
        response
    );
}

#[test]
fn info_the_ship() {
    let info_bytes = include_bytes!("../test_bytes/mucosmosTheShip.info");

    let response = parse_source_info(&info_bytes[1..]).unwrap();

    assert_eq!(
        SourceResponseInfo {
            protocol: 7,
            name: "mucosmos.nl Pakjesboot 12 : Cruise ships : Votekick disabled".to_string(),
            map: "huronian".to_string(),
            folder: "ship".to_string(),
            game: "The Ship".to_string(),
            app_id: 2400,
            players: 5,
            max_players: 32,
            bots: 5,
            server_type: ServerType::Dedicated,
            environment: Environment::Windows,
            visibility: false,
            vac: false,
            the_ship: Some(TheShipFields {
                mode: TheShipGameMode::Hunt,
                witnesses: 2,
                duration: 3,
            }),
            version: "1.0.0.16".to_string(),

            extra_data_flag: 0,
            port: None,
            steam_id: None,
            source_tv_port: None,
            source_tv_name: None,
            keywords: None,
            game_id: None,
        },
        response
    );
}

#[test]
fn info_sourcetv() {
    let info_bytes = include_bytes!("../test_bytes/deathmatchTF2.info");

    let response = parse_source_info(&info_bytes[1..]).unwrap();

    assert_eq!(
        SourceResponseInfo {
            protocol: 47,
            name: "DeathMatchTF".to_string(),
            map: "koth_harvest_final".to_string(),
            folder: "tf".to_string(),
            game: "Team Fortress".to_string(),
            app_id: 440,
            players: 0,
            max_players: 128,
            bots: 0,
            server_type: ServerType::Dedicated,
            environment: Environment::Linux,
            visibility: false,
            vac: true,
            the_ship: None,
            version: "6394067".to_string(),

            extra_data_flag: 113,
            port: None,
            steam_id: Some(90147603733485569),
            source_tv_port: Some(27020),
            source_tv_name: Some("DeathMatchTF".to_string()),
            keywords: Some("alltalk,cp,deathmatch,deathmatchtf,dmtf,harvest,harvest1,increased_maxplayers,nocrits,norespawntime".to_string()),
            game_id: Some(440),
        },
        response
    );
}
