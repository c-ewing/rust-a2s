use nom::{
    combinator::rest, error::Error, multi::many_m_n, number::complete::le_i16, Finish, IResult,
};

use crate::parser_util::c_string;

// # Structs
#[derive(Clone, Debug, PartialEq, Eq)]
/// Contains the data specified in an [`A2S_RULES response`](https://developer.valvesoftware.com/wiki/Server_queries#Response_Format_3)  
/// Older games / engines may respond with a single packet response that truncates the rules somewhere in a rule : value pair.
/// This truncated data is retained withing the remaining data field.
pub struct RulesResponse {
    /// Maximum number of rules contained within the response payload.
    pub num_rules: i16,
    /// Vec containing all the parsed rules : values pairs
    pub rules: Vec<RuleData>,
    /// Any data left over after attempting to parse the rules. This is not a hard error
    /// as some engine versions truncated rule data do a single packet instead of sending multiple packets
    pub remaining_data: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
/// Pairs of rules : values
pub struct RuleData {
    /// Rule name
    pub name: String,
    /// Value
    pub value: String,
}

// # Exposed final parser
/// Parse the data specified in an [`A2S_RULES response`](https://developer.valvesoftware.com/wiki/Server_queries#Response_Format_3)  
/// Older games / engines may respond with a single packet response that truncates the rules somewhere in a rule : value pair.
/// This truncated data is retained withing the remaining data field.

/// TODO: If there is remaining data after parsing the correct number of rules then raise an error
pub fn parse_rules(input: &[u8]) -> Result<RulesResponse, Error<&[u8]>> {
    match p_rules(input).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}

// # Private parsing helper functions

/// Does the parsing
fn rules(input: &[u8]) -> IResult<&[u8], RulesResponse> {
    let (input, num_rules) = le_i16(input)?;
    // Parse up to num_rules rules from the payload
    let (input, rule_data) = many_rule_data(input, num_rules)?;
    // Grab the rest of the input, this clears input for us so we don't have to after the match
    // This is done to satisfy the all_consuming
    let (input, remaining_data) = rest(input)?;

    let remaining_data = String::from_utf8_lossy(remaining_data).into_owned();

    if rule_data.len() as i16 == num_rules && !remaining_data.is_empty() {
        return Err(nom::Err::Error(Error::new(
            input,
            nom::error::ErrorKind::NonEmpty,
        )));
    }

    Ok((
        input,
        RulesResponse {
            rules: num_rules,
            rule_data,

            remaining_data,
        },
    ))
}

// Uses many_m_n over count as rules could be truncated
fn many_rule_data(input: &[u8], rules: i16) -> IResult<&[u8], Vec<RuleData>> {
    many_m_n(0, rules as usize, rule_data)(input)
}

fn rule_data(input: &[u8]) -> IResult<&[u8], RuleData> {
    let (input, name) = c_string(input)?;
    let (input, value) = c_string(input)?;

    Ok((input, RuleData { name, value }))
}

// # Tests
// TODO: get the response from ('66.55.158.156', 27015) for rules --> cblaCS16.rules, its breaking python_A2s
// TODO: find a game with a truncated response

#[test]
fn short_rules_the_ship() {
    let rule_bytes = include_bytes!("../test_bytes/mucosmosTheShip.rules");

    // Skip the header byte
    let rules = parse_rules(&rule_bytes[1..]).unwrap();

    // Checking if the rules match exactly would be ugly, there are 56 of them
    assert_eq!(56, rules.num_rules);
    assert_eq!(56, rules.rules.len());
    // Check the first and last rule just to be sure
    assert_eq!(
        RuleData {
            name: "sm_sourcesleuth_version".to_string(),
            value: "1.6.3".to_string()
        },
        rules.rules[0]
    );
    assert_eq!(
        RuleData {
            name: "sv_password".to_string(),
            value: "0".to_string()
        },
        rules.rules[55]
    );
}

#[test]
fn long_rules_tf2() {
    let rule_bytes = include_bytes!("../test_bytes/deathmatchTF2.rules");

    // Skip the extra packet type and header byte (multipacket sometimes adds an extra header)
    let rules = parse_rules(&rule_bytes[5..]).unwrap();

    // Checking if the rules match exactly would be ugly, there are 212 of them
    assert_eq!(212, rules.num_rules);
    assert_eq!(212, rules.rules.len());
    // Check the first and last rule just to be sure
    assert_eq!(
        RuleData {
            name: "coop".to_string(),
            value: "0".to_string()
        },
        rules.rules[0]
    );
    assert_eq!(
        RuleData {
            name: "tv_relaypassword".to_string(),
            value: "0".to_string()
        },
        rules.rules[211]
    );
}

#[test]
fn long_truncated_rules() {
    // Packet from souce wiki
    // TODO: Put in include_bytes!() file
    // Omitts first 5 bytes as parse_player assumes the packet data has been combined and the message type determined
    let long_rules: [u8; 1386] = [
        0x5D, 0x00, 0x5F, 0x74, 0x75, 0x74, 0x6F, 0x72, 0x5F, 0x62, 0x6F, 0x6D, 0x62, 0x5F, 0x76,
        0x69, 0x65, 0x77, 0x61, 0x62, 0x6C, 0x65, 0x5F, 0x63, 0x68, 0x65, 0x63, 0x6B, 0x5F, 0x69,
        0x6E, 0x74, 0x65, 0x72, 0x76, 0x61, 0x6C, 0x00, 0x30, 0x2E, 0x35, 0x00, 0x5F, 0x74, 0x75,
        0x74, 0x6F, 0x72, 0x5F, 0x64, 0x65, 0x62, 0x75, 0x67, 0x5F, 0x6C, 0x65, 0x76, 0x65, 0x6C,
        0x00, 0x30, 0x00, 0x5F, 0x74, 0x75, 0x74, 0x6F, 0x72, 0x5F, 0x65, 0x78, 0x61, 0x6D, 0x69,
        0x6E, 0x65, 0x5F, 0x74, 0x69, 0x6D, 0x65, 0x00, 0x30, 0x2E, 0x35, 0x00, 0x5F, 0x74, 0x75,
        0x74, 0x6F, 0x72, 0x5F, 0x68, 0x69, 0x6E, 0x74, 0x5F, 0x69, 0x6E, 0x74, 0x65, 0x72, 0x76,
        0x61, 0x6C, 0x5F, 0x74, 0x69, 0x6D, 0x65, 0x00, 0x31, 0x30, 0x2E, 0x30, 0x00, 0x5F, 0x74,
        0x75, 0x74, 0x6F, 0x72, 0x5F, 0x6C, 0x6F, 0x6F, 0x6B, 0x5F, 0x61, 0x6E, 0x67, 0x6C, 0x65,
        0x00, 0x31, 0x30, 0x00, 0x5F, 0x74, 0x75, 0x74, 0x6F, 0x72, 0x5F, 0x6C, 0x6F, 0x6F, 0x6B,
        0x5F, 0x64, 0x69, 0x73, 0x74, 0x61, 0x6E, 0x63, 0x65, 0x00, 0x32, 0x30, 0x30, 0x00, 0x5F,
        0x74, 0x75, 0x74, 0x6F, 0x72, 0x5F, 0x6D, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x5F, 0x63,
        0x68, 0x61, 0x72, 0x61, 0x63, 0x74, 0x65, 0x72, 0x5F, 0x64, 0x69, 0x73, 0x70, 0x6C, 0x61,
        0x79, 0x5F, 0x74, 0x69, 0x6D, 0x65, 0x5F, 0x63, 0x6F, 0x65, 0x66, 0x66, 0x69, 0x63, 0x69,
        0x65, 0x6E, 0x74, 0x00, 0x30, 0x2E, 0x30, 0x37, 0x00, 0x5F, 0x74, 0x75, 0x74, 0x6F, 0x72,
        0x5F, 0x6D, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x5F, 0x6D, 0x69, 0x6E, 0x69, 0x6D, 0x75,
        0x6D, 0x5F, 0x64, 0x69, 0x73, 0x70, 0x6C, 0x61, 0x79, 0x5F, 0x74, 0x69, 0x6D, 0x65, 0x00,
        0x31, 0x00, 0x5F, 0x74, 0x75, 0x74, 0x6F, 0x72, 0x5F, 0x6D, 0x65, 0x73, 0x73, 0x61, 0x67,
        0x65, 0x5F, 0x72, 0x65, 0x70, 0x65, 0x61, 0x74, 0x73, 0x00, 0x35, 0x00, 0x5F, 0x74, 0x75,
        0x74, 0x6F, 0x72, 0x5F, 0x76, 0x69, 0x65, 0x77, 0x5F, 0x64, 0x69, 0x73, 0x74, 0x61, 0x6E,
        0x63, 0x65, 0x00, 0x31, 0x30, 0x30, 0x30, 0x00, 0x61, 0x6C, 0x6C, 0x6F, 0x77, 0x5F, 0x73,
        0x70, 0x65, 0x63, 0x74, 0x61, 0x74, 0x6F, 0x72, 0x73, 0x00, 0x31, 0x00, 0x61, 0x6D, 0x78,
        0x5F, 0x63, 0x6C, 0x69, 0x65, 0x6E, 0x74, 0x5F, 0x6C, 0x61, 0x6E, 0x67, 0x75, 0x61, 0x67,
        0x65, 0x73, 0x00, 0x31, 0x00, 0x61, 0x6D, 0x78, 0x5F, 0x6C, 0x61, 0x6E, 0x67, 0x75, 0x61,
        0x67, 0x65, 0x00, 0x66, 0x72, 0x00, 0x61, 0x6D, 0x78, 0x5F, 0x6E, 0x65, 0x78, 0x74, 0x6D,
        0x61, 0x70, 0x00, 0x64, 0x65, 0x5F, 0x61, 0x7A, 0x74, 0x65, 0x63, 0x00, 0x61, 0x6D, 0x78,
        0x5F, 0x74, 0x69, 0x6D, 0x65, 0x6C, 0x65, 0x66, 0x74, 0x00, 0x30, 0x30, 0x3A, 0x30, 0x30,
        0x00, 0x61, 0x6D, 0x78, 0x6D, 0x6F, 0x64, 0x78, 0x5F, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6F,
        0x6E, 0x00, 0x31, 0x2E, 0x37, 0x36, 0x64, 0x00, 0x63, 0x6F, 0x6F, 0x70, 0x00, 0x30, 0x00,
        0x63, 0x73, 0x64, 0x6D, 0x5F, 0x61, 0x63, 0x74, 0x69, 0x76, 0x65, 0x00, 0x31, 0x00, 0x63,
        0x73, 0x64, 0x6D, 0x5F, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6F, 0x6E, 0x00, 0x32, 0x2E, 0x31,
        0x00, 0x64, 0x65, 0x61, 0x74, 0x68, 0x6D, 0x61, 0x74, 0x63, 0x68, 0x00, 0x31, 0x00, 0x64,
        0x65, 0x63, 0x61, 0x6C, 0x66, 0x72, 0x65, 0x71, 0x75, 0x65, 0x6E, 0x63, 0x79, 0x00, 0x36,
        0x30, 0x00, 0x65, 0x64, 0x67, 0x65, 0x66, 0x72, 0x69, 0x63, 0x74, 0x69, 0x6F, 0x6E, 0x00,
        0x32, 0x00, 0x68, 0x6F, 0x73, 0x74, 0x61, 0x67, 0x65, 0x5F, 0x64, 0x65, 0x62, 0x75, 0x67,
        0x00, 0x30, 0x00, 0x68, 0x6F, 0x73, 0x74, 0x61, 0x67, 0x65, 0x5F, 0x73, 0x74, 0x6F, 0x70,
        0x00, 0x30, 0x00, 0x68, 0x75, 0x6D, 0x61, 0x6E, 0x73, 0x5F, 0x6A, 0x6F, 0x69, 0x6E, 0x5F,
        0x74, 0x65, 0x61, 0x6D, 0x00, 0x61, 0x6E, 0x79, 0x00, 0x6A, 0x74, 0x70, 0x31, 0x30, 0x31,
        0x38, 0x31, 0x00, 0x63, 0x68, 0x75, 0x74, 0x65, 0x00, 0x6D, 0x61, 0x78, 0x5F, 0x71, 0x75,
        0x65, 0x72, 0x69, 0x65, 0x73, 0x5F, 0x73, 0x65, 0x63, 0x00, 0x31, 0x00, 0x6D, 0x61, 0x78,
        0x5F, 0x71, 0x75, 0x65, 0x72, 0x69, 0x65, 0x73, 0x5F, 0x73, 0x65, 0x63, 0x5F, 0x67, 0x6C,
        0x6F, 0x62, 0x61, 0x6C, 0x00, 0x31, 0x00, 0x6D, 0x61, 0x78, 0x5F, 0x71, 0x75, 0x65, 0x72,
        0x69, 0x65, 0x73, 0x5F, 0x77, 0x69, 0x6E, 0x64, 0x6F, 0x77, 0x00, 0x31, 0x00, 0x6D, 0x65,
        0x74, 0x61, 0x6D, 0x6F, 0x64, 0x5F, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6F, 0x6E, 0x00, 0x31,
        0x2E, 0x31, 0x39, 0x00, 0x6D, 0x70, 0x5F, 0x61, 0x6C, 0x6C, 0x6F, 0x77, 0x6D, 0x6F, 0x6E,
        0x73, 0x74, 0x65, 0x72, 0x73, 0x00, 0x30, 0x00, 0x6D, 0x70, 0x5F, 0x61, 0x75, 0x74, 0x6F,
        0x6B, 0x69, 0x63, 0x6B, 0x00, 0x30, 0x00, 0x6D, 0x70, 0x5F, 0x61, 0x75, 0x74, 0x6F, 0x74,
        0x65, 0x61, 0x6D, 0x62, 0x61, 0x6C, 0x61, 0x6E, 0x63, 0x65, 0x00, 0x30, 0x00, 0x6D, 0x70,
        0x5F, 0x62, 0x75, 0x79, 0x74, 0x69, 0x6D, 0x65, 0x00, 0x39, 0x39, 0x39, 0x39, 0x00, 0x6D,
        0x70, 0x5F, 0x63, 0x34, 0x74, 0x69, 0x6D, 0x65, 0x72, 0x00, 0x33, 0x35, 0x00, 0x6D, 0x70,
        0x5F, 0x63, 0x68, 0x61, 0x74, 0x74, 0x69, 0x6D, 0x65, 0x00, 0x35, 0x00, 0x6D, 0x70, 0x5F,
        0x63, 0x6F, 0x6E, 0x73, 0x69, 0x73, 0x74, 0x65, 0x6E, 0x63, 0x79, 0x00, 0x31, 0x00, 0x6D,
        0x70, 0x5F, 0x66, 0x61, 0x64, 0x65, 0x74, 0x6F, 0x62, 0x6C, 0x61, 0x63, 0x6B, 0x00, 0x30,
        0x00, 0x6D, 0x70, 0x5F, 0x66, 0x6C, 0x61, 0x73, 0x68, 0x6C, 0x69, 0x67, 0x68, 0x74, 0x00,
        0x31, 0x00, 0x6D, 0x70, 0x5F, 0x66, 0x6F, 0x6F, 0x74, 0x73, 0x74, 0x65, 0x70, 0x73, 0x00,
        0x31, 0x00, 0x6D, 0x70, 0x5F, 0x66, 0x6F, 0x72, 0x63, 0x65, 0x63, 0x61, 0x6D, 0x65, 0x72,
        0x61, 0x00, 0x30, 0x00, 0x6D, 0x70, 0x5F, 0x66, 0x6F, 0x72, 0x63, 0x65, 0x63, 0x68, 0x61,
        0x73, 0x65, 0x63, 0x61, 0x6D, 0x00, 0x30, 0x00, 0x6D, 0x70, 0x5F, 0x66, 0x72, 0x61, 0x67,
        0x73, 0x6C, 0x65, 0x66, 0x74, 0x00, 0x30, 0x00, 0x6D, 0x70, 0x5F, 0x66, 0x72, 0x65, 0x65,
        0x66, 0x6F, 0x72, 0x61, 0x6C, 0x6C, 0x00, 0x30, 0x00, 0x6D, 0x70, 0x5F, 0x66, 0x72, 0x65,
        0x65, 0x7A, 0x65, 0x74, 0x69, 0x6D, 0x65, 0x00, 0x32, 0x00, 0x6D, 0x70, 0x5F, 0x66, 0x72,
        0x69, 0x65, 0x6E, 0x64, 0x6C, 0x79, 0x66, 0x69, 0x72, 0x65, 0x00, 0x30, 0x00, 0x6D, 0x70,
        0x5F, 0x67, 0x68, 0x6F, 0x73, 0x74, 0x66, 0x72, 0x65, 0x71, 0x75, 0x65, 0x6E, 0x63, 0x79,
        0x00, 0x30, 0x2E, 0x31, 0x00, 0x6D, 0x70, 0x5F, 0x68, 0x6F, 0x73, 0x74, 0x61, 0x67, 0x65,
        0x70, 0x65, 0x6E, 0x61, 0x6C, 0x74, 0x79, 0x00, 0x31, 0x33, 0x00, 0x6D, 0x70, 0x5F, 0x6B,
        0x69, 0x63, 0x6B, 0x70, 0x65, 0x72, 0x63, 0x65, 0x6E, 0x74, 0x00, 0x30, 0x00, 0x6D, 0x70,
        0x5F, 0x6C, 0x69, 0x6D, 0x69, 0x74, 0x74, 0x65, 0x61, 0x6D, 0x73, 0x00, 0x30, 0x00, 0x6D,
        0x70, 0x5F, 0x6C, 0x6F, 0x67, 0x64, 0x65, 0x74, 0x61, 0x69, 0x6C, 0x00, 0x33, 0x00, 0x6D,
        0x70, 0x5F, 0x6C, 0x6F, 0x67, 0x66, 0x69, 0x6C, 0x65, 0x00, 0x31, 0x00, 0x6D, 0x70, 0x5F,
        0x6C, 0x6F, 0x67, 0x6D, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x73, 0x00, 0x31, 0x00, 0x6D,
        0x70, 0x5F, 0x6D, 0x61, 0x70, 0x76, 0x6F, 0x74, 0x65, 0x72, 0x61, 0x74, 0x69, 0x6F, 0x00,
        0x31, 0x00, 0x6D, 0x70, 0x5F, 0x6D, 0x61, 0x78, 0x72, 0x6F, 0x75, 0x6E, 0x64, 0x73, 0x00,
        0x30, 0x00, 0x6D, 0x70, 0x5F, 0x6D, 0x69, 0x72, 0x72, 0x6F, 0x72, 0x64, 0x61, 0x6D, 0x61,
        0x67, 0x65, 0x00, 0x30, 0x00, 0x6D, 0x70, 0x5F, 0x70, 0x6C, 0x61, 0x79, 0x65, 0x72, 0x69,
        0x64, 0x00, 0x30, 0x00, 0x6D, 0x70, 0x5F, 0x72, 0x6F, 0x75, 0x6E, 0x64, 0x74, 0x69, 0x6D,
        0x65, 0x00, 0x33, 0x00, 0x6D, 0x70, 0x5F, 0x73, 0x74, 0x61, 0x72, 0x74, 0x6D, 0x6F, 0x6E,
        0x65, 0x79, 0x00, 0x38, 0x30, 0x30, 0x00, 0x6D, 0x70, 0x5F, 0x74, 0x69, 0x6D, 0x65, 0x6C,
        0x65, 0x66, 0x74, 0x00, 0x30, 0x00, 0x6D, 0x70, 0x5F, 0x74, 0x69, 0x6D, 0x65, 0x6C, 0x69,
        0x6D, 0x69, 0x74, 0x00, 0x30, 0x00, 0x6D, 0x70, 0x5F, 0x74, 0x6B, 0x70, 0x75, 0x6E, 0x69,
        0x73, 0x68, 0x00, 0x30, 0x00, 0x6D, 0x70, 0x5F, 0x77, 0x69, 0x6E, 0x64, 0x69, 0x66, 0x66,
        0x65, 0x72, 0x65, 0x6E, 0x63, 0x65, 0x00, 0x31, 0x00, 0x6D, 0x70, 0x5F, 0x77, 0x69, 0x6E,
        0x6C, 0x69, 0x6D, 0x69, 0x74, 0x00, 0x30, 0x00, 0x70, 0x61, 0x75, 0x73, 0x61, 0x62, 0x6C,
        0x65, 0x00, 0x31, 0x00, 0x73, 0x76, 0x5F, 0x61, 0x63, 0x63, 0x65, 0x6C, 0x65, 0x72, 0x61,
        0x74, 0x65, 0x00, 0x35, 0x00, 0x73, 0x76, 0x5F, 0x61, 0x69, 0x6D, 0x00, 0x30, 0x00, 0x73,
        0x76, 0x5F, 0x61, 0x69, 0x72, 0x61, 0x63, 0x63, 0x65, 0x6C, 0x65, 0x72, 0x61, 0x74, 0x65,
        0x00, 0x31, 0x30, 0x30, 0x00, 0x73, 0x76, 0x5F, 0x61, 0x69, 0x72, 0x6D, 0x6F, 0x76, 0x65,
        0x00, 0x31, 0x00, 0x73, 0x76, 0x5F, 0x61, 0x6C, 0x6C, 0x6F, 0x77, 0x75, 0x70, 0x6C, 0x6F,
        0x61, 0x64, 0x00, 0x31, 0x00, 0x73, 0x76, 0x5F, 0x61, 0x6C, 0x6C, 0x74, 0x61, 0x6C, 0x6B,
        0x00, 0x31, 0x00, 0x73, 0x76, 0x5F, 0x62, 0x6F, 0x75, 0x6E, 0x63, 0x65, 0x00, 0x31, 0x00,
        0x73, 0x76, 0x5F, 0x63, 0x68, 0x65, 0x61, 0x74, 0x73, 0x00, 0x30, 0x00, 0x73, 0x76, 0x5F,
        0x63, 0x6C, 0x69, 0x65, 0x6E, 0x74, 0x74, 0x72, 0x61, 0x63, 0x65, 0x00, 0x31, 0x00, 0x73,
        0x76, 0x5F, 0x63, 0x6C, 0x69, 0x70, 0x6D, 0x6F, 0x64, 0x65, 0x00, 0x30, 0x00, 0x73, 0x76,
        0x5F, 0x63, 0x6F, 0x6E, 0x74, 0x61,
    ];

    let response = parse_rules(&long_rules).unwrap();

    // Just checks that there is remaining data
    assert_eq!(93, response.num_rules);
    assert_eq!("sv_conta".to_string(), response.remaining_data);
}

#[test]
/// Check that payloads with data remaining after all rules have been parsed are properly rejected
fn payload_after_rules() {
    let mut rule_bytes = include_bytes!("../test_bytes/mucosmosTheShip.rules").to_vec();
    rule_bytes.extend(&[0xFF, 0xFF, 0xFF]);

    // Skip the header byte
    let rules_error = parse_rules(&rule_bytes[1..]).unwrap_err();

    let error = nom::error::Error::new(&rule_bytes[..0], nom::error::ErrorKind::NonEmpty);

    assert_eq!(error, rules_error)
}
