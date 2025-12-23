use packet_router::Client;
use serial::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use std::time::Duration;
mod nodes;
use nodes::clock::Clock;
use nodes::serial_client::SerialClient;

use packet_trait::PacketTrait;
use topics::PacketFormat;

fn main() {
    println!("Hello, world!");

    let mut router = packet_router::Router::<PacketFormat>::new();

    let device_path =
        "/dev/serial/by-id/usb-Espressif_USB_JTAG_serial_debug_unit_98:3D:AE:52:AD:78-if00";
    let mut serialport = serial::open(&device_path).expect("Failed to open serial port");
    serialport
        .set_timeout(Duration::from_millis(0))
        .expect("Failed to set timeout");

    let mut serial_client = SerialClient::<PacketFormat, _>::new(serialport);
    let log_client = Rc::new(RefCell::new(Client::<PacketFormat>::default()));
    log_client
        .borrow_mut()
        .subscriptions
        .push("all".to_string());

    router.register_client(Rc::downgrade(&serial_client.client));
    router.register_client(Rc::downgrade(&log_client));

    let mut clock_node = Clock::new();
    router.register_client(Rc::downgrade(&clock_node.client));

    loop {
        serial_client.read();
        clock_node.tick();
        router.poll();

        let log_packets: Vec<Rc<PacketFormat>> =
            log_client.borrow_mut().router_to_client.drain(..).collect();
        for packet in log_packets {
            println!(
                "Log Packet: Topic: {}, ID: {}, Time: {}",
                packet.get_topic(),
                packet.id,
                packet.time
            );
        }
        serial_client.write();
    }
}
