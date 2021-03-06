use super::*;

use nom::{number::complete::le_i32, IResult};

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

// # Parsing functions
pub fn p_info_request(input: &[u8]) -> IResult<&[u8], InfoRequest> {
    let (input, payload) = c_string(input)?;
    let (input, challenge) = le_i32(input)?;

    Ok((input, InfoRequest { payload, challenge }))
}

pub fn p_challenge_request(input: &[u8]) -> IResult<&[u8], ChallengeRequest> {
    let (input, challenge) = le_i32(input)?;

    Ok((input, ChallengeRequest { challenge }))
}

// TODO: Tests + Implementations
