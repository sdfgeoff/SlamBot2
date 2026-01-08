use serial::prelude::*;
use std::rc::Rc;

use core::str::FromStr;
use std::time::Duration;
mod nodes;
use heapless::String;
use nodes::clock::{Clock, get_current_time};
use nodes::log::Log;
use nodes::serial_client::SerialClient;

use topics::{PacketFormat, PacketData};

fn main() {
    println!("Hello, world!");
    let mut router = packet_router::Router::<PacketFormat<PacketData>>::new();

    //let device_path = "/dev/serial/by-id/usb-Espressif_USB_JTAG_serial_debug_unit_98:3D:AE:52:AD:78-if00";
    let device_path =
        "/dev/serial/by-id/usb-Espressif_USB_JTAG_serial_debug_unit_98:3D:AE:50:AA:F8-if00";
    let mut serialport = serial::open(&device_path).expect("Failed to open serial port");
    serialport
        .set_timeout(Duration::from_millis(1))
        .expect("Failed to set timeout");

    let mut serial_client = SerialClient::new(serialport);
    let mut log_client = Log::new(true);
    router.register_client(Rc::downgrade(&serial_client.client));
    router.register_client(Rc::downgrade(&log_client.client));

    let mut clock_node = Clock::new();
    router.register_client(Rc::downgrade(&clock_node.client));

    let mut serial_stats_last_sent = std::time::Instant::now();

    loop {
        serial_client.read();
        clock_node.tick();
        router.poll();
        log_client.step();
        serial_client.write();

        if serial_stats_last_sent.elapsed().as_secs() >= 5 {
            let mut values = heapless::Vec::new();
            // serde_json::to_string(&serial_client.stats).unwrap();
            values
                .push(topics::DiagnosticKeyValue {
                    key: String::from_str("decode_errors").unwrap(),
                    value: heapless::format!("{}", serial_client.stats.decode_error_count).unwrap(),
                })
                .ok();
            values
                .push(topics::DiagnosticKeyValue {
                    key: String::from_str("tx_packets").unwrap(),
                    value: heapless::format!("{}", serial_client.stats.tx_packets).unwrap(),
                })
                .ok();
            values
                .push(topics::DiagnosticKeyValue {
                    key: String::from_str("tx_bytes").unwrap(),
                    value: heapless::format!("{}", serial_client.stats.tx_bytes).unwrap(),
                })
                .ok();
            values
                .push(topics::DiagnosticKeyValue {
                    key: String::from_str("rx_packets").unwrap(),
                    value: heapless::format!("{}", serial_client.stats.rx_packets).unwrap(),
                })
                .ok();
            values
                .push(topics::DiagnosticKeyValue {
                    key: String::from_str("rx_bytes").unwrap(),
                    value: heapless::format!("{}", serial_client.stats.rx_bytes).unwrap(),
                })
                .ok();
            values
                .push(topics::DiagnosticKeyValue {
                    key: String::from_str("encode_errors").unwrap(),
                    value: heapless::format!("{}", serial_client.stats.encode_error_count).unwrap(),
                })
                .ok();
            values
                .push(topics::DiagnosticKeyValue {
                    key: String::from_str("write_errors").unwrap(),
                    value: heapless::format!("{}", serial_client.stats.write_error_count).unwrap(),
                })
                .ok();

            serial_client
                .client
                .borrow_mut()
                .client_to_router
                .push(PacketFormat {
                    to: None,
                    from: None,
                    data: topics::PacketData::DiagnosticMsg(topics::DiagnosticMsg {
                        level: topics::DiagnosticStatus::Ok,
                        name: String::try_from("serial_stats").unwrap(),
                        message: String::from_str("").unwrap(),
                        values,
                    }),
                    time: get_current_time(),
                    id: 0,
                });
            serial_stats_last_sent = std::time::Instant::now();
        }
    }
}
