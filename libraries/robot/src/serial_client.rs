use std::rc::Rc;
use packet_encoding::{decode_packet, encode_packet, PacketFinder};
use packet_router::Client;
use serde::{Serialize};
use serde::de::DeserializeOwned;
use serial::SerialPort;
use std::cell::RefCell;
use packet_trait::PacketTrait;

pub struct SerialClient<T: PacketTrait, V: SerialPort> {
    serialport: V,
    pub client: Rc<RefCell<Client<T>>>,
    packet_finder: PacketFinder,
}


impl<T: PacketTrait + DeserializeOwned + Serialize, V: SerialPort> SerialClient<T, V> {
    pub fn new(serialport: V) -> Self {
        SerialClient {
            serialport,
            client: Rc::new(RefCell::new(Client::default())),
            packet_finder: PacketFinder::new(),
        }
    }

    pub fn read(&mut self) {
        // Read from serial port into incoming queue
        let mut mini_buffer: [u8; 256] = [0u8; 256];
        let read_bytes = self.serialport.read(&mut mini_buffer).unwrap_or(0);
        for byte in mini_buffer.iter().take(read_bytes) {
            if let Some(packet) = self.packet_finder.push_byte(*byte) && !packet.is_empty(){
                    let mut packet_data = packet.to_vec();
                    match decode_packet::<T>(&mut packet_data) {
                        Ok(packet) => {
                            self.client.borrow_mut().router_to_client.push(Rc::new(packet));
                        },
                        Err(e) => {
                            println!("Failed to decode packet: {:?} {:X?}", e, packet_data);
                        }
                    
                }
            }
        }
    }

    pub fn write(&mut self) {
        let packets = self.client.borrow_mut().fetch_client_to_router();
        for packet in packets {
            let mut encode_buffer: [u8; 512] = [0; 512];
            encode_buffer[0] = 0; // COBS initial byte
            match encode_packet(&packet, &mut encode_buffer[1..]) {
                Ok(encoded_size) => {
                    encode_buffer[encoded_size + 1] = 0x00; // COBS final byte
                    self.serialport.write_all(&encode_buffer[..encoded_size + 2]).unwrap();
                },
                Err(e) => {
                    println!("Failed to encode packet: {:?}", e);
                }
            }
        }

    }
}