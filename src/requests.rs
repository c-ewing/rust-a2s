// # Imports

use std::u8;

use nom::{combinator::rest, error::Error, number::complete::le_i32, Finish, IResult};

use crate::parser_util::{c_string, opt_le_i32};

// TODO: These will be handled one parser level up, they only have a header
// PING (DEP): 0x69 : Depreciated
// SERVERQUERY_GETCHALLENGE (DEP): 0x57 : Depricated

// # Structs

#[derive(Clone, Debug, PartialEq, Eq)]
/// Data contained within a request packet for [A2S_INFO](https://developer.valvesoftware.com/wiki/Server_queries#Request_Format)
pub struct InfoRequest {
    /// Payload string of the request, should normally be `Source Engine Query\0` (null ommited)
    pub payload: String,
    /// Challenge value of the request if present, -1 to attempt without challenge value or to request one. If there is a challenge response
    /// packet sent (0x41, 'A' header) resend request with the challenge value
    pub challenge: Option<i32>,
    /// Any extra data provided along with the payload. Future games may place/require data here so it is included
    pub remaining: Vec<u8>,
}
// All but the info request are generic in just having a header and a challenge value
#[derive(Clone, Debug, PartialEq, Eq)]
/// Data contained within a request packet for [A2S_PLAYER](https://developer.valvesoftware.com/wiki/Server_queries#Request_Format_2) and [A2S_RULES](https://developer.valvesoftware.com/wiki/Server_queries#Request_Format_3)
pub struct ChallengeRequest {
    /// Challenge value of the request, -1 to attempt without challenge value or to request one. If there is a challenge response
    /// packet sent (0x41, 'A' header) resend request with the challenge value
    challenge: i32,
}
// # Public Parsers

/**
Attempts to parse the provided slice into a info request
Later source engine games and certain updated older games require a challenge value before sending the response to prevent a [reflection attack](https://developer.valvesoftware.com/wiki/Server_queries#Request_Format)
An initial request without a challenge can recieve a challenge response packet containing a challege value to be appened to the request.

# Errors
The payload value is expected to always be `Source Engine Query`, if it is not a ErrorKind::Satisfy is returned
Any other [`nom::error::Error`](https://docs.rs/nom/6.1.2/nom/error/struct.Error.html) results if the parse fails to find the correct format

# Examples

Parsing of a Source Info Request wtihout challenge
```
use a2s_parse::requests::{parse_info_request, InfoRequest};

// Payload assuming header and packet type indicators are stripped and no challenge is offered
// Payload of "Source Engine Query\0"
let payload: [u8; 20] = [0x53, 0x6f, 0x75, 0x72, 0x63, 0x65, 0x20, 0x45, 0x6e, 0x67, 0x69, 0x6e, 0x65, 0x20, 0x51, 0x75, 0x65, 0x72, 0x79, 0x00];

let request = parse_info_request(&payload).unwrap();

assert_eq!(
    InfoRequest {
        payload: "Source Engine Query".to_string(),
        challenge: None,
        remaining: Vec::new()
    },
    request)
```
*/

/// Attempt to parse an [InfoRequest] out of the provided slice
/// Returns an error if the parse fails or if the payload does not match `Source Engine Query`
pub fn parse_info_request(input: &[u8]) -> Result<InfoRequest, Error<&[u8]>> {
    match p_info_request(input).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}

/// Attempt to parse a [ChallengeRequest] out of the provided slice
/// The players has no extra data other than the challenge value
pub fn parse_players_request(input: &[u8]) -> Result<ChallengeRequest, Error<&[u8]>> {
    match p_challenge_request(input).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}

/// Attempt to parse a [ChallengeRequest] out of the provided slice
/// The rules has no extra data other than the challenge value
/// Raises a
pub fn parse_rules_request(input: &[u8]) -> Result<ChallengeRequest, Error<&[u8]>> {
    match p_challenge_request(input).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}

// # Private helpers

