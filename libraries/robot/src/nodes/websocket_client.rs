use std::collections::{HashMap, HashSet};
use std::net::{TcpListener, TcpStream};
use std::rc::Rc;
use std::cell::RefCell;
use std::time::Instant;
use topics::{DiagnosticMsg, PacketData, PacketFormat, SubscriptionRequest};
use tungstenite::{WebSocket, accept};
use serde::{Serialize};

use packet_encoding::decode_packet;
use packet_router::{Client, Router};
use heapless::{String as HString, format as hformat};
use std::str::FromStr;

use crate::nodes::clock::get_current_time;

#[derive(Serialize)]
pub struct WebsocketClientStats {
    pub decode_error_count: u32,
    pub tx_packets: u32,
    pub tx_bytes: u32,
    pub rx_packets: u32,
    pub rx_bytes: u32,
    pub encode_error_count: u32,
    pub write_error_count: u32,
}


impl WebsocketClientStats {
    fn to_log(&self) -> DiagnosticMsg {
        let mut values = heapless::Vec::<topics::DiagnosticKeyValue, 8>::new();
        values
            .push(topics::DiagnosticKeyValue {
                key: HString::from_str("decode_errors").unwrap(),
                value: hformat!("{}", self.decode_error_count).unwrap(),
            })
            .ok();
        values
            .push(topics::DiagnosticKeyValue {
                key: HString::from_str("tx_packets").unwrap(),
                value: hformat!("{}", self.tx_packets).unwrap(),
            })
            .ok();
        values
            .push(topics::DiagnosticKeyValue {
                key: HString::from_str("tx_bytes").unwrap(),
                value: hformat!("{}", self.tx_bytes).unwrap(),
            })
            .ok();
        values
            .push(topics::DiagnosticKeyValue {
                key: HString::from_str("rx_packets").unwrap(),
                value: hformat!("{}", self.rx_packets).unwrap(),
            })
            .ok();
        values
            .push(topics::DiagnosticKeyValue {
                key: HString::from_str("rx_bytes").unwrap(),
                value: hformat!("{}", self.rx_bytes).unwrap(),
            })
            .ok();
        values
            .push(topics::DiagnosticKeyValue {
                key: HString::from_str("encode_errors").unwrap(),
                value: hformat!("{}", self.encode_error_count).unwrap(),
            })
            .ok();
        values
            .push(topics::DiagnosticKeyValue {
                key: HString::from_str("write_errors").unwrap(),
                value: hformat!("{}", self.write_error_count).unwrap(),
            })
            .ok();

        DiagnosticMsg {
            level: topics::DiagnosticStatus::Ok,
            name: HString::from_str("websocket_stats").unwrap(),
            message: HString::from_str("").unwrap(),
            values,
        }
    }
}


pub struct WebsocketClient {
    pub client: Rc<RefCell<Client<PacketFormat<PacketData>>>>,
    pub websocket: WebSocket<TcpStream>,

    pub stats: WebsocketClientStats,
    pub stats_send_time: Instant,

    pub is_alive: bool,
}


impl WebsocketClient {
    pub fn new(websocket: WebSocket<TcpStream>) -> Self {
        let client = Rc::new(RefCell::new(Client::<PacketFormat<PacketData>>::default()));
        WebsocketClient {
            client,
            websocket,
            stats: WebsocketClientStats {
                decode_error_count: 0,
                tx_packets: 0,
                tx_bytes: 0,
                rx_packets: 0,
                rx_bytes: 0,
                encode_error_count: 0,
                write_error_count: 0,
            },
            stats_send_time: Instant::now(),
            is_alive: true,
        }
    }

    pub fn update_topics(&mut self, sub_req: &SubscriptionRequest) {
        let topics_set = HashSet::<String>::from_iter(sub_req.topics.iter().map(|s| s.to_string()));
        if topics_set
            .symmetric_difference(&self.client.borrow().subscriptions)
            .count()
            > 0
        {
            self.client.borrow_mut().subscriptions = topics_set;
        }
    }

