use nom::{error::Error, number::complete::le_i32, Finish, IResult};

use crate::parser_util::c_string;

// TODO:

// # Structs
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InfoRequest {
    pub payload: String,
    pub challenge: i32,
}
// All but the info request are generic in just having a header and a challenge value
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChallengeRequest {
    challenge: i32,
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

// # Parsing functions
fn p_info_request(input: &[u8]) -> IResult<&[u8], InfoRequest> {
    let (input, payload) = c_string(input)?;
    let (input, challenge) = le_i32(input)?;

    Ok((input, InfoRequest { payload, challenge }))
}

fn p_challenge_request(input: &[u8]) -> IResult<&[u8], ChallengeRequest> {
    let (input, challenge) = le_i32(input)?;

    Ok((input, ChallengeRequest { challenge }))
}

// TODO: Tests + Implementations
