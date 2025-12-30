use esp_hal::{Blocking};

use esp_hal::usb_serial_jtag::UsbSerialJtag;
use packet_encoding::{PacketEncodeErr, encode_packet};
use esp_hal::time::{Instant, Duration};
use heapless::{String, format};
use core::str::FromStr;
use topics::*;
use crate::Clock;



pub enum SendError {
    EncodeError(PacketEncodeErr),
    Timeout
}

fn send_message(
    usb: &mut UsbSerialJtag<Blocking>,
    message: &PacketFormat,
) -> Result<(), SendError> {
    const TIMEOUT: Duration = Duration::from_millis(1);

    let mut encode_buffer = [0u8; 600];
    encode_buffer[0] = 0; // COBS initial byte
    let encoded_size = encode_packet(message, &mut encode_buffer[1..]).map_err(SendError::EncodeError)?;
    encode_buffer[encoded_size + 1] = 0x00; // COBS final byte
    let encode_sized = &encode_buffer[..encoded_size + 2];

    // usb.write(encode_sized).map_err(|_| SendError::UsbError)?;

    // Slice into 64 byte chunks and write each chunk
    let start_time = Instant::now();
    for chunk in encode_sized.chunks(64) {
        let mut duration = start_time.elapsed();
        for byte in chunk {
            while duration < TIMEOUT {
                duration = start_time.elapsed();
                match usb.write_byte_nb(*byte) {
                    Ok(_) => break,
                    Err(_) => continue,
                }
            }
        }
        while duration < TIMEOUT {
            duration = start_time.elapsed();
            match usb.flush_tx_nb() {
                Ok(_) => break,
                Err(_) => continue,
            }
        }

        if !(duration < TIMEOUT) {
            return Err(SendError::Timeout);
        }
    }

    Ok(())
}





pub struct HostConnection<'a> {
    usb: UsbSerialJtag<'a , Blocking>,
    packet_finder: packet_encoding::PacketFinder,
    message_id: u32,
}

impl<'a> HostConnection<'a> {
    pub fn new(usb: UsbSerialJtag<'a, Blocking>) -> Self {
        HostConnection {
            usb,
            packet_finder: packet_encoding::PacketFinder::new(),
            message_id: 0,
        }
    }

    pub fn send_packet(&mut self, clock: &Clock, data: PacketData, to: Option<u16>) -> Result<(), SendError> {
        let packet = PacketFormat {
            to,
            from: None,
            data,
            time: clock.get_time(),
            id: self.message_id,
        };
        self.message_id = self.message_id.wrapping_add(1);
        send_message(&mut self.usb, &packet)
    }


    pub fn step(&mut self, clock: &mut Clock) {
        while let Ok(byte) = self.usb.read_byte() {
            if let Some(mut packet) = self.packet_finder.push_byte(byte)
                && !packet.is_empty()
            {
                match packet_encoding::decode_packet::<PacketFormat>(&mut packet) {
                    Ok(packet) => {
                        match packet.data {
                            PacketData::ClockResponse(resp) => {
                                let round_trip_time = clock.handle_clock_response(&resp);
                                self.send_packet(
                                    clock,
                                    PacketData::LogMessage(LogMessage {
                                        level: LogLevel::Info,
                                        event: String::from_str("time_sync").unwrap(),
                                        json: format!(
                                            "{{\"offset\": {}, \"rtt\": {}}}",
                                            clock.offset.unwrap_or(0), round_trip_time
                                        ).ok(),
                                    }),
                                    None,
                                ).ok();

                            }
                            _ => {}
                        }   
                    }
                    Err(_e) => {
                        // packet_errors = packet_errors.wrapping_add(1);
                    }
                }
            }
        }
    }
}