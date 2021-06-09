use nom::{combinator::all_consuming, error::Error, Finish, IResult};

use crate::parser_util::c_string;

// # Public parser
/**
Attempts to parse the provided slice into a ping response
# Warning: Depreciated according to the wiki
[Wiki Page](https://developer.valvesoftware.com/wiki/Server_queries#A2A_PING)

Source servers respond with `00000000000000\0`, while Gold Source servers respond with `\0`.
The null value is dropped in the returned String.
Any other response should be considered invalid.

# Errors
A [`nom::error::Error`](https://docs.rs/nom/6.1.2/nom/error/struct.Error.html) results if the parse fails for any reason

# Examples

Parsing of a Source server response
```
use a2s_parse::ping::parse_ping;
// Payload omitts first 5 bytes as parse_player assumes the packet and payload type have been determined
let payload: [u8; 15] = [
    0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x00,
];

let response = parse_ping(&payload).unwrap();

assert_eq!("00000000000000".to_string(), response);
```
 */

pub fn parse_ping(input: &[u8]) -> Result<String, Error<&[u8]>> {
    match p_ping(input).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}

// # Private parsing helper functions
/// Make sure that all of the input data was consumed. If it is not the response should be considered invalid as the
/// spec lists only a C style string as the response.
fn p_ping(input: &[u8]) -> IResult<&[u8], String> {
    all_consuming(c_string)(input)
}

// # Tests
#[test]
fn goldsource_response() {
    // Omitts first 5 bytes as parse_ping assumes the packet data has been combined and the message type determined
    let payload: [u8; 1] = [0x00];

    let response = parse_ping(&payload).unwrap();

    assert_eq!("".to_string(), response);
}

#[test]
fn source_response() {}

#[test]
fn no_payload() {
    // Omitts first 5 bytes as parse_ping assumes the packet data has been combined and the message type determined
    let payload: [u8; 0] = [];

    // using [..] transforms it into a slice
    let response = parse_ping(&payload[..]).unwrap_err();
    let error = nom::error::Error::new(&payload[..], nom::error::ErrorKind::Char);
    assert_eq!(error, response);
}

#[test]
fn extra_payload() {
    // Omitts first 5 bytes as parse_player assumes the packet data has been combined and the message type determined
    let payload: [u8; 16] = [
        0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x00,
        0x30,
    ];

    let response = parse_ping(&payload).unwrap_err();
    // [..1] tricks it into being a slice
    let error = nom::error::Error::new(&payload[..1], nom::error::ErrorKind::Eof);

    assert_eq!(error, response);
}
