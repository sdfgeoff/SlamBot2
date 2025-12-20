#![no_std]

use cobs::{CobsEncoder, decode_in_place};
use crc16::{ARC, State};
use serde::{Deserialize, Serialize};
use serde_cbor::{Serializer, ser::SliceWrite, de::from_mut_slice};

#[derive(Debug)]
pub enum PacketEncodeErr {
    SerdeError(serde_cbor::Error),
    CobsError,
    DestBufTooSmallError,
}


/**
 * COBS(
 *     CBOR(MESSAGE)
 *     CRC16(CBOR(MESSAGE))
 * )
 */
pub fn encode_packet(message: &impl Serialize, encode_buffer: &mut [u8]) -> Result<usize, PacketEncodeErr> {
    // CBOR
    let mut serialize_buffer = [0u8; 500];  // Wish I knew how to get rid of this fixed size buffer without needing alloc
    let writer = SliceWrite::new(&mut serialize_buffer[..]);
    let mut ser = Serializer::new(writer);
    message.serialize(&mut ser).map_err(PacketEncodeErr::SerdeError)?;
    let writer = ser.into_inner();
    let size = writer.bytes_written();
    let serialized = &serialize_buffer[..size];
    
    // CRC16
    let crc = State::<ARC>::calculate(serialized);

    // COBS
    let mut encoder = CobsEncoder::new(encode_buffer);
    encoder.push(serialized).map_err(|_| PacketEncodeErr::DestBufTooSmallError)?;
    encoder.push(&crc.to_le_bytes()).map_err(|_| PacketEncodeErr::DestBufTooSmallError)?;
    let encoded_size = encoder.finalize();
    Ok(encoded_size)
}

#[derive(Debug)]
pub enum PacketDecodeErr {
    SerdeError(serde_cbor::Error),
    CobsError,
    CrcMismatchError,
}

pub fn decode_packet<T: for<'a> Deserialize<'a>>(data: &mut [u8]) -> Result<T, PacketDecodeErr> {
    // COBS
    let decoded_size = decode_in_place(data).map_err(|_| PacketDecodeErr::CobsError)?;
    if decoded_size < 2 {
        return Err(PacketDecodeErr::CobsError);
    }
    let (payload, crc_bytes) = data[..decoded_size].split_at_mut(decoded_size - 2);
    
    // CRC16
    let received_crc = u16::from_le_bytes([crc_bytes[0], crc_bytes[1]]); 
    let calculated_crc = State::<ARC>::calculate(payload);
    if received_crc != calculated_crc {
        return Err(PacketDecodeErr::CrcMismatchError);
    }

    // CBOR
    let message: T = from_mut_slice(payload).map_err(PacketDecodeErr::SerdeError)?;
    Ok(message)
}