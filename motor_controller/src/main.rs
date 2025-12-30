#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]


use core::str::FromStr;

// use esp_backtrace as _;
use esp_hal::{Blocking, main};
use esp_hal::time::{Instant, Duration};


use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use packet_encoding::{PacketEncodeErr, encode_packet};

esp_bootloader_esp_idf::esp_app_desc!();

use heapless::{String, Vec, format};
use core::cell::RefCell;

use topics::*;

mod clock;
use clock::Clock;

mod host_connection;
use host_connection::HostConnection;





#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let mut led = Output::new(peripherals.GPIO8, Level::High, OutputConfig::default());


    let mut clock = Clock::new();
    let mut lastClockSyncTime = Instant::now();

    let mut host_connection = HostConnection::new(
        UsbSerialJtag::new(peripherals.USB_DEVICE)
    );

    // Send boot message
    host_connection.send_packet(
        &clock,
        PacketData::DiagnosticMsg(DiagnosticMsg {
            level: DiagnosticStatus::Ok,
            name: String::from_str("mc_boot").unwrap(),
            message: String::from_str("").unwrap(),
            values: Vec::new(),
        }),
        None,
    )
    .ok();

    let mut packet_send_errors: u32 = 0;

    loop {
        let loopStartTime = Instant::now();
        if lastClockSyncTime.elapsed() >= Duration::from_secs(5) {
            host_connection.send_packet(
                &clock,
                clock.generate_request_data(),
                None,
            ).unwrap_or_else(|_e| {
                packet_send_errors = packet_send_errors.wrapping_add(1);
            });
            lastClockSyncTime = loopStartTime;
            led.toggle();
        }
        host_connection.step(&mut clock);
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
