use crate::parser_util::{c_string, opt_le_u8};

use nom::{
    combinator::all_consuming,
    error::Error,
    multi::many_m_n,
    number::complete::{le_f32, le_i32, le_u8},
    Finish, IResult,
};

// # Structs
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerResponse {
    pub players: u8,
    pub player_data: Vec<PlayerData
  ,
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

// Returns the player info or an error if the parsing failed or there was remaining data in the input
pub fn parse_player(input: &[u8]) -> Result<PlayerResponse, Error<&[u8]>> {
    match p_player(input).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}

// # Private parsing helper functions
// Makes sure that all of the input data was consumed, if not to much data was fed or something
pub fn p_player(input: &[u8]) -> IResult<&[u8], PlayerResponse> {
    all_consuming(player)(input)
}

// Does the bulk of the parsing
fn player(input: &[u8]) -> IResult<&[u8], PlayerResponse> {
    // If no players are connected a server can only transmit the header byte and no other data
    let (input, players) = opt_le_u8(input)?;

    let players = match players {
        Some(v) => v,
        None => {
            return Ok((
                input,
                ResponsePlayer {
                    players: 0,
                    player_data: Vec::new(),
                },
            ))
        }
    };

    let (input, mut player_data) = many_player_data(input, players)?;

    // The Ship adds fields after the regular player data
    let (input, ship_data) = many_the_ship_data(input, players)?;

    // If there is ship data, add it to already collected player data if they are the same length
    // TODO: If the length doesn't match the ship data invalid?
    if ship_data.len() == player_data.len() {
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
        PlayerResponse {
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
fn short_all_players_connected() {
    let player_bytes = include_bytes!("../test_bytes/cblaCS16.players");

    // Skip the header byte
    let players = parse_player(&player_bytes[1..]).unwrap();

    assert_eq!(6, players.players);
    assert_eq!(6, players.player_data.len());
    // Check the first and last players just to be sure
    assert_eq!(
        PlayerData {
            index: 0,
            name: "RAGE OF THE BOY".to_string(),
            score: 0,
            duration: 2447.0,
            ship_data: None
        },
        players.player_data[0]
    );
    assert_eq!(
        PlayerData {
            index: 0,
            name: "dslodolce".to_string(),
            score: 8,
            duration: 3697.25,
            ship_data: None
        },
        players.player_data[5]
    );
}

#[test]
fn short_the_ship() {
    let player_bytes = include_bytes!("../test_bytes/mucosmosTheShip.players");

    // Skip the header byte
    let players = parse_player(&player_bytes[1..]).unwrap();

    assert_eq!(5, players.players);
    assert_eq!(5, players.player_data.len());
    // Check the first and last players just to be sure
    assert_eq!(
        PlayerData {
            index: 0,
            name: "Shipmate1".to_string(),
            score: 1,
            duration: -1.0,
            ship_data: Some(TheShipData {
                deaths: 0,
                money: 4970
            })
        },
        players.player_data[0]
    );
    assert_eq!(
        PlayerData {
            index: 4,
            name: "Shipmate5".to_string(),
            score: 0,
            duration: -1.0,
            ship_data: Some(TheShipData {
                deaths: 0,
                money: 840
            })
        },
        players.player_data[4]
    );
}

#[test]
fn no_connected_players() {
    let player_bytes = include_bytes!("../test_bytes/chaoticTTT.players");

    // Skip the header byte
    let players = parse_player(&player_bytes[1..]).unwrap();

    assert_eq!(0, players.players);
    assert_eq!(0, players.player_data.len());
}

#[test]
fn connecting_player() {
    let player_bytes = include_bytes!("../test_bytes/deathmatchTF2.players");

    // Skip the header byte
    // Stop before the actual end of the player list to simulate a player not being fully connected
    let players = parse_player(&player_bytes[1..213]).unwrap();

    assert_eq!(11, players.players);
    assert_eq!(10, players.player_data.len());

    println!("{:?}", players);
    // Check the first and last players just to be sure
    assert_eq!(
        PlayerData {
            index: 0,
            name: "chinosdjjeej".to_string(),
            score: 43,
            duration: 4219.23,
            ship_data: None
        },
        players.player_data[0]
    );
    assert_eq!(
        PlayerData {
            index: 0,
            name: "poot".to_string(),
            score: 6,
            duration: 656.1299,
            ship_data: None
        },
        players.player_data[9]
    );
}

#[test]
fn extra_data_after_players() {
    let mut player_bytes = include_bytes!("../test_bytes/cblaCS16.players").to_vec();
    player_bytes.extend(&[0xFF, 0xFF, 0xFF]);

    // Skip the header byte
    let players = parse_player(&player_bytes[1..]).unwrap_err();

    let error = nom::error::Error::new(&[0xFF, 0xFF, 0xFF][..], nom::error::ErrorKind::Eof);

    assert_eq!(error, players);
    
}
