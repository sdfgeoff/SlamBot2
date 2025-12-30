#![no_std]

use cobs::DecodeError;
use cobs::{CobsEncoder, decode_in_place};
use crc16::{ARC, State};
use heapless::Vec;
use serde::{Deserialize, Serialize};
use serde_cbor::{Serializer, de::from_mut_slice, ser::SliceWrite};

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
pub fn encode_packet(
    message: &impl Serialize,
    encode_buffer: &mut [u8],
) -> Result<usize, PacketEncodeErr> {
    // CBOR
    let mut serialize_buffer = [0u8; 500]; // Wish I knew how to get rid of this fixed size buffer without needing alloc
    let writer = SliceWrite::new(&mut serialize_buffer[..]);
    let mut ser = Serializer::new(writer);
    message
        .serialize(&mut ser)
        .map_err(PacketEncodeErr::SerdeError)?;
    let writer = ser.into_inner();
    let size = writer.bytes_written();
    let serialized = &serialize_buffer[..size];

    // CRC16
    let crc = State::<ARC>::calculate(serialized);

    // COBS
    let mut encoder = CobsEncoder::new(encode_buffer);
    encoder
        .push(serialized)
        .map_err(|_| PacketEncodeErr::DestBufTooSmallError)?;
    encoder
        .push(&crc.to_le_bytes())
        .map_err(|_| PacketEncodeErr::DestBufTooSmallError)?;
    let encoded_size = encoder.finalize();
    Ok(encoded_size)
}

#[derive(Debug)]
pub enum PacketDecodeErr {
    SerdeError(serde_cbor::Error),
    TooSmall,
    CobsError(DecodeError),
    CrcMismatchError,
}

pub fn decode_packet<T: for<'a> Deserialize<'a>>(data: &mut [u8]) -> Result<T, PacketDecodeErr> {
    // COBS
    let decoded_size = decode_in_place(data).map_err(PacketDecodeErr::CobsError)?;
    if decoded_size < 2 {
        return Err(PacketDecodeErr::TooSmall);
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

pub struct PacketFinder {
    buffer: Vec<u8, 512>,
    max_buffer_size: usize,
}

impl Default for PacketFinder {
    fn default() -> Self {
        PacketFinder::new()
    }
}

impl PacketFinder {
    pub fn new() -> Self {
        PacketFinder {
            buffer: Vec::new(),
            max_buffer_size: 512,
        }
    }

    pub fn push_byte(&mut self, byte: u8) -> Option<Vec<u8, 512>> {
        if self.buffer.len() < self.max_buffer_size {
            if byte == 0x00 {
                if !self.buffer.is_empty() {
                    // Found a packet
                    let packet = Vec::<u8, 512>::from_slice(&self.buffer[1..]).unwrap();
                    self.buffer.clear();
                    self.buffer.push(byte).unwrap();
                    return Some(packet);
                } else {
                    // Starting a packet
                    self.buffer.clear();
                    self.buffer.push(byte).unwrap();
                }
            } else if !self.buffer.is_empty() {
                self.buffer.push(byte).unwrap();
            }
        } else {
            // Buffer hit capacity, reset
            self.buffer.clear();
        }
        None
    }
}
