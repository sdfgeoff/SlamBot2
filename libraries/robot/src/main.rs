use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

mod nodes;
use nodes::clock::Clock;
use nodes::log::Log;
use nodes::serial_adapter::SerialAdapter;
use nodes::websocket_client::WebsocketAcceptor;
use nodes::position_estimator::PositionEstimator;

use topics::{PacketData, PacketFormat};

fn main() {
    println!("Hello, world!");
    let router_raw = packet_router::Router::<PacketFormat<PacketData>>::new();
    let router = Rc::new(RefCell::new(router_raw));

    let mut serial_adapter = SerialAdapter::new(Rc::clone(&router), Duration::from_secs(2));

    let mut log_client = Log::new(false);
    router.borrow_mut().register_client(Rc::downgrade(&log_client.client));

    let mut clock_node = Clock::new();
    router.borrow_mut().register_client(Rc::downgrade(&clock_node.client));

    let mut websocket_acceptor = WebsocketAcceptor::new(Rc::clone(&router), "127.0.0.1:9001");
    
    let mut position_estimator = PositionEstimator::new();
    router
        .borrow_mut()
        .register_client(Rc::downgrade(&position_estimator.client));

    loop {
        clock_node.tick();
        router.borrow_mut().poll();
        log_client.step();
        websocket_acceptor.tick();
        serial_adapter.tick();
        position_estimator.tick();
    }
}
