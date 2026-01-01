use esp_hal::Blocking;

use crate::Clock;
use core::str::FromStr;
use esp_hal::time::{Duration, Instant};
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use heapless::{String, Vec, format};
use packet_encoding::{PacketEncodeErr, encode_packet};
use topics::*;

pub enum SendError {
    EncodeError(PacketEncodeErr),
    Timeout,
}

fn send_message(
    usb: &mut UsbSerialJtag<Blocking>,
    message: &PacketFormat,
) -> Result<(), SendError> {
    const TIMEOUT: Duration = Duration::from_millis(10);


    let encode_start = Instant::now();

    let mut encode_buffer = [0u8; 600];
    encode_buffer[0] = 0; // COBS initial byte
    let encoded_size =
        encode_packet(message, &mut encode_buffer[1..]).map_err(SendError::EncodeError)?;
    encode_buffer[encoded_size + 1] = 0x00; // COBS final byte
    let encode_sized = &encode_buffer[..encoded_size + 2];

    let encode_duration = encode_start.elapsed();


    let simple_transmit_start_time = Instant::now();
    usb.write(encode_sized).expect("Failed to write to USB");
    let simple_transmit_duration = simple_transmit_start_time.elapsed();

    let transmit_start_time = Instant::now();

    // Slice into 64 byte chunks and write each chunk
    let start_time = Instant::now();
    for chunk in encode_sized.chunks(64) {
        for byte in chunk {
            while start_time.elapsed() < TIMEOUT {
                match usb.write_byte_nb(*byte) {
                    Ok(_) => {
                        break
                    }
                    Err(_) => {
                        continue
                    }

                }
            }
        }
        while start_time.elapsed() < TIMEOUT {
            match usb.flush_tx_nb() {
                Ok(_) => {
                    break
                },
                Err(_) => {
                    continue
                }
            }
        }

        // #[allow(clippy::nonminimal_bool)]
        // if timed_out {
        //     return Err(SendError::Timeout);
        // }
    }


    let transmit_duration = transmit_start_time.elapsed();

    let data1: String<64> = format!("Encode Duration {} us\r\n", encode_duration.as_micros()).unwrap();
    let data2: String<64> = format!("Transmit Duration {} us\r\n", transmit_duration.as_micros()).unwrap();
    let data3: String<64> = format!("Simple Transmit Duration {} us\r\n", simple_transmit_duration.as_micros()).unwrap();
    usb.write(data1.as_bytes()).ok();
    usb.write(data2.as_bytes()).ok();
    usb.write(data3.as_bytes()).ok();


    // TODO: Performance of the transmit with timeout sucks. Simple transmit takes 165us, but the timeout version takes 2725us.
    // This is terrible!

    // The API doesn't seem to offer us any alternative, so we should probably reimplement their write with a timeout:
    /*
    
    let per = peripherals.USB_DEVICE.register_block();
    let bits = per.ep1_conf().read().bits();

    // From https://github.com/esp-rs/esp-hal/blob/main/esp-hal/src/usb_serial_jtag.rs
    pub fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        for chunk in data.chunks(64) {
            for byte in chunk {
                self.regs()
                    .ep1()
                    .write(|w| unsafe { w.rdwr_byte().bits(*byte) });
            }
            self.regs().ep1_conf().modify(|_, w| w.wr_done().set_bit());

            // FIXME: raw register access
            while self.regs().ep1_conf().read().bits() & 0b011 == 0b000 {
                // wait  // Do timeout check in here.
            }
        }

        Ok(())
    }

    */

    Ok(())
}

pub struct HostConnection<'a> {
    usb: UsbSerialJtag<'a, Blocking>,
    packet_finder: packet_encoding::PacketFinder,
    message_id: u32,
    decode_errors: u32,
}

impl<'a> HostConnection<'a> {
    pub fn new(usb: UsbSerialJtag<'a, Blocking>) -> Self {
        HostConnection {
            usb,
            packet_finder: packet_encoding::PacketFinder::new(),
            message_id: 0,
            decode_errors: 0,
        }
    }

    pub fn send_packet(
        &mut self,
        clock: &Clock,
        data: PacketData,
        to: Option<u16>,
    ) -> Result<(), SendError> {
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

    pub fn step(&mut self) -> Option<PacketFormat> {
        while let Ok(byte) = self.usb.read_byte() {
            if let Some(mut packet) = self.packet_finder.push_byte(byte)
                && !packet.is_empty()
            {
                if let Ok(packet) = packet_encoding::decode_packet::<PacketFormat>(&mut packet) {
                   return Some(packet)

                } else {
                        self.decode_errors = self.decode_errors.wrapping_add(1);
                    
                }
            }
        }
        None
    }
}
