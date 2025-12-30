use serial::prelude::*;
use std::rc::Rc;

use std::time::Duration;
mod nodes;
use nodes::clock::{Clock, get_current_time};
use nodes::log::Log;
use nodes::serial_client::SerialClient;
use heapless::String;

use topics::PacketFormat;

fn main() {
    println!("Hello, world!");
    let mut router = packet_router::Router::<PacketFormat>::new();

    let device_path =
        "/dev/serial/by-id/usb-Espressif_USB_JTAG_serial_debug_unit_98:3D:AE:52:AD:78-if00";
    let mut serialport = serial::open(&device_path).expect("Failed to open serial port");
    serialport
        .set_timeout(Duration::from_millis(1))
        .expect("Failed to set timeout");

    let mut serial_client = SerialClient::<PacketFormat, _>::new(serialport);
    let mut log_client = Log::new(false);
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
            let stats = serde_json::to_string(&serial_client.stats).unwrap();
            serial_client
                .client
                .borrow_mut()
                .client_to_router
                .push(PacketFormat {
                    to: None,
                    from: None,
                    data: topics::PacketData::LogMessage(topics::LogMessage {
                        level: topics::LogLevel::Info,
                        event: String::try_from("serial_stats").unwrap(),
                        json: Some(
                            String::<256>::try_from(stats.as_str()).unwrap(),
                        ),
                    }),
                    time: get_current_time(),
                    id: 0,
                });
            serial_stats_last_sent = std::time::Instant::now();
        }


    }
}
