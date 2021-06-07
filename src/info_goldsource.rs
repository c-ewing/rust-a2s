use nom::{
    combinator::all_consuming,
    error::Error,
    number::complete::{le_i32, le_u8},
    Finish, IResult,
};

use crate::parser_util::{
    c_string, environment, parse_bool, parse_null, server_type, Environment, ServerType,
};

// # Structs
#[derive(Clone, Debug, PartialEq, Eq)]
/// Data contained within an [A2S_INFO Response](https://developer.valvesoftware.com/wiki/Server_queries#Obsolete_GoldSource_Response) for Goldsource
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

// # Exposed final parser
// TODO: comment better
// Returns the info or an error if the parsing failed or there was remaining data in the input
// Remaining data in the input is not considered failure as old servers truncated data to one packet,

pub fn parse_goldsource_info(input: &[u8]) -> Result<GoldSourceResponseInfo, Error<&[u8]>> {
    match p_goldsource_info(input).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}

// # Private parsing helper functions
// Make sure the parser ate all the data
// TODO: move into main parsing function
fn p_goldsource_info(input: &[u8]) -> IResult<&[u8], GoldSourceResponseInfo> {
    all_consuming(goldsource_info)(input)
}

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