/// Perform the parse attempt for info requests
/// Raises an error if the parse fails or a ErrorKind::Satisfy if the payload does not match the expected value
fn p_info_request(input: &[u8]) -> IResult<&[u8], InfoRequest> {
    let (input, payload) = c_string(input)?;

    if payload != "Source Engine Query" {
        return Err(nom::Err::Error(nom::error::Error {
            input,
            code: nom::error::ErrorKind::Satisfy,
        }));
    }

    let (input, challenge) = opt_le_i32(input)?;
    let (input, remaining) = rest(input)?;

    Ok((
        input,
        InfoRequest {
            payload,
            challenge,
            remaining: remaining.to_vec(),
        },
    ))
}

/// Perform the parse attempt for all basic requests with only the challenge value
/// Raises an error if the parse fails or a ErrorKind::TooLarge if there is data after the challenege value
fn p_challenge_request(input: &[u8]) -> IResult<&[u8], ChallengeRequest> {
    let (input, challenge) = le_i32(input)?;

    // If the input is not empty there is extra data that shouldn't be there, raise a soft error so other parsers can be tried
    if !input.is_empty() {
        return Err(nom::Err::Error(nom::error::Error {
            input,
            code: nom::error::ErrorKind::TooLarge,
        }));
    }

    Ok((input, ChallengeRequest { challenge }))
}

// # Tests

#[test]
fn request_info() {
    let request_bytes = include_bytes!("../test_bytes/chaoticTTT.requestinfo");

    // Skip the first byte as the file still has the header value
    let request = parse_info_request(&request_bytes[1..]).unwrap();

    assert_eq!(
        InfoRequest {
            payload: "Source Engine Query".to_string(),
            challenge: None,
            remaining: Vec::new()
        },
        request
    )
}

#[test]
fn request_info_with_challenge() {
    let request_bytes = include_bytes!("../test_bytes/chaoticTTT.requestinfo");
    // Add a challenge value to the request
    let mut request_bytes = request_bytes.to_vec();
    request_bytes.extend(&[0xFF, 0xFF, 0xFF, 0xFF]);

    // Skip the first byte as the file still has the header value
    let request = parse_info_request(&request_bytes[1..]).unwrap();

    assert_eq!(
        InfoRequest {
            payload: "Source Engine Query".to_string(),
            challenge: Some(-1),
            remaining: Vec::new()
        },
        request
    )
}

#[test]
fn request_info_with_extra_data() {
    let request_bytes = include_bytes!("../test_bytes/chaoticTTT.requestinfo");
    // Add a extra data to the request
    let mut request_bytes = request_bytes.to_vec();
    request_bytes.extend(&[0xFF, 0xFF, 0xFF]);

    // Skip the first byte as the file still has the header value
    let request = parse_info_request(&request_bytes[1..]).unwrap();

    assert_eq!(
        InfoRequest {
            payload: "Source Engine Query".to_string(),
            challenge: None,
            remaining: vec![0xFF, 0xFF, 0xFF],
        },
        request
    )
}

#[test]
fn request_players() {
    let request_bytes = include_bytes!("../test_bytes/chaoticTTT.requestplayers");

    // Skip the first byte as the file still has the header value
    let request = parse_players_request(&request_bytes[1..]).unwrap();

    assert_eq!(
        ChallengeRequest {
            challenge: -1852284646,
        },
        request
    )
}

#[test]
fn request_rules() {
    let request_bytes = include_bytes!("../test_bytes/chaoticTTT.requestrules");

    // Skip the first byte as the file still has the header value
    let request = parse_rules_request(&request_bytes[1..]).unwrap();

    assert_eq!(
        ChallengeRequest {
            challenge: -2101649440,
        },
        request
    )
}

#[test]
fn request_players_with_extra_data() {
    // This covers all functions that call p_challenge_request
    let request_bytes = include_bytes!("../test_bytes/chaoticTTT.requestplayers");
    // Add a extra data to the request
    let mut request_bytes = request_bytes.to_vec();
    request_bytes.extend(&[0xFF, 0xFF, 0xFF]);

    // Skip the first byte as the file still has the header value
    let request_error = parse_players_request(&request_bytes[1..]).unwrap_err();

    let error = nom::error::Error::new(&[0xFF, 0xFF, 0xFF][..], nom::error::ErrorKind::TooLarge);

    assert_eq!(error, request_error)
}
