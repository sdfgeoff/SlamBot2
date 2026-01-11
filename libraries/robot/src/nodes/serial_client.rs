use packet_encoding::{PacketFinder, decode_packet, encode_packet};
use packet_router::Client;
use serde::Serialize;
use serial::SerialPort;
use std::rc::Rc;
use std::time::{Duration, Instant};
use std::{cell::RefCell, collections::HashSet};

use topics::{DiagnosticMsg, PacketData, PacketFormat, SubscriptionRequest};

use heapless::{String as HString, format as hformat};
use std::str::FromStr;

use crate::nodes::clock::get_current_time;

#[derive(Serialize)]
pub struct SerialClientStats {
    pub decode_error_count: u32,
    pub tx_packets: u32,
    pub tx_bytes: u32,
    pub rx_packets: u32,
    pub rx_bytes: u32,
    pub encode_error_count: u32,
    pub write_error_count: u32,
}

impl SerialClientStats {
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
            name: HString::from_str("serial_stats").unwrap(),
            message: HString::from_str("").unwrap(),
            values,
        }
    }
}


pub struct SerialClient<V: SerialPort> {
    serialport: V,
    pub client: Rc<RefCell<Client<PacketFormat<PacketData>>>>,
    packet_finder: PacketFinder,

    pub stats: SerialClientStats,
    pub stats_send_time: Instant,
}

impl<V: SerialPort> SerialClient<V> {
    pub fn new(serialport: V) -> Self {
        SerialClient {
            serialport,
            client: Rc::new(RefCell::new(Client::default())),
            packet_finder: PacketFinder::new(),
            stats: SerialClientStats {
                decode_error_count: 0,
                tx_packets: 0,
                tx_bytes: 0,
                rx_packets: 0,
                rx_bytes: 0,
                encode_error_count: 0,
                write_error_count: 0,
            },
            stats_send_time: Instant::now(),
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

    pub fn read(&mut self) {
        // Read from serial port into incoming queue
        let mut mini_buffer: [u8; 256] = [0u8; 256];
        let read_bytes = self.serialport.read(&mut mini_buffer).unwrap_or(0);
        for byte in mini_buffer.iter().take(read_bytes) {
            if let Some(packet) = self.packet_finder.push_byte(*byte)
                && !packet.is_empty()
            {
                self.stats.rx_packets += 1;
                self.stats.rx_bytes += packet.len() as u32;

                let mut packet_data = packet.to_vec();
                match decode_packet::<PacketFormat<PacketData>>(&mut packet_data) {
                    Ok(packet) => {
                        if let PacketData::SubscriptionRequest(sub_req) = &packet.data {
                            self.update_topics(&sub_req);
                        } else {
                            self.client.borrow_mut().client_to_router.push(packet);
                        }
                    }
                    Err(e) => {
                        self.stats.decode_error_count += 1;
                        println!("Failed to decode packet: {:?} {:X?}", e, packet_data);
                    }
                }
            }
        }
    }

    pub fn write(&mut self) {
        let packets = self.client.borrow_mut().fetch_all();
        for packet in packets {
            self.stats.tx_packets += 1;

            let mut encode_buffer: [u8; 512] = [0; 512];
            encode_buffer[0] = 0; // COBS initial byte
            match encode_packet(&(*packet), &mut encode_buffer[1..]) {
                Ok(encoded_size) => {
                    encode_buffer[encoded_size + 1] = 0x00; // COBS final byte
                    if let Err(e) = self
                        .serialport
                        .write_all(&encode_buffer[..encoded_size + 2])
                    {
                        self.stats.write_error_count += 1;
                        println!("Failed to write packet: {:?}", e);
                    }
                    self.stats.tx_bytes += (encoded_size + 2) as u32;
                }
                Err(e) => {
                    self.stats.encode_error_count += 1;
                    println!("Failed to encode packet: {:?}", e);
                }
            }
        }
    }

    pub fn tick(&mut self) {
        self.read();
        self.write();
        if self.stats_send_time.elapsed() >= Duration::from_secs(1) {
            let diag_msg: DiagnosticMsg = self.stats.to_log();
            self.client
                .borrow_mut()
                .client_to_router
                .push(PacketFormat {
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
