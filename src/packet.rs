use nom::{
    combinator::rest,
    error::Error,
    number::complete::{le_i16, le_i32, le_u8},
    Finish, IResult,
};

// # Structs / Enums
#[derive(Clone, Debug, PartialEq, Eq)]
/// Gold Source Multi Packet response packet as described on the [wiki](https://developer.valvesoftware.com/wiki/Server_queries#Goldsource_Server)
pub struct GoldsourceMultiPacket<'a> {
    /// Unique number assigned by the server per response
    pub id: i32,
    /// Upper four bits represent the current packet number and the lower four represent the total number of packets in the response
    packet_number: u8,
    /// Packet number in this response
    pub current_packet: u8,
    /// Total number of packets in the response
    pub total_packets: u8,
    /// Payload of the response
    pub payload: &'a [u8],
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Source Multi Packet response packet as described on the [wiki](https://developer.valvesoftware.com/wiki/Server_queries#Source_Server)
pub struct SourceMultiPacket<'a> {
    /// Unique packet id, if the most significant digit is set then the payload is compressed with bzip2
    pub id: i32,
    /// Number of packets in the response
    pub total: u8,
    /// Packet number in the response
    pub number: u8,
    /// Size of the packet before packet switching occurs. Very few games do not have this field
    pub size: Option<i16>,
    /// If the most significant digit is set then the packet also contains data about the compressed payload, decompressed size and crc32.
    /// Only contained within the first packet of a response.
    pub compression_data: Option<CompressionData>,
    /// Payload of the response
    pub payload: &'a [u8],
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Optional data contained within the first packet of a Source Multi Packet response
pub struct CompressionData {
    /// Total size of the decompressed payload
    pub decompressed_size: i32,
    /// CRC32 checksum of the payload data
    pub crc32_checksum: i32,
}

/// Indicates the type of payload contained within the packet  
/// Used in [`packet`](crate::packet)
pub enum PayloadHeader {
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
    /// Likely invalid packet
    Other(u8),
}

impl From<u8> for PayloadHeader {
    fn from(input: u8) -> Self {
        match input {
            // 'T'
            0x54 => PayloadHeader::InfoRequest,
            // 'I'
            0x49 => PayloadHeader::InfoResponseSource,
            // 'm'
            0x6D => PayloadHeader::InfoResponseGoldSource,
            // 'U'
            0x55 => PayloadHeader::PlayerRequest,
            // 'D'
            0x44 => PayloadHeader::PlayerResponse,
            // 'V'
            0x56 => PayloadHeader::RulesRequest,
            // 'E'
            0x45 => PayloadHeader::RulesResponse,
            // 'i'
            0x69 => PayloadHeader::PingRequest,
            // 'j'
            0x6A => PayloadHeader::PingResponse,
            // 'W'
            0x57 => PayloadHeader::ChallengeRequest,
            // 'A'
            0x41 => PayloadHeader::ChallengeResponse,
            // All other values don't correspond to anything according to the wiki
            _ => PayloadHeader::Other(input),
        }
    }
}

// # Exposed final parsers
/// Attempt to parse the provided slice into a valid Goldsource Response, nom errors are returned on failure.
pub fn parse_goldsource_multi_packet(input: &[u8]) -> Result<GoldsourceMultiPacket, Error<&[u8]>> {
    match p_goldsource_multi_packet(input).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}
/// Attempt to parse the provided slice into a valid Source Response, nom errors are returned on failure.
pub fn parse_source_multi_packet(input: &[u8]) -> Result<SourceMultiPacket, Error<&[u8]>> {
    match p_source_multi_packet(input).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}

// # Additional minor parsers for determining single/multi packet and the payload type
/// The first byte of the payload indicates the message type contained within according to the [`PayloadHeader`](crate::parser_util::PayloadHeader)
pub fn parse_payload_header(input: &[u8]) -> Result<PayloadHeader, Error<&[u8]>> {
    match p_payload_header(input).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}

/// Returns true if the first byte of the response is -2, indicating the response is split over multiple packets [wiki](https://developer.valvesoftware.com/wiki/Server_queries#Simple_Response_Format)
pub fn parse_is_split_payload(input: &[u8]) -> Result<bool, Error<&[u8]>> {
    match p_is_split_payload(input).finish() {
        Ok(v) => Ok(v.1),
        Err(e) => Err(e),
    }
}

// # Private parsing helper functions
fn p_goldsource_multi_packet(input: &[u8]) -> IResult<&[u8], GoldsourceMultiPacket> {
    let (input, id) = le_i32(input)?;
    let (input, packet_number) = le_u8(input)?;
    let current_packet = packet_number >> 4;
    let total_packets = packet_number & 0x0F;
    let (input, payload) = rest(input)?;

    Ok((
        input,
        GoldsourceMultiPacket {
            id,
            packet_number,
            current_packet,
            total_packets,
            payload,
        },
    ))
}

fn p_source_multi_packet(input: &[u8]) -> IResult<&[u8], SourceMultiPacket> {
    let (input, id) = le_i32(input)?;
    let (input, total) = le_u8(input)?;
    let (input, number) = le_u8(input)?;
    // TODO: Size is an odd one as most games use it.. but certain ones dont.. have to chain parsers maybe?
    // Wiki lists: 215, 17550, 17700, and 240 when protocol = 7 as not having the size field
    let (input, size) = le_i16(input)?;
    // If it is packet 0 of the response and the most significant bit of id is 1 then the packet payload is compressed
    // MSB set means negative
    let (input, compression_data) = compression_data(input, number == 0 && id < 0)?;
    let (input, payload) = rest(input)?;

    Ok((
        input,
        SourceMultiPacket {
            id,
            total,
            number,
            size: Some(size),
            compression_data,
            payload,
        },
    ))
}

fn p_is_split_payload(input: &[u8]) -> IResult<&[u8], bool> {
    let (input, single_packet) = le_i32(input)?;

    Ok((input, single_packet == -2))
}

fn p_payload_header(input: &[u8]) -> IResult<&[u8], PayloadHeader> {
    let (input, payload_header) = le_u8(input)?;

    Ok((input, payload_header.into()))
}

fn compression_data(input: &[u8], compressed: bool) -> IResult<&[u8], Option<CompressionData>> {
    match compressed {
        true => {
            let (input, decompressed_size) = le_i32(input)?;
            let (input, crc32_checksum) = le_i32(input)?;

            Ok((
                input,
                Some(CompressionData {
                    decompressed_size,
                    crc32_checksum,
                }),
            ))
        }
        false => Ok((input, None)),
    }
}

// # Tests
