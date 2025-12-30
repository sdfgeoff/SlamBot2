#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::str::FromStr;

use esp_hal::time::{Duration, Instant};
// use esp_backtrace as _;
use esp_hal::{Blocking, main};

use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use packet_encoding::{PacketEncodeErr, encode_packet};

esp_bootloader_esp_idf::esp_app_desc!();

use heapless::{String, format};

use topics::*;


enum SendError {
    UsbError,
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
        let mut duration = Instant::now() - start_time;
        for byte in chunk {
            while duration < TIMEOUT {
                duration = Instant::now() - start_time;
                match usb.write_byte_nb(*byte) {
                    Ok(_) => break,
                    Err(_) => continue,
                }
            }
        }
        while duration < TIMEOUT {
            duration = Instant::now() - start_time;
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



#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let mut led = Output::new(peripherals.GPIO8, Level::High, OutputConfig::default());

    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);

    let mut lastWriteTime = Instant::now();

    let mut packet_finder = packet_encoding::PacketFinder::new();

    // Send boot message
    send_message(
        &mut usb_serial,
        &PacketFormat {
            to: None,
            from: None,
            data: PacketData::LogMessage(LogMessage {
                level: LogLevel::Info,
                event: String::from_str("mc_boot").unwrap(),
                json: None,
            }),
            time: Instant::now().duration_since_epoch().as_micros(),
            id: 0,
        },
    )
    .ok();

    let mut message = PacketFormat {
        to: None,
        from: None,
        data: PacketData::ClockRequest(ClockRequest { request_time: 0 }),
        time: 0,
        id: 0,
    };

    let mut packet_errors: u32 = 0;
    let mut packet_send_errors: u32 = 0;
    let mut time_offset: Option<u64> = None;

    loop {
        let loopStartTime = Instant::now();
        if (loopStartTime - lastWriteTime) > Duration::from_millis(500) {
            message.time = Instant::now().duration_since_epoch().as_micros();
            send_message(&mut usb_serial, &message).ok();
            message.id = message.id.wrapping_add(1);

            lastWriteTime = loopStartTime;
            led.toggle();
        }

        while let Ok(byte) = usb_serial.read_byte() {
            if let Some(mut packet) = packet_finder.push_byte(byte)
                && !packet.is_empty()
            {
                match packet_encoding::decode_packet::<PacketFormat>(&mut packet) {
                    Ok(packet) => {
                        match packet.data {
                            PacketData::ClockResponse(resp) => {
                                let current_time = Instant::now().duration_since_epoch().as_micros();
                                let round_trip_time = current_time - packet.time;
                                let estimated_offset = ((packet.time + round_trip_time / 2) as i64)
                                    - (resp.recieved_time as i64);

                                if let Some(offset) = time_offset {
                                    let new_offset =
                                        (offset as i64 * 7 + estimated_offset) / 8;
                                    time_offset = Some(new_offset as u64);
                                } else {
                                    time_offset = Some(estimated_offset as u64);
                                }
                                send_message(
                                    &mut usb_serial,
                                    &PacketFormat {
                                        to: None,
                                        from: None,
                                        data: PacketData::LogMessage(LogMessage {
                                            level: LogLevel::Info,
                                            event: String::from_str("time_sync").unwrap(),
                                            json: format!(
                                                    "{{\"offset\": {}, \"rtt\": {}}}",
                                                    estimated_offset, round_trip_time
                                                ).ok(),
                                        }),
                                        time: Instant::now().duration_since_epoch().as_micros(),
                                        id: 0,
                                    },
                                )
                                .ok();

                            }
                            _ => {}
                        }   
                    }
                    Err(_e) => {
                        packet_errors = packet_errors.wrapping_add(1);
                    }
                }
            }
        }
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
