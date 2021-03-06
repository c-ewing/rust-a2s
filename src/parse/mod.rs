mod source_info;
use crate::parse::source_info::p_source_info;

mod goldsource_info;
use crate::parse::goldsource_info::p_goldsource_info;

mod requests;
use crate::parse::requests::{p_challenge_request, p_info_request, ChallengeRequest, InfoRequest};

mod player;
use crate::parse::player::{p_player, ResponsePlayer};

mod rules;
use crate::parse::rules::{p_rule, ResponseRule};

mod ping;
use crate::parse::ping::p_ping;

mod packet;
use crate::parse::packet::{GoldsourceMultiPacket, SourceMultiPacket, p_goldsource_multi_packet, p_source_multi_packet};

use nom::{
    bytes::complete::take_till,
    character::complete::char,
    combinator::{all_consuming, opt},
    error::Error,
    number::complete::{le_i32, le_u8},
    sequence::terminated,
    Finish, IResult,
};



// # Structs / Enums
pub enum Header {
    InfoRequest,
    InfoResponseSource,
    InfoResponseGoldSource,
    PlayerRequest,
    PlayerResponse,
    RulesRequest,
    RulesResponse,
    PingRequest,
    PingResponse,
    ChallengeRequest,
    ChallengeResponse,
    Other(u8),
}

impl From<u8> for Header {
    fn from(input: u8) -> Self {
        match input {
            // 'T'
            0x54 => Header::InfoRequest,
            // 'I'
            0x49 => Header::InfoResponseSource,
            // 'm'
            0x6D => Header::InfoResponseGoldSource,
            // 'U'
            0x55 => Header::PlayerRequest,
            // 'D'
            0x44 => Header::PlayerResponse,
            // 'V'
            0x56 => Header::RulesRequest,
            // 'E'
            0x45 => Header::RulesResponse,
            // 'i'
            0x69 => Header::PingRequest,
            // 'j'
            0x6A => Header::PingResponse,
            // 'W'
            0x57 => Header::ChallengeRequest,
            // 'A'
            0x41 => Header::ChallengeResponse,
            // All other values don't correspond to anything according to the wiki
            _ => Header::Other(input),
        }
    }
}

// # Exposed parsing functions:
// # Parse whole packets
pub fn parse_goldsource_packet(input: &[u8]) -> Result<GoldsourceMultiPacket, Error<&[u8]>> {
    match p_goldsource_multi_packet(input).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}

pub fn parse_source_packet(input: &[u8]) -> Result<SourceMultiPacket, Error<&[u8]>> {
    match p_source_multi_packet(input).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}


// # Parse single and combined payloads
// Returns the info or an error if the parsing failed or there was remaining data in the input
pub fn parse_source_info(input: &[u8]) -> Result<source_info::ResponseInfo, Error<&[u8]>> {
    match p_source_info(input).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}

// Returns the info or an error if the parsing failed or there was remaining data in the input
pub fn parse_goldsource_info(input: &[u8]) -> Result<goldsource_info::ResponseInfo, Error<&[u8]>> {
    match p_goldsource_info(input).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}

// Returns the player info or an error if the parsing failed or there was remaining data in the input
pub fn parse_player(input: &[u8]) -> Result<ResponsePlayer, Error<&[u8]>> {
    match p_player(input).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}

// Returns the player info or an error if the parsing failed.
// Remaining data in the input is not considered failure as old servers truncated data to one packet,
// this data is contained in the remaining_data field.
pub fn parse_rule(input: &[u8]) -> Result<ResponseRule, Error<&[u8]>> {
    match p_rule(input).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}

// Returns the ping response payload or an error if the parsing failed or there was remaining data in the input
pub fn parse_ping(input: &[u8]) -> Result<String, Error<&[u8]>> {
    match p_ping(input).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}

// # Added Parsing requests for completeness, only challenge request is likely to be used
// Info may have additional info after the defined fields so it is also returned
// TODO: take a look at these once full match parsing implemented
pub fn parse_info_request(input: &[u8]) -> Result<(&[u8], InfoRequest), Error<&[u8]>> {
    p_info_request(input).finish()
}

pub fn parse_player_request(input: &[u8]) -> Result<ChallengeRequest, Error<&[u8]>> {
    match p_challenge_request(input).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}

// # General Helper functions used across several parsers

// take all character until null term and discard null term
fn c_string(input: &[u8]) -> IResult<&[u8], String> {
    terminated(take_till(|c| c == 0x00u8), char(0x00 as char))(input)
        .map(|(next, res)| (next, String::from_utf8_lossy(res).into_owned()))
}

// Optionally returns the challenge value if it was provided
fn opt_le_u8(input: &[u8]) -> IResult<&[u8], Option<u8>> {
    opt(le_u8)(input)
}

fn null(input: &[u8]) -> IResult<&[u8], char> {
    char(0x00 as char)(input)
}

fn bool(input: &[u8]) -> IResult<&[u8], bool> {
    le_u8(input).map(|(next, res)| (next, res != 0))
}
