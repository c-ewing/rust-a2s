use super::*;

use nom::{combinator::rest, number::complete::{le_i16, le_u8}};
use nom::IResult;

// # Structs / Enums
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SinglePacket<'a> {
    pub header: i32,
    pub payload: &'a [u8],
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GoldsourceMultiPacket<'a> {
    // Header?
    pub id: i32,
    packet_number: u8,
    pub current_packet: u8,
    pub total_packets: u8,
    pub payload: &'a [u8],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceMultiPacket<'a> {
    // Header?
    pub id: i32,
    pub total: u8,
    pub number: u8,
    pub size: Option<i16>,
    pub compression_data: Option<CompressionData>,
    pub payload: &'a [u8],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompressionData {
    pub decompressed_size: i32,
    pub crc32_checksum: i32,
}

// # Exposed final parsers
// Makes sure that all of the input data was consumed, if not to much data was fed or something
// TODO: comment better
pub fn p_goldsource_multi_packet(input: &[u8]) -> IResult<&[u8], GoldsourceMultiPacket> {
    let (input, id) = le_i32(input)?;
    let (input, packet_number) = le_u8(input)?;
    let current_packet = packet_number >> 4;
    let total_packets = packet_number & 0x0F;
    let (input, payload) = rest(input)?;

    Ok((input, GoldsourceMultiPacket {
        id,
        packet_number,
        current_packet,
        total_packets,
        payload,
    }))
}

pub fn p_source_multi_packet(input: &[u8]) -> IResult<&[u8], SourceMultiPacket> {
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

    Ok((input, SourceMultiPacket {
        id,
        total,
        number,
        size: Some(size),
        compression_data,
        payload,
    }))
}

// # Private parsing helper functions
#[allow(dead_code, unused_variables)]
fn single_packet(input: &[u8]) -> IResult<&[u8], SinglePacket> {
    unimplemented!("Unused because we really don't need a struct just to hold a ref")
}

fn compression_data(input: &[u8], first_packet_and_compressed: bool) -> IResult<&[u8], Option<CompressionData>>{
    match first_packet_and_compressed {
        true => {
            let (input, decompressed_size) = le_i32(input)?;
            let (input, crc32_checksum) = le_i32(input)?;

            Ok((input, Some(CompressionData{
                decompressed_size,
                crc32_checksum,
            })))
        },
        false => Ok((input,None)),
    }

}




// # Tests