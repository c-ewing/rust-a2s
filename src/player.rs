use crate::parser_util::c_string;

use nom::{
    combinator::all_consuming,
    error::Error,
    multi::many_m_n,
    number::complete::{le_f32, le_i32, le_u8},
    Finish, IResult,
};

// # Structs
#[derive(Clone, Debug, PartialEq)]
pub struct ResponsePlayer {
    pub players: u8,
    pub player_data: Vec<PlayerData>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerData {
    pub index: u8,
    pub name: String,
    pub score: i32,
    pub duration: f32,
    // The ship is special and sends data after the standard fields
    pub ship_data: Option<TheShipData>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct TheShipData {
    pub deaths: i32,
    pub money: i32,
}

// # Exposed final parser
// TODO: comment better
// Returns the player info or an error if the parsing failed or there was remaining data in the input
pub fn parse_player(input: &[u8]) -> Result<ResponsePlayer, Error<&[u8]>> {
    match p_player(input).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}

// # Private parsing helper functions
// Makes sure that all of the input data was consumed, if not to much data was fed or something
pub fn p_player(input: &[u8]) -> IResult<&[u8], ResponsePlayer> {
    all_consuming(player)(input)
}

// Does the bulk of the parsing
fn player(input: &[u8]) -> IResult<&[u8], ResponsePlayer> {
    let (input, players) = le_u8(input)?;
    let (input, mut player_data) = many_player_data(input, players)?;

    // The Ship adds fields after the regular player data
    let (input, ship_data) = many_the_ship_data(input, players)?;

    // If there is ship data, add it to already collected player data
    if !ship_data.is_empty() {
        // Iterate over the mutable player data pair with the associated ship data and replace the default
        // None in the player data with a copy of the ship data
        player_data
            .iter_mut()
            .zip(ship_data.iter())
            .for_each(|pair| {
                pair.0.ship_data = Some(pair.1.to_owned());
            });
    }

    Ok((
        input,
        ResponsePlayer {
            players,
            player_data,
        },
    ))
}

// Uses many_m_n over count as connecting players are included in the players count but no data is stored.
fn many_player_data(input: &[u8], player_count: u8) -> IResult<&[u8], Vec<PlayerData>> {
    many_m_n(0, player_count as usize, player_data)(input)
}

fn player_data(input: &[u8]) -> IResult<&[u8], PlayerData> {
    let (input, index) = le_u8(input)?;
    let (input, name) = c_string(input)?;
    let (input, score) = le_i32(input)?;
    let (input, duration) = le_f32(input)?;

    Ok((
        input,
        PlayerData {
            index,
            name,
            score,
            duration,
            ship_data: None,
        },
    ))
}

fn many_the_ship_data(input: &[u8], players: u8) -> IResult<&[u8], Vec<TheShipData>> {
    many_m_n(0, players as usize, ship_data)(input)
}

fn ship_data(input: &[u8]) -> IResult<&[u8], TheShipData> {
    let (input, deaths) = le_i32(input)?;
    let (input, money) = le_i32(input)?;

    Ok((input, TheShipData { deaths, money }))
}

// # Test
#[test]
fn two_player() {
    // Packet from souce wiki
    // Omitts first 5 bytes as parse_player assumes the packet data has been combined and the message type determined
    let player: [u8; 49] = [
        0x02, 0x01, 0x5B, 0x44, 0x5D, 0x2D, 0x2D, 0x2D, 0x2D, 0x3E, 0x54, 0x2E, 0x4E, 0x2E, 0x57,
        0x3C, 0x2D, 0x2D, 0x2D, 0x2D, 0x00, 0x0E, 0x00, 0x00, 0x00, 0xB4, 0x97, 0x00, 0x44, 0x02,
        0x4B, 0x69, 0x6C, 0x6C, 0x65, 0x72, 0x20, 0x21, 0x21, 0x21, 0x00, 0x05, 0x00, 0x00, 0x00,
        0x69, 0x24, 0xD9, 0x43,
    ];

    let response = parse_player(&player).unwrap();

    let expected_players = vec![
        PlayerData {
            index: 1,
            name: "[D]---->T.N.W<----".to_string(),
            score: 14,
            duration: 514.37036f32,
            ship_data: None,
        },
        PlayerData {
            index: 2,
            name: "Killer !!!".to_string(),
            score: 5,
            duration: 434.28445f32,
            ship_data: None,
        },
    ];

    assert_eq!(2, response.players);
    assert_eq!(expected_players, response.player_data)
}

#[test]
fn connecting_player() {
    // Packet from souce wiki
    // Omitts first 5 bytes as parse_player assumes the packet data has been combined and the message type determined
    let player: [u8; 29] = [
        0x02, 0x01, 0x5B, 0x44, 0x5D, 0x2D, 0x2D, 0x2D, 0x2D, 0x3E, 0x54, 0x2E, 0x4E, 0x2E, 0x57,
        0x3C, 0x2D, 0x2D, 0x2D, 0x2D, 0x00, 0x0E, 0x00, 0x00, 0x00, 0xB4, 0x97, 0x00, 0x44,
    ];

    let response = parse_player(&player).unwrap();

    let expected_player = vec![PlayerData {
        index: 1,
        name: "[D]---->T.N.W<----".to_string(),
        score: 14,
        duration: 514.37036f32,
        ship_data: None,
    }];

    assert_eq!(2, response.players);
    assert_eq!(expected_player, response.player_data);
}

#[test]
fn the_ship_player_data() {
    // Packet from souce wiki
    // Omitts first 5 bytes as parse_player assumes the packet data has been combined and the message type determined
    let the_ship_players: [u8; 167] = [
        0x06, 0x00, 0x53, 0x68, 0x69, 0x70, 0x6D, 0x61, 0x74, 0x65, 0x31, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x80, 0xBF, 0x01, 0x53, 0x68, 0x69, 0x70, 0x6D, 0x61, 0x74, 0x65, 0x32,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0xBF, 0x02, 0x53, 0x68, 0x69, 0x70, 0x6D,
        0x61, 0x74, 0x65, 0x33, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0xBF, 0x03, 0x53,
        0x68, 0x69, 0x70, 0x6D, 0x61, 0x74, 0x65, 0x34, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x80, 0xBF, 0x04, 0x53, 0x68, 0x69, 0x70, 0x6D, 0x61, 0x74, 0x65, 0x35, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x80, 0xBF, 0x07, 0x28, 0x31, 0x29, 0x4C, 0x61, 0x6E, 0x64, 0x4C,
        0x75, 0x62, 0x62, 0x65, 0x72, 0x00, 0x00, 0x00, 0x00, 0x00, 0xD3, 0x8E, 0x68, 0x45, 0x00,
        0x00, 0x00, 0x00, 0xC4, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC4, 0x09, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0xC4, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC4, 0x09, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0xC4, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC4, 0x09,
        0x00, 0x00,
    ];

    let response = parse_player(&the_ship_players).unwrap();

    let default_ship_data = Some(TheShipData {
        deaths: 0,
        money: 2500,
    });

    let expected_players = vec![
        PlayerData {
            index: 0,
            name: "Shipmate1".to_string(),
            score: 0,
            duration: -1.0,
            ship_data: default_ship_data.to_owned(),
        },
        PlayerData {
            index: 1,
            name: "Shipmate2".to_string(),
            score: 0,
            duration: -1.0,
            ship_data: default_ship_data.to_owned(),
        },
        PlayerData {
            index: 2,
            name: "Shipmate3".to_string(),
            score: 0,
            duration: -1.0,
            ship_data: default_ship_data.to_owned(),
        },
        PlayerData {
            index: 3,
            name: "Shipmate4".to_string(),
            score: 0,
            duration: -1.0,
            ship_data: default_ship_data.to_owned(),
        },
        PlayerData {
            index: 4,
            name: "Shipmate5".to_string(),
            score: 0,
            duration: -1.0,
            ship_data: default_ship_data.to_owned(),
        },
        PlayerData {
            index: 7,
            name: "(1)LandLubber".to_string(),
            score: 0,
            duration: 3720.9265,
            ship_data: default_ship_data.to_owned(),
        },
    ];

    assert_eq!(6, response.players);
    assert_eq!(expected_players, response.player_data);
}
