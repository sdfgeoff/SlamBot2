use std::cell::RefCell;
use std::rc::Weak;
use std::{collections::HashMap, rc::Rc};

pub trait PacketTrait {
    fn get_to(&self) -> Option<u16>;
    fn get_topic(&self) -> &str;
    fn set_from(&mut self, from: u16);
}

pub struct Client<T: PacketTrait> {
    outgoing_queue: Vec<T>,
    incoming_queue: Vec<Rc<T>>,
    subscriptions: Vec<String>,
}

impl<T: PacketTrait> Client<T> {
    fn fetch_outgoing_queue(&mut self) -> Vec<T> {
        self.outgoing_queue.drain(..).collect()
    }
    fn write_incoming_queue(&mut self, packets: Vec<Rc<T>>) {
        self.incoming_queue.extend(packets);
    }
    fn get_subscriptions(&self) -> &Vec<String> {
        &self.subscriptions
    }
}

pub struct Router<T: PacketTrait> {
    clients_by_address: HashMap<u16, Weak<RefCell<Client<T>>>>,
    address_max: u16,
}

impl<T: PacketTrait> Default for Router<T> {
    fn default() -> Self {
        Router::<T>::new()
    }
}

impl<T: PacketTrait> Router<T> {
    pub fn new() -> Self {
        Router::<T> {
            clients_by_address: HashMap::new(),
            address_max: 0,
        }
    }
    pub fn register_client(&mut self, client: Weak<RefCell<Client<T>>>) {
        self.address_max += 1;
        self.clients_by_address.insert(self.address_max, client);
    }

    /** Distributes packets. Reads from all packets outgoing queue's and   */
    pub fn poll(&mut self) {
        // Clean dead clients
        self.clients_by_address
            .retain(|_, client_weak| client_weak.upgrade().is_some());
        let clients_by_address: HashMap<u16, Rc<RefCell<Client<T>>>> = self
            .clients_by_address
            .iter()
            .filter_map(|(address, client_weak)| client_weak.upgrade().map(|c| (*address, c)))
            .collect();

        // Build addeleration structure from subscription topic to addresses
        let mut address_by_topic: HashMap<String, Vec<u16>> = HashMap::new();
        for (address, client) in clients_by_address.iter() {
            let client = client.borrow();
            let subscriptions = client.get_subscriptions();
            for topic in subscriptions.iter() {
                address_by_topic
                    .entry(topic.clone())
                    .or_default()
                    .push(*address);
            }
        }

        // Grab all packets from all clients
        let mut all_outgoing_packets: Vec<Rc<T>> = Vec::new();
        for (address, client) in clients_by_address.iter() {
            let client_outgoing_packets = client.borrow_mut().fetch_outgoing_queue();
            for mut packet in client_outgoing_packets.into_iter() {
                packet.set_from(*address);
                all_outgoing_packets.push(Rc::new(packet));
            }
        }

        // Figure out where packets neeed to go based on 'to' address or topic subscription
        let mut packets_for_addresses: HashMap<u16, Vec<Rc<T>>> = HashMap::new();
        for packet in all_outgoing_packets.iter() {
            if let Some(to_address) = packet.get_to() {
                packets_for_addresses
                    .entry(to_address)
                    .or_default()
                    .push(packet.clone());
            } else if let Some(topic_addresses) = address_by_topic.get(packet.get_topic()) {
                for address in topic_addresses {
                    packets_for_addresses
                        .entry(*address)
                        .or_default()
                        .push(packet.clone());
                }
            }
        }

        // Send packets to clients
        for (address, packets) in packets_for_addresses.into_iter() {
            if let Some(client) = clients_by_address.get(&address) {
                client.borrow_mut().write_incoming_queue(packets);
            }
        }
    }
}
