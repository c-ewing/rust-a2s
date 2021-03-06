use super::*;

use nom::IResult;

// # Exposed final parser
// Makes sure that all of the input data was consumed, if not to much data was fed or something
// TODO: comment better
pub fn p_ping(input: &[u8]) -> IResult<&[u8], String> {
    all_consuming(ping)(input)
}

// # Private parsing helper functions
// Does the bulk of the parsing
fn ping(input: &[u8]) -> IResult<&[u8], String> {
    let (input, response) = c_string(input)?;

    Ok((input, response))
}

// # Test
#[test]
fn goldsource_response() {
    // Packet from souce wiki
    // Omitts first 5 bytes as parse_player assumes the packet data has been combined and the message type determined
    let response: [u8; 1] = [0x00];

    let response = parse_ping(&response).unwrap();

    assert_eq!("".to_string(), response);
}

#[test]
fn source_response() {
    // Packet from souce wiki
    // Omitts first 5 bytes as parse_player assumes the packet data has been combined and the message type determined
    let response: [u8; 15] = [
        0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x00,
    ];

    let response = parse_ping(&response).unwrap();

    assert_eq!("00000000000000".to_string(), response);
}
