use nom::{
    combinator::rest,
    error::Error,
    number::complete::{le_i16, le_i32, le_u8},
    Finish, IResult,
};

// # Structs / Enums

/// Pre-source and Source single packet message
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SinglePacket<'a> {
    /// Type of the contained message
    pub message_header: MessageHeader,
    /// Payload data without header
    pub payload: &'a [u8],
}

/// Generic Packet Fragment for both Source and Goldsource
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PacketFragment<'a> {
    /// Unique response identifier
    pub id: i32,
    /// Total number of packets in the response
    pub total_packets: u8,
    /// This fragments packet number, used to reorder and reassemble the payload
    pub packet_number: u8,
    /// Payload contained within the fragment
    pub payload: &'a [u8],

    /// If the payload is compressed or not
    pub payload_compressed: bool,
    /// Source: Packet maximum size, Some games do not include this field: 215, 17550, 17700, 240 when protocol = 7
    pub size: Option<i16>,
    /// Source: Total size of the decompressed payload, only present in the first packet of a response when it is compressed
    pub decompressed_size: Option<i32>,
    /// Source: CRC32 checksum of the payload data, only present in the first packet of a response when it is compressed
    pub crc32_checksum: Option<i32>,
}

/// Contains both types of packets to allow for either to be returned from functions
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Packet<'a> {
    /// Message is in a single packet
    SinglePack(SinglePacket<'a>),
    /// Message is split across packets
    PAcketFragment(PacketFragment<'a>),
}

/// Indicates the type of payload contained within the packet  
/// Used in [`packet`](crate::packet)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MessageHeader {
    /// [A2S_INFO Request](https://developer.valvesoftware.com/wiki/Server_queries#Request_Format) -> 'T'
    InfoRequest,
    /// [A2S_INFO Response for Source](https://developer.valvesoftware.com/wiki/Server_queries#Response_Format) -> 'I'
    InfoResponseSource,
    /// [A2S_INFO Response for GoldSource](https://developer.valvesoftware.com/wiki/Server_queries#Obsolete_GoldSource_Response) -> 'm'
    InfoResponseGoldSource,
    /// [A2S_PLAYER Request](https://developer.valvesoftware.com/wiki/Server_queries#Request_Format_2) -> 'U'
    PlayerRequest,
    /// [A2S_PLAYER Response](https://developer.valvesoftware.com/wiki/Server_queries#Response_Format_2) -> 'D'
    PlayerResponse,
    /// [A2S_RULES Request](https://developer.valvesoftware.com/wiki/Server_queries#Request_Format_3) -> 'V'
    RulesRequest,
    /// [A2S_RULES Response](https://developer.valvesoftware.com/wiki/Server_queries#Response_Format_3) -> 'E'
    RulesResponse,
    /// [A2S_PING Request](https://developer.valvesoftware.com/wiki/Server_queries#Request_Format_4) -> 'i'
    PingRequest,
    /// [A2S_PING Response](https://developer.valvesoftware.com/wiki/Server_queries#Response_Format_4) -> 'j'
    PingResponse,
    /// [A2S_SERVERQUERY_GETCHALLENGE Request](https://developer.valvesoftware.com/wiki/Server_queries#Request_Format_5) -> 'W'
    ChallengeRequest,
    /// [A2S_SERVERQUERY_GETCHALLENGE Response](https://developer.valvesoftware.com/wiki/Server_queries#Response_Format_5) -> 'A'
    ChallengeResponse,
    /// Invalid packet header type
    Invalid,
}

impl From<u8> for MessageHeader {
    fn from(input: u8) -> Self {
        match input {
            // 'T'
            0x54 => MessageHeader::InfoRequest,
            // 'I'
            0x49 => MessageHeader::InfoResponseSource,
            // 'm'
            0x6D => MessageHeader::InfoResponseGoldSource,
            // 'U'
            0x55 => MessageHeader::PlayerRequest,
            // 'D'
            0x44 => MessageHeader::PlayerResponse,
            // 'V'
            0x56 => MessageHeader::RulesRequest,
            // 'E'
            0x45 => MessageHeader::RulesResponse,
            // 'i'
            0x69 => MessageHeader::PingRequest,
            // 'j'
            0x6A => MessageHeader::PingResponse,
            // 'W'
            0x57 => MessageHeader::ChallengeRequest,
            // 'A'
            0x41 => MessageHeader::ChallengeResponse,
            // All other values don't correspond to anything according to the wiki
            _ => MessageHeader::Invalid,
        }
    }
}

