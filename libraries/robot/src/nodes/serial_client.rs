use packet_encoding::{PacketFinder, decode_packet, encode_packet};
use packet_router::Client;
use packet_trait::PacketTrait;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serial::SerialPort;
use std::cell::RefCell;
use std::rc::Rc;


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

pub struct SerialClient<T: PacketTrait, V: SerialPort> {
    serialport: V,
    pub client: Rc<RefCell<Client<T>>>,
    packet_finder: PacketFinder,
    pub stats: SerialClientStats,
}

impl<T: PacketTrait + DeserializeOwned + Serialize, V: SerialPort> SerialClient<T, V> {
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
                match decode_packet::<T>(&mut packet_data) {
                    Ok(packet) => {
                        self.client.borrow_mut().client_to_router.push(packet);
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
}
