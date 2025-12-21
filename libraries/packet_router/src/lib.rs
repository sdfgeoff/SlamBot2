use std::{collections::HashMap, rc::Rc};
use serde::{Serialize, Deserialize};
use packet_encoding::decode_packet;
use std::rc::Weak;
use std::cell::RefCell;

#[derive(Serialize, Deserialize, Debug)]
pub struct SubscriptionPacket {
    pub topics: Vec<String>,
}


#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug)]
enum PacketTypes {
    Subscription(SubscriptionPacket),
    TestMessage(String),
}



#[derive(Serialize, Deserialize, Debug)]
pub struct Packet<T> {

    #[serde(default)]
    to: Option<u16>,
    
    #[serde(default)]
    from: Option<u16>,
    data: T,
    time: u64,
    id: u64,

    topic: String,
}

struct Client {
    outgoing_queue: Vec<Rc<Vec<u8>>>,
    incoming_queue: Vec<Rc<Vec<u8>>>,
}

impl Client {
    fn fetch_outgoing_queue(&mut self) -> Vec<Rc<Vec<u8>>> {
        self.outgoing_queue.drain(..).collect()
    }
    fn write_incoming_queue(&mut self, packets: Vec<Rc<Vec<u8>>>){
        self.incoming_queue.extend(packets);
    }
}


struct Router {
    clients_by_address: HashMap<u16, Weak<RefCell<Client>>>,
    address_by_topic: HashMap<String, Vec<u16>>,
    address_max: u16,
}


impl Router {
    pub fn new() -> Self {
        Router {
            clients_by_address: HashMap::new(),
            address_by_topic: HashMap::new(),
            address_max: 0,
        }
    }
    pub fn register_client(&mut self, client: Weak<RefCell<Client>>) {
        self.address_max += 1;
        self.clients_by_address.insert(self.address_max, client);
    }


    fn handle_subscription_packet(&mut self, address: u16, subscription_packet: &SubscriptionPacket) {
        for (topic, addresses) in self.address_by_topic.iter_mut() {
            if subscription_packet.topics.contains(topic) {
                addresses.push(address);
            } else {
                let index = addresses.iter().position(|&x| x == address);
                if let Some(i) = index {
                    addresses.remove(i);
                }
            }
        }
    }

    /** Distributes packets. Reads from all packets outgoing queue's and   */
    pub fn poll(&mut self) {
        // Clean dead clients
        self.clients_by_address.retain(|_, client_weak| client_weak.upgrade().is_some());

        // Grab all packets from all clients
        let mut all_outgoing_packets: Vec<Rc<Packet<PacketTypes>>> = Vec::new();
        let mut subscription_packets: HashMap<u16, SubscriptionPacket> = HashMap::new();
        for (address, client) in self.clients_by_address.iter_mut() {
            let client = match client.upgrade() {
                Some(c) => c,
                None => continue, // Skip dead clients
            };
            let client_outgoing_packets = client.borrow_mut().fetch_outgoing_queue();
            for packet in client_outgoing_packets.iter() {
                let mut decoded_packet: Packet<PacketTypes> = match decode_packet(&mut packet.as_ref().clone()) {
                    Ok(pkt) => pkt,
                    Err(_) => continue, // Skip invalid packets
                };
                decoded_packet.from = Some(*address);

                match decoded_packet.data {
                    PacketTypes::Subscription(sub_packet) => {
                        subscription_packets.insert(*address, sub_packet);
                    }
                    _ => {
                        all_outgoing_packets.push(Rc::new(decoded_packet));
                    }
                }
            }
        }

        // Update subscriptions
        for (address, sub_packet) in subscription_packets.iter() {
            self.handle_subscription_packet(*address, sub_packet);
        }

        // Figure out where packets neeed to go based on 'to' address or topic subscription
        let mut packets_for_addresses: HashMap<u16, Vec<Rc<Packet<PacketTypes>>>> = HashMap::new();
        for packet in all_outgoing_packets.iter() {
            if let Some(to_address) = packet.to {
                packets_for_addresses.entry(to_address).or_insert(Vec::new()).push(packet.clone());
            } else if let Some(topic_addresses) = self.address_by_topic.get(&packet.topic) {
                for address in topic_addresses {
                    packets_for_addresses.entry(*address).or_insert(Vec::new()).push(packet.clone());
                }
            }
        }

        // Send packets to clients
        for (address, packets) in packets_for_addresses.iter() {
            if let Some(client) = self.clients_by_address.get(address) {
                let client = match client.upgrade() {
                    Some(c) => c,
                    None => continue, // Skip dead clients
                };
                client.borrow_mut().write_incoming_queue(packets.iter().map(|p| {
                    let mut encode_buffer = [0u8; 600];
                    encode_buffer[0] = 0; // COBS initial byte
                    let encoded_size = packet_encoding::encode_packet(&**p, &mut encode_buffer[1..]).unwrap();
                    encode_buffer[encoded_size + 1] = 0x00; // COBS final byte
                    Rc::new(encode_buffer[..encoded_size + 2].to_vec())
                }).collect());
            }
        }
    }

}



