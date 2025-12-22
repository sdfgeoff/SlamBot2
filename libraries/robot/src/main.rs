use serial::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use packet_router::Client;

use std::time::Duration;
mod serial_client;
use serial_client::SerialClient;
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
    log_client.borrow_mut().subscriptions.push("all".to_string());

    router.register_client(Rc::downgrade(&serial_client.client));
    router.register_client(Rc::downgrade(&log_client));

    loop {
        serial_client.read();
        router.poll();
        serial_client.write();

        let log_packets: Vec<Rc<PacketFormat>> = log_client.borrow_mut().router_to_client.drain(..).collect();
        for packet in log_packets {
            println!("Log Packet: Topic: {}, ID: {}, Time: {}", packet.topic, packet.id, packet.time);
        }
    }
}