    pub fn tick(&mut self) {
        // Read from websocket into incoming queue
        match self.websocket.read() {
            Ok(msg) => {
                // We do not want to send back ping/pong messages.
                if msg.is_binary() {
                    // For simplicity, we just echo the message back.
                    let mut data_raw: Vec<u8> = msg.into_data().to_vec();
                    let data_len = data_raw.len();
                    let without_zeros: &mut [u8] = data_raw.as_mut_slice()[1..data_len-1].as_mut();
                    let data = decode_packet::<PacketFormat<PacketData>>(without_zeros);
                    match data {
                        Ok(packet) => {

                            self.stats.rx_packets += 1;
                            self.stats.rx_bytes += data_raw.len() as u32;
                            if let PacketData::SubscriptionRequest(sub_req) = &packet.data {
                                self.update_topics(&sub_req);
                            } else {
                                self.client.borrow_mut().send(packet);
                            }
                        }
                        Err(_) => {
                            self.stats.decode_error_count += 1;
                        }
                    }
                } else {
                    // Respond telling client to use binary
                    let warning_msg = tungstenite::Message::Text("Please send binary messages only.".into());
                    if let Err(_) = self.websocket.write(warning_msg) {
                        self.stats.decode_error_count += 1; // Counts as a failed message
                    }
                }
            }
            Err(tungstenite::Error::ConnectionClosed) => {
                self.is_alive = false;
            }
            Err(tungstenite::Error::AlreadyClosed) => {
                self.is_alive = false;
            }
            _ => {
                // No data to read
            }
        }

        // Write from outgoing queue to websocket
        for packet in self.client.borrow_mut().fetch_all() {
            // Encode packet
            let mut encode_buffer = [0u8; 600];
            encode_buffer[0] = 0; // COBS initial byte
            let encoded_size =
                match packet_encoding::encode_packet(&*packet, &mut encode_buffer[1..]) {
                    Ok(size) => size,
                    Err(_) => {
                        self.stats.encode_error_count += 1;
                        continue;
                    }
                };
            encode_buffer[encoded_size + 1] = 0x00; // COBS final byte
            let encode_sized = &encode_buffer[..encoded_size + 2];

            // Send over websocket
            if let Err(err) = self.websocket.send(tungstenite::Message::Binary(tungstenite::Bytes::copy_from_slice(encode_sized))) {
                match err {
                    tungstenite::Error::ConnectionClosed => {
                        self.is_alive = false;
                    }
                    tungstenite::Error::AlreadyClosed => {
                        self.is_alive = false;
                    }
                    _ => {}
                }
                self.stats.write_error_count += 1;

            } else {
                self.stats.tx_packets += 1;
                self.stats.tx_bytes += encode_sized.len() as u32;
            }
        }

        // Send stats once per second
        if self.stats_send_time.elapsed() >= std::time::Duration::from_secs(1) {
            let diag_msg: DiagnosticMsg = self.stats.to_log();
            self.client
                .borrow_mut()
                .send(PacketFormat {
                    to: None,
                    from: None,
                    data: PacketData::DiagnosticMsg(diag_msg),
                    time: get_current_time(),
                    id: 0,
                });
            self.stats_send_time = Instant::now();
        }
    }
}




pub struct WebsocketAcceptor {
    pub router: Rc<RefCell<Router<PacketFormat<PacketData>>>>,
    pub clients_by_ip: HashMap<String, WebsocketClient>,
    pub server: TcpListener,
}


impl WebsocketAcceptor {
    pub fn new(router: Rc<RefCell<Router<PacketFormat<PacketData>>>>, address: &str) -> Self {
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
            if let Ok(websocket) = accept(stream) {
                let peer_addr = addr.to_string();
                let client = WebsocketClient::new(websocket);
                self.router.borrow_mut().register_client(Rc::downgrade(&client.client));

                self.clients_by_ip.insert(
                    peer_addr.clone(),
                    client,
                );
                println!("New websocket client connected: {}", peer_addr);
            }
        }


        for (_ip, client) in self.clients_by_ip.iter_mut() {
            client.tick();
        }

        // Remove dead clients
        self.clients_by_ip.retain(|_ip, client| client.is_alive);
    }
}