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

use heapless::String;

use topics::*;

fn send_message(
    usb: &mut UsbSerialJtag<Blocking>,
    message: &PacketFormat,
) -> Result<(), PacketEncodeErr> {
    let mut encode_buffer = [0u8; 600];
    encode_buffer[0] = 0; // COBS initial byte
    let encoded_size = encode_packet(message, &mut encode_buffer[1..])?;
    encode_buffer[encoded_size + 1] = 0x00; // COBS final byte
    usb.write(&encode_buffer[..encoded_size + 2]).unwrap();
    Ok(())
}

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let mut led = Output::new(peripherals.GPIO8, Level::High, OutputConfig::default());

    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);

    let mut lastWriteTime = Instant::now();

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

    loop {
        let loopStartTime = Instant::now();
        if (loopStartTime - lastWriteTime) > Duration::from_millis(500) {
            message.time = Instant::now().duration_since_epoch().as_micros();
            send_message(&mut usb_serial, &message).unwrap();
            message.id = message.id.wrapping_add(1);

            lastWriteTime = loopStartTime;
            led.toggle();
        }

        match usb_serial.read_byte() {
            Ok(byte) => {
                usb_serial.write(&[byte]).unwrap();
                usb_serial.write(b"!").unwrap();
            }
            Err(_) => {}
        }
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
