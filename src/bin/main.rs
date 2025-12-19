// #![no_std]
// #![no_main]
// #![deny(
//     clippy::mem_forget,
//     reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
//     holding buffers for the duration of a data transfer."
// )]

// use esp_hal::clock::CpuClock;
// use esp_hal::gpio::{Level, Output, OutputConfig};
// use esp_hal::main;
// use esp_hal::time::{Duration, Instant};
// use esp_hal::timer::timg::TimerGroup;
// use esp_println::println;
// use std::{
//     io::{stdin, stdout, Write},
//     ptr::null_mut,
//     thread::spawn,
// };
// extern crate alloc;

// // This creates a default app-descriptor required by the esp-idf bootloader.
// // For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
// esp_bootloader_esp_idf::esp_app_desc!();

// #[main]
// fn main() -> ! {
//     // generator version: 1.0.1

//     let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
//     let peripherals = esp_hal::init(config);

//     esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 66320);

//     let timg0 = TimerGroup::new(peripherals.TIMG0);
//     let sw_interrupt =
//         esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
//     esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);
//     let radio_init = esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller");
//     let (mut _wifi_controller, _interfaces) =
//         esp_radio::wifi::new(&radio_init, peripherals.WIFI, Default::default())
//             .expect("Failed to initialize Wi-Fi controller");

//     // Set GPIO0 as an output, and set its state high initially.
//     let mut led = Output::new(peripherals.GPIO8, Level::High, OutputConfig::default());

//     let reader = stdin();
//     let mut writer = stdout();

//     loop {
//         let delay_start = Instant::now();
//         while delay_start.elapsed() < Duration::from_millis(500) {}
//         println!("Hello, world!");
//         led.toggle();

//         let mut line = String::new();
//         match reader.read_line(&mut line) {
//             Ok(_) => {
//                 let line_trimmed = line.trim_end_matches(&['\r', '\n']);
//                 writer.write(line_trimmed.as_bytes()).unwrap();
//                 writer.write(b"\r\n").unwrap();
//             }
//             Err(e) => {
//                 println!("Error: {e}\r\n");
//                 loop {
//                     unsafe { vTaskDelay(1000) };
//                 }
//             }
//         }
//     }

//     // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples/src/bin
// }



//! UART loopback test
//!
//! Folowing pins are used:
//! TX    GPIO12
//! RX    GPIO13
//!
//! Depending on your target and the board you are using you have to change the pins.
//!
//! This example transfers data via UART.
//! Connect TX and RX pins to see the outgoing data is read as incoming data.




//! CDC-ACM serial port example using polling in a busy loop.
//!
//! This example should be built in release mode.
//!
//! The following wiring is assumed:
//! - DP => GPIO20
//! - DM => GPIO19

#![no_std]
#![no_main]

use cobs::{CobsEncoder, encode};
use crc16::{ARC, State};
use esp_hal::time::{Duration, Instant};
// use esp_backtrace as _;
use esp_hal::{
    Blocking, main
};

static mut EP_MEMORY: [u32; 1024] = [0; 1024];

use esp_hal::usb_serial_jtag::UsbSerialJtag;

esp_bootloader_esp_idf::esp_app_desc!();

use serde::Serialize;
use serde_cbor::Serializer;
use serde_cbor::ser::SliceWrite;
use heapless::Vec;

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




fn send_message(usb: &mut UsbSerialJtag<Blocking>, message: &(impl Serialize)) -> Result<(), serde_cbor::Error> {
    let mut serialize_buffer = [0u8; 500];
    let mut encode_buffer = [0u8; 600];
    
    let writer = SliceWrite::new(&mut serialize_buffer[..]);
    let mut ser = Serializer::new(writer);
    message.serialize(&mut ser)?;
    let writer = ser.into_inner();
    let size = writer.bytes_written();
    let serialized = &serialize_buffer[..size];
    let crc = State::<ARC>::calculate(&serialize_buffer);

    let mut encoder = CobsEncoder::new(&mut encode_buffer);
    encoder.push(serialized).unwrap();
    encoder.push(&crc.to_le_bytes()).unwrap();
    let encoded_size = encoder.finalize();

    usb.write(b"Start{").unwrap();
    usb.write(&[0u8]).unwrap();
    usb.write(&encode_buffer[..encoded_size]).unwrap();
    usb.write(&[0u8]).unwrap();
    usb.write(b"}End\r\n").unwrap();

    Ok(())
}



#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();
    let peripherals = esp_hal::init(esp_hal::Config::default());

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
