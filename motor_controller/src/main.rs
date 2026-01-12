#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::str::FromStr;

use esp_hal::gpio::DriveMode;
use esp_hal::ledc::channel::ChannelIFace;
use esp_hal::ledc::timer::TimerIFace;
use esp_hal::ledc::{LSGlobalClkSource, Ledc, LowSpeed, channel, timer};
use esp_hal::main;
use esp_hal::time::{Duration, Instant, Rate};
use libm::{cosf, sinf};

use esp_hal::gpio::{Input, InputConfig, Io, Level, Output, OutputConfig};

esp_bootloader_esp_idf::esp_app_desc!();

use heapless::{String, Vec, format};

use topics::*;

mod clock;
use clock::Clock;

mod host_connection;
use host_connection::{HostConnection, NonBlockingJtagUart};

mod encoders;
use encoders::{ENCODER_STATE, Encoder, Encoders};

mod motor_controller;
use motor_controller::{MotorControllers, MotorDriver};

mod packet_data;
use packet_data::PacketData;

mod consts;
use consts::{WHEEL_CIRCUMFERENCE, WHEEL_BASE_WIDTH, ENCODER_TICKS_PER_REVOLUTION};

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let mut clock = Clock::new();
    let mut lastClockSyncTime = Instant::now();
    let mut lastEncoderSendTime = Instant::now();

    let mut host_connection = HostConnection::new(NonBlockingJtagUart::new(
        peripherals.USB_DEVICE,
        Duration::from_millis(100),
    ));

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

    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer1);
    lstimer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty5Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_khz(24),
        })
        .expect("Failed to configure LEDC timer");

    let config = channel::config::Config {
        timer: &lstimer0,
        duty_pct: 0,
        drive_mode: DriveMode::PushPull,
    };

    let mut motor_controllers = MotorControllers {
        left: MotorDriver {
            a: ledc.channel(
                channel::Number::Channel0,
                Output::new(peripherals.GPIO0, Level::High, OutputConfig::default()),
            ),
            b: ledc.channel(
                channel::Number::Channel1,
                Output::new(peripherals.GPIO1, Level::High, OutputConfig::default()),
            ),
        },
        right: MotorDriver {
            a: ledc.channel(
                channel::Number::Channel2,
                Output::new(peripherals.GPIO3, Level::High, OutputConfig::default()),
            ),
            b: ledc.channel(
                channel::Number::Channel3,
                Output::new(peripherals.GPIO4, Level::High, OutputConfig::default()),
            ),
        },
        set_velocity_time: Instant::now(),
    };

    motor_controllers
        .left
        .a
        .configure(config)
        .expect("Failed to configure LEDC channel 0");
    motor_controllers
        .left
        .b
        .configure(config)
        .expect("Failed to configure LEDC channel 1");
    motor_controllers
        .right
        .a
        .configure(config)
        .expect("Failed to configure LEDC channel 2");
    motor_controllers
        .right
        .b
        .configure(config)
        .expect("Failed to configure LEDC channel 3");


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

    let mut odometryTracker = OdometryDelta {
        start_time: clock.get_time(),
        end_time: clock.get_time(),
        delta_position: [0.0, 0.0],
        delta_orientation: 0.0,
    };

    loop {
        let (left_count, right_count) = critical_section::with(|cs| {
            if let Some(encoders) = ENCODER_STATE.borrow(cs).borrow_mut().as_mut() {
                let left = encoders.left.count;
                let right = encoders.right.count;
                encoders.left.count = 0;
                encoders.right.count = 0;
                (left, right)
            } else {
                (0, 0)
            }
        });
        update_odometry(&mut odometryTracker, left_count, right_count);
        motor_controllers.tick();

        let loop_start_time = Instant::now();
        if lastClockSyncTime.elapsed() >= Duration::from_secs(1) {
            host_connection
                .send_packet(&clock, clock.generate_request_data(), None)
                .unwrap_or_else(|_e| {
                    packet_send_errors = packet_send_errors.wrapping_add(1);
                });
            lastClockSyncTime = loop_start_time;
            led.toggle();
        }
        if lastEncoderSendTime.elapsed() >= Duration::from_millis(100) {
            odometryTracker.end_time = clock.get_time();
            host_connection
                .send_packet(
                    &clock,
                    PacketData::OdometryDelta(odometryTracker.clone()),
                    None,
                )
                .unwrap_or_else(|_e| {
                    packet_send_errors = packet_send_errors.wrapping_add(1);
                });
            odometryTracker.start_time = odometryTracker.end_time;
            odometryTracker.delta_position = [0.0, 0.0];
            odometryTracker.delta_orientation = 0.0;

            lastEncoderSendTime = loop_start_time;
        }
        while let Some(packet) = host_connection.step() {
            match packet.data {
                PacketData::ClockResponse(resp) => {
                    let round_trip_time = clock.handle_clock_response(&resp);
                    let mut values: Vec<topics::DiagnosticKeyValue, 8> = Vec::new();
                    values
                        .push(diag_value("offset", &clock.offset.unwrap_or(0)))
                        .ok();
                    values.push(diag_value("rtt", &round_trip_time)).ok();
                    host_connection
                        .send_packet(
                            &clock,
                            PacketData::DiagnosticMsg(topics::DiagnosticMsg {
                                level: DiagnosticStatus::Ok,
                                name: String::from_str("time_sync").unwrap(),
                                message: String::from_str("").unwrap(),
                                values,
                            }),
                            None,
                        )
                        .ok();
                }
                PacketData::MotionVelocityRequest(req) => {
                    motor_controllers.handle_speed_request(&req);
                }
                _ => {}
            }
        }
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

fn diag_value(key: &str, value: &impl core::fmt::Display) -> topics::DiagnosticKeyValue {
    topics::DiagnosticKeyValue {
        key: String::from_str(key).unwrap(),
        value: format!("{}", value).unwrap(),
    }
}

fn update_odometry(odometry: &mut OdometryDelta, left_count: i64, right_count: i64) {
    let left_distance = -left_count as f32 * WHEEL_CIRCUMFERENCE / ENCODER_TICKS_PER_REVOLUTION;
    let right_distance = right_count as f32 * WHEEL_CIRCUMFERENCE / ENCODER_TICKS_PER_REVOLUTION;
    let delta_distance = (left_distance + right_distance) / 2.0;
    let delta_theta = (right_distance - left_distance) / WHEEL_BASE_WIDTH;

    odometry.delta_position[0] += delta_distance * cosf(odometry.delta_orientation);
    odometry.delta_position[1] += delta_distance * sinf(odometry.delta_orientation);
    odometry.delta_orientation += delta_theta;
}