// # Exposed final parsers
/// Parse a packet payload into message type and message
/// Packet type (single/split) must be determined before hand and removed
pub fn parse_single_packet(input: &[u8]) -> Result<SinglePacket, Error<&[u8]>> {
    match single_packet(input).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}

/// Attempt to parse the provided slice into a valid Goldsource Response, nom errors are returned on failure.
pub fn parse_goldsource_multi_packet(input: &[u8]) -> Result<PacketFragment, Error<&[u8]>> {
    match goldsource_multi_packet(input).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}
/// Attempt to parse the provided slice into a valid Source Response, nom errors are returned on failure.
/// Size is true except for AppIds: `215, 17550, 17700, and 240 when protocol = 7`
pub fn parse_source_multi_packet(input: &[u8], size: bool) -> Result<PacketFragment, Error<&[u8]>> {
    match source_multi_packet(input, size).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}

/// Attempt to parse the type of message contained in the payload
pub fn message_type(input: &[u8]) -> Result<MessageHeader, Error<&[u8]>> {
    match message_header(input).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}

/// Read the front of the packet to determine if the payload is split or not
pub fn is_payload_split(input: &[u8]) -> Result<bool, Error<&[u8]>> {
    match is_split(input).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}

// # Private parsing helper functions
fn is_split(input: &[u8]) -> IResult<&[u8], bool> {
    let (input, packet_header) = le_i32(input)?;

    if !(packet_header == -1 || packet_header == -2) {
        return Err(nom::Err::Error(nom::error::Error {
            input,
            code: nom::error::ErrorKind::NoneOf,
        }));
    }

    Ok((input, packet_header == -2))
}

fn message_header(input: &[u8]) -> IResult<&[u8], MessageHeader> {
    let (input, payload_header) = le_u8(input)?;

    Ok((input, payload_header.into()))
}

fn single_packet(input: &[u8]) -> IResult<&[u8], SinglePacket> {
    let (input, message_header) = message_header(input)?;

    if message_header == MessageHeader::Invalid {
        return Err(nom::Err::Error(nom::error::Error {
            input,
            code: nom::error::ErrorKind::NoneOf,
        }));
    }

    let (input, payload) = rest(input)?;

    Ok((
        input,
        SinglePacket {
            message_header,
            payload,
        },
    ))
}

fn source_multi_packet(input: &[u8], size_included: bool) -> IResult<&[u8], PacketFragment> {
    let (input, id) = le_i32(input)?;
    let (input, total_packets) = le_u8(input)?;
    let (input, packet_number) = le_u8(input)?;
    // Wiki lists: 215, 17550, 17700, and 240 when protocol = 7 as not having the size field
    let (input, size) = if size_included {
        le_i16(input).map(|(input, val)| (input, Some(val)))?
    } else {
        (input, None)
    };

    // If it is packet 0 of the response and the most significant bit of id is 1 then the packet payload is compressed
    // MSB set means negative
    let payload_compressed = packet_number == 0 && id < 0;

    let (input, decompressed_size) = if payload_compressed {
        le_i32(input).map(|(input, val)| (input, Some(val)))?
    } else {
        (input, None)
    };

    let (input, crc32_checksum) = if payload_compressed {
        le_i32(input).map(|(input, val)| (input, Some(val)))?
    } else {
        (input, None)
    };

    let (input, payload) = rest(input)?;

    Ok((
        input,
        PacketFragment {
            id,
            total_packets,
            packet_number,
            payload,
            payload_compressed,
            size,
            decompressed_size,
            crc32_checksum,
        },
    ))
}

fn goldsource_multi_packet(input: &[u8]) -> IResult<&[u8], PacketFragment> {
    let (input, id) = le_i32(input)?;
    let (input, packet_number) = le_u8(input)?;
    let packet_number = packet_number >> 4;
    let total_packets = packet_number & 0x0F;
    let (input, payload) = rest(input)?;

    Ok((
        input,
        PacketFragment {
            id,
            total_packets,
            packet_number,
            payload,
            payload_compressed: false,
            size: None,
            decompressed_size: None,
            crc32_checksum: None,
        },
    ))
}

// # Tests

#[test]
fn single_packet_info() {
    let packet_bytes = include_bytes!("../test_bytes/chaoticTTT.info");

    let packet = parse_single_packet(packet_bytes).unwrap();

    assert_eq!(MessageHeader::InfoResponseSource, packet.message_header);
    // Check the first and last bytes of the payload
    assert_eq!(17, packet.payload[0]);
    assert_eq!(0, packet.payload[159]);
}
