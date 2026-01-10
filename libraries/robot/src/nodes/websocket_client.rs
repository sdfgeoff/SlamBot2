use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::thread::spawn;
use tungstenite::{WebSocket, accept};

use packet_trait::PacketTrait;
use packet_encoding::{PacketFinder, decode_packet, encode_packet};
use packet_router::{Client, Router};
use serde::Serialize;
use serial::SerialPort;
use std::rc::Rc;
use std::{cell::RefCell, collections::HashSet};




pub struct WebsocketClient<T: PacketTrait> {
    pub client: Client<T>,
    pub websocket: WebSocket<TcpStream>,
}

impl<T: PacketTrait> WebsocketClient<T> {
    pub fn new(websocket: WebSocket<TcpStream>) -> Self {
        WebsocketClient {
            client: Client::<T>::default(),
            websocket,
        }
    }

    pub fn tick(&mut self) {
        // Read from websocket into incoming queue
        if let Ok(msg) = self.websocket.read() {

            // We do not want to send back ping/pong messages.
            if msg.is_binary() || msg.is_text() {
                // For simplicity, we just echo the message back.
                println!("Received message: {:?}", msg);
                self.websocket.send(msg).unwrap();
            }
        }
    }
}




pub struct WebsocketAcceptor<T: PacketTrait> {
    pub router: Rc<RefCell<Router<T>>>,
    pub clients_by_ip: HashMap<String, WebsocketClient<T>>,
    pub server: TcpListener,
}


impl<T: PacketTrait> WebsocketAcceptor<T> {
    pub fn new(router: Rc<RefCell<Router<T>>>, address: &str) -> Self {
        let server = TcpListener::bind(address).unwrap();
        println!("Websocket server listening on {}", address);
        server.set_nonblocking(true).expect("Failed to set non-blocking");
        WebsocketAcceptor {
            router,
            clients_by_ip: HashMap::new(),
            server,
        }
    }

    pub fn tick(&mut self) {
        let websocket = self.server.accept();
        if let Ok((stream, addr)) = websocket {
            stream.set_nonblocking(true).expect("Failed to set non-blocking");
            if let(Ok(websocket))= accept(stream){
                let peer_addr = addr.to_string();
                self.clients_by_ip.insert(
                    peer_addr.clone(),
                    WebsocketClient::new(websocket),
                );
                println!("New websocket client connected: {}", peer_addr);
            }
        }

        for (ip, client) in self.clients_by_ip.iter_mut() {
            client.tick();
        }


    }
}