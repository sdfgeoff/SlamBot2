#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use esp_hal::time::{Duration, Instant};
// use esp_backtrace as _;
use esp_hal::{
    Blocking, main
};

use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::gpio::{Level, Output, OutputConfig};
use packet_encoding::{encode, PacketEncodeErr};

esp_bootloader_esp_idf::esp_app_desc!();

use heapless::Vec;
use serde::Serialize;

#[derive(Serialize)]
struct User {
    user_id: u32,
    password_hash: [u8; 4],
}

#[derive(Serialize)]
struct CommandMessage {
    uri: Vec<u8, 30>,
    command: Vec<u8, 50>,
}

#[derive(Serialize)]
enum MessageData {
    Command(CommandMessage),
    Response(Vec<u8, 80>),
}



#[derive(Serialize)]
struct Message {
    id: u32, // Unique identifier for the message
    topic: Vec<u8, 10>, // Topic of the message
    data: MessageData, // The actual message data
    time: u64, // Timestamp of when the message was created, in microseconds since epoch 
    seq: u8, // Sequence number, increments with each message on this particular topic
    from: u16, // Address of who the message is from
    to: u16, // Address of who the message is to
}




fn send_message(usb: &mut UsbSerialJtag<Blocking>, message: &(impl Serialize)) -> Result<(), PacketEncodeErr> {
    let mut encode_buffer = [0u8; 600];
    encode_buffer[0] = 0; // COBS initial byte
    let encoded_size = encode(message, &mut encode_buffer[1..])?;
    encode_buffer[encoded_size + 1] = 0x00; // COBS final byte

    usb.write(b"Start{").unwrap();
    usb.write(&encode_buffer[..encoded_size + 2]).unwrap();
    usb.write(b"}End\r\n").unwrap();

    Ok(())
}



#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let mut led = Output::new(peripherals.GPIO8, Level::High, OutputConfig::default());


    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);

    let mut lastWriteTime = Instant::now();

    let mut message = Message {
        id: 1,
        topic: Vec::from_slice(b"auth").unwrap(),
        data: MessageData::Command(CommandMessage {
            uri: Vec::from_slice(b"/login").unwrap(),
            command: Vec::from_slice(b"request").unwrap(),
        }),
        time: 0,
        seq: 0,
        from: 0,
        to: 0,
    };


    loop {
        let loopStartTime = Instant::now();
        if (loopStartTime - lastWriteTime) > Duration::from_millis(500) {
            message.time = Instant::now().duration_since_epoch().as_micros();
            send_message(&mut usb_serial, &message).unwrap();
            message.seq = message.seq.wrapping_add(1);
            message.id = message.id.wrapping_add(1);

            lastWriteTime = loopStartTime;
            led.toggle();
        }

        match usb_serial.read_byte() {
            Ok(byte) => {
                usb_serial.write(&[byte]).unwrap();
                usb_serial.write(b"!").unwrap();
            }
            Err(_) => {
            }
        }
    }
}


#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
