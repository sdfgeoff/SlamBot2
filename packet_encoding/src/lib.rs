#![no_std]

use cobs::CobsEncoder;
use crc16::{ARC, State};
use serde::Serialize;
use serde_cbor::{Serializer, ser::SliceWrite};


#[derive(Debug)]
pub enum PacketEncodeErr {
    SerdeError(serde_cbor::Error),
    CobsError,
    DestBufTooSmallError,
}

pub fn encode(message: &impl Serialize, encode_buffer: &mut [u8]) -> Result<usize, PacketEncodeErr> {
    let mut serialize_buffer = [0u8; 500];
    
    let writer = SliceWrite::new(&mut serialize_buffer[..]);
    let mut ser = Serializer::new(writer);
    message.serialize(&mut ser).map_err(PacketEncodeErr::SerdeError)?;
    let writer = ser.into_inner();
    let size = writer.bytes_written();
    let serialized = &serialize_buffer[..size];
    let crc = State::<ARC>::calculate(&serialize_buffer);

    let mut encoder = CobsEncoder::new(encode_buffer);
    encoder.push(serialized).map_err(|_| PacketEncodeErr::DestBufTooSmallError)?;
    encoder.push(&crc.to_le_bytes()).map_err(|_| PacketEncodeErr::DestBufTooSmallError)?;
    let encoded_size = encoder.finalize();
    return Ok(encoded_size + 1);
}