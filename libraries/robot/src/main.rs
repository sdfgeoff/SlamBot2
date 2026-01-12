use serial::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use core::str::FromStr;
use std::time::Duration;
mod nodes;
use heapless::String;
use nodes::clock::{Clock, get_current_time};
use nodes::log::Log;
use nodes::serial_client::SerialClient;
use nodes::websocket_client::WebsocketAcceptor;

use topics::{PacketData, PacketFormat};

fn main() {
    println!("Hello, world!");
    let router_raw = packet_router::Router::<PacketFormat<PacketData>>::new();
    let router = Rc::new(RefCell::new(router_raw));

    //let device_path = "/dev/serial/by-id/usb-Espressif_USB_JTAG_serial_debug_unit_98:3D:AE:52:AD:78-if00";
    let device_path =
        "/dev/serial/by-id/usb-Espressif_USB_JTAG_serial_debug_unit_98:3D:AE:50:AA:F8-if00";
    let mut serialport = serial::open(&device_path).expect("Failed to open serial port");
    serialport
        .set_timeout(Duration::from_millis(1))
        .expect("Failed to set timeout");

    let mut serial_client = SerialClient::new(serialport);
    let mut log_client = Log::new(false);
    router.borrow_mut().register_client(Rc::downgrade(&serial_client.client));
    router.borrow_mut().register_client(Rc::downgrade(&log_client.client));

    let mut clock_node = Clock::new();
    router.borrow_mut().register_client(Rc::downgrade(&clock_node.client));

    let mut websocket_acceptor = WebsocketAcceptor::new(Rc::clone(&router), "127.0.0.1:9001");


    loop {
        clock_node.tick();
        router.borrow_mut().poll();
        // log_client.step();
        websocket_acceptor.tick();
        serial_client.tick();
    }
}
