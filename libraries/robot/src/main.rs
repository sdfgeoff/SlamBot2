use serial::prelude::*;
use std::rc::Rc;

use std::time::Duration;
mod serial_client;
use serial_client::SerialClient;
use topics::PacketFormat;

fn main() {
    println!("Hello, world!");

    let mut router = packet_router::Router::<PacketFormat>::new();

    let device_path = "/dev/serial/by-id/usb-Espressif_USB_JTAG_serial_debug_unit_98:3D:AE:52:AD:78-if00";
    let mut serialport = serial::open(&device_path).expect("Failed to open serial port");
    serialport.set_timeout(Duration::from_millis(0)).expect("Failed to set timeout");

    let mut serial_client = SerialClient::<PacketFormat, _>::new(serialport);

    router.register_client(Rc::downgrade(&serial_client.client));

    loop {
        serial_client.read();
        router.poll();
        serial_client.write();
    }   

}
