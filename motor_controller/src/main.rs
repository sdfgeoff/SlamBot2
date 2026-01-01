#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::str::FromStr;

// use esp_backtrace as _;
use esp_hal::main;
use esp_hal::time::{Duration, Instant};

use esp_hal::gpio::{Level, Output, OutputConfig, Input, InputConfig, Io};
use esp_hal::usb_serial_jtag::UsbSerialJtag;

esp_bootloader_esp_idf::esp_app_desc!();

use heapless::{String, Vec};

use topics::*;

mod clock;
use clock::Clock;

mod host_connection;
use host_connection::HostConnection;

mod encoders;
use encoders::{Encoders, Encoder, ENCODER_STATE};

mod motor_controller;

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());


    let mut clock = Clock::new();
    let mut lastClockSyncTime = Instant::now();
    let mut lastEncoderSendTime = Instant::now();

    let mut host_connection = HostConnection::new(UsbSerialJtag::new(peripherals.USB_DEVICE));


    let mut led = Output::new(peripherals.GPIO8, Level::High, OutputConfig::default());


    let mut io = Io::new(peripherals.IO_MUX);
    io.set_interrupt_handler(encoders::encoder_interrupt_handler);


    let mut encoders = Encoders {
        left: Encoder {
            a_input: Input::new(peripherals.GPIO20, InputConfig::default()),
            b_input: Input::new(peripherals.GPIO21, InputConfig::default()),
            count: 0,
        },
        right: Encoder {
            a_input: Input::new(peripherals.GPIO7, InputConfig::default()),
            b_input: Input::new(peripherals.GPIO6, InputConfig::default()),
            count: 0,
        },
    };
    encoders.configure();
    critical_section::with(|cs| {
        ENCODER_STATE.borrow(cs).replace(Some(encoders));
    });

    // Send boot message
    host_connection
        .send_packet(
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
            host_connection
                .send_packet(&clock, clock.generate_request_data(), None)
                .unwrap_or_else(|_e| {
                    packet_send_errors = packet_send_errors.wrapping_add(1);
                });
            lastClockSyncTime = loopStartTime;
            led.toggle();
        }
        if lastEncoderSendTime.elapsed() >= Duration::from_millis(100) {
            let (left_count, right_count) = critical_section::with(|cs| {
                if let Some(encoders) = ENCODER_STATE.borrow(cs).borrow_mut().as_mut() {
                    let left = encoders.left.count;
                    let right = encoders.right.count;
                    (left, right)
                } else {
                    (0, 0)
                }
            });

            let mut values: Vec<DiagnosticKeyValue, 8> = Vec::new();
            values
                .push(DiagnosticKeyValue {
                    key: String::from_str("left").unwrap(),
                    value: heapless::format!("{}", left_count).unwrap(),
                })
                .ok();
            values
                .push(DiagnosticKeyValue {
                    key: String::from_str("right").unwrap(),
                    value: heapless::format!("{}", right_count).unwrap(),
                })
                .ok();

            host_connection
                .send_packet(
                    &clock,
                    PacketData::DiagnosticMsg(DiagnosticMsg {
                        level: DiagnosticStatus::Ok,
                        name: String::from_str("encoder_counts").unwrap(),
                        message: String::from_str("").unwrap(),
                        values,
                    }),
                    None,
                )
                .unwrap_or_else(|_e| {
                    packet_send_errors = packet_send_errors.wrapping_add(1);
                });

            lastEncoderSendTime = loopStartTime;
        }
        host_connection.step(&mut clock);
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
    }
}
