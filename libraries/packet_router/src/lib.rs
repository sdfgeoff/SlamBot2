use packet_trait::PacketTrait;
use std::cell::RefCell;
use std::rc::Weak;
use std::{collections::HashMap, rc::Rc};

mod client;

pub use client::Client;

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
            let client_outgoing_packets = client.borrow_mut().fetch_client_to_router();
            for mut packet in client_outgoing_packets.into_iter() {
                packet.set_from(*address);
                all_outgoing_packets.push(Rc::new(packet));
            }
        }

        let subscribers_to_all_topic: Vec<u16> = address_by_topic
            .get("all")
            .cloned()
            .unwrap_or_default();

        // Figure out where packets neeed to go based on 'to' address or topic subscription
        let mut packets_for_addresses: HashMap<u16, Vec<Rc<T>>> = HashMap::new();
        for packet in all_outgoing_packets.iter() {
            if let Some(to_address) = packet.get_to() {
                packets_for_addresses
                    .entry(to_address)
                    .or_default()
                    .push(packet.clone());
            } else {
                let topic = packet.get_topic();
                subscribers_to_all_topic.iter().for_each(|address| {
                    packets_for_addresses
                        .entry(*address)
                        .or_default()
                        .push(packet.clone());
                });
                if let Some(topic_addresses) = address_by_topic.get(topic) {
                    for address in topic_addresses {
                        packets_for_addresses
                            .entry(*address)
                            .or_default()
                            .push(packet.clone());
                    }
                }
            }
        }

        // Send packets to clients
        for (address, packets) in packets_for_addresses.into_iter() {
            if let Some(client) = clients_by_address.get(&address) {
                client.borrow_mut().write_router_to_client(packets);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Clone, Debug, PartialEq)]
    struct TestPacket {
        to: Option<u16>,
        from: Option<u16>,
        topic: String,
        data: String,
    }

    impl TestPacket {
        fn new(topic: String, data: String) -> Self {
            TestPacket {
                to: None,
                from: None,
                topic,
                data,
            }
        }

        fn with_to(mut self, to: u16) -> Self {
            self.to = Some(to);
            self
        }
    }

    impl PacketTrait for TestPacket {
        fn get_to(&self) -> Option<u16> {
            self.to
        }

        fn get_topic(&self) -> &str {
            &self.topic
        }

        fn set_from(&mut self, from: u16) {
            self.from = Some(from);
        }
    }

    impl<T: PacketTrait> Client<T> {
        fn new() -> Self {
            Client {
                client_to_router: Vec::new(),
                router_to_client: Vec::new(),
                subscriptions: Vec::new(),
            }
        }

        fn subscribe(&mut self, topic: String) {
            self.subscriptions.push(topic);
        }

        fn send_packet(&mut self, packet: T) {
            self.client_to_router.push(packet);
        }

        fn receive_packet(&mut self) -> Option<Rc<T>> {
            if self.router_to_client.is_empty() {
                None
            } else {
                Some(self.router_to_client.remove(0))
            }
        }

        fn has_packets(&self) -> bool {
            !self.router_to_client.is_empty()
        }

        fn packet_count(&self) -> usize {
            self.router_to_client.len()
        }
    }

    #[test]
    fn test_router_creation() {
        let router: Router<TestPacket> = Router::new();
        assert_eq!(router.address_max, 0);
        assert!(router.clients_by_address.is_empty());
    }

    #[test]
    fn test_client_registration() {
        let mut router: Router<TestPacket> = Router::new();
        let client = Rc::new(RefCell::new(Client::new()));

        router.register_client(Rc::downgrade(&client));

        assert_eq!(router.address_max, 1);
        assert_eq!(router.clients_by_address.len(), 1);
    }

    #[test]
    fn test_direct_address_routing() {
        let mut router: Router<TestPacket> = Router::new();

        // Create two clients
        let client1 = Rc::new(RefCell::new(Client::new()));
        let client2 = Rc::new(RefCell::new(Client::new()));

        router.register_client(Rc::downgrade(&client1));
        router.register_client(Rc::downgrade(&client2));

        // Client 1 sends a packet directly to client 2 (address 2)
        let packet = TestPacket::new("test".to_string(), "hello".to_string()).with_to(2);
        client1.borrow_mut().send_packet(packet);

        router.poll();

        // Client 2 should receive the packet
        assert!(client2.borrow().has_packets());
        assert!(!client1.borrow().has_packets());

        let received = client2.borrow_mut().receive_packet().unwrap();
        assert_eq!(received.topic, "test");
        assert_eq!(received.data, "hello");
        assert_eq!(received.from, Some(1)); // Should be set to sender's address
    }

    #[test]
    fn test_topic_subscription_routing() {
        let mut router: Router<TestPacket> = Router::new();

        // Create three clients
        let client1 = Rc::new(RefCell::new(Client::new()));
        let client2 = Rc::new(RefCell::new(Client::new()));
        let client3 = Rc::new(RefCell::new(Client::new()));

        router.register_client(Rc::downgrade(&client1));
        router.register_client(Rc::downgrade(&client2));
        router.register_client(Rc::downgrade(&client3));

        // Client 2 and 3 subscribe to "sensor_data"
        client2.borrow_mut().subscribe("sensor_data".to_string());
        client3.borrow_mut().subscribe("sensor_data".to_string());

        // Client 1 sends a packet to "sensor_data" topic
        let packet = TestPacket::new("sensor_data".to_string(), "temperature: 25C".to_string());
        client1.borrow_mut().send_packet(packet);

        router.poll();

        // Only clients 2 and 3 should receive the packet
        assert!(!client1.borrow().has_packets());
        assert!(client2.borrow().has_packets());
        assert!(client3.borrow().has_packets());

        let received2 = client2.borrow_mut().receive_packet().unwrap();
        let received3 = client3.borrow_mut().receive_packet().unwrap();

        assert_eq!(received2.topic, "sensor_data");
        assert_eq!(received2.from, Some(1));
        assert_eq!(received3.topic, "sensor_data");
        assert_eq!(received3.from, Some(1));
    }

    #[test]
    fn test_all_topic_broadcast() {
        let mut router: Router<TestPacket> = Router::new();

        // Create three clients
        let client1 = Rc::new(RefCell::new(Client::new()));
        let client2 = Rc::new(RefCell::new(Client::new()));
        let client3 = Rc::new(RefCell::new(Client::new()));

        router.register_client(Rc::downgrade(&client1));
        router.register_client(Rc::downgrade(&client2));
        router.register_client(Rc::downgrade(&client3));

        // Client 1 sends a packet to "all" topic
        let packet = TestPacket::new("all".to_string(), "broadcast message".to_string());
        client1.borrow_mut().send_packet(packet);

        router.poll();

        // All clients should receive the packet
        assert!(client1.borrow().has_packets());
        assert!(client2.borrow().has_packets());
        assert!(client3.borrow().has_packets());

        // Check that all received the same message
        for client in [&client1, &client2, &client3] {
            let received = client.borrow_mut().receive_packet().unwrap();
            assert_eq!(received.topic, "all");
            assert_eq!(received.data, "broadcast message");
            assert_eq!(received.from, Some(1));
        }
    }

    #[test]
    fn test_multiple_subscriptions() {
        let mut router: Router<TestPacket> = Router::new();

        let client1 = Rc::new(RefCell::new(Client::new()));
        let client2 = Rc::new(RefCell::new(Client::new()));

        router.register_client(Rc::downgrade(&client1));
        router.register_client(Rc::downgrade(&client2));

        // Client 2 subscribes to multiple topics
        client2.borrow_mut().subscribe("topic1".to_string());
        client2.borrow_mut().subscribe("topic2".to_string());

        // Send packets to both topics
        client1
            .borrow_mut()
            .send_packet(TestPacket::new("topic1".to_string(), "msg1".to_string()));
        client1
            .borrow_mut()
            .send_packet(TestPacket::new("topic2".to_string(), "msg2".to_string()));

        router.poll();

        // Client 2 should receive both packets
        assert_eq!(client2.borrow().packet_count(), 2);
        assert!(!client1.borrow().has_packets());
    }

    #[test]
    fn test_no_subscribers() {
        let mut router: Router<TestPacket> = Router::new();

        let client1 = Rc::new(RefCell::new(Client::new()));
        let client2 = Rc::new(RefCell::new(Client::new()));

        router.register_client(Rc::downgrade(&client1));
        router.register_client(Rc::downgrade(&client2));

        // Send packet to topic with no subscribers
        let packet = TestPacket::new("nonexistent_topic".to_string(), "lost message".to_string());
        client1.borrow_mut().send_packet(packet);

        router.poll();

        // No client should receive the packet
        assert!(!client1.borrow().has_packets());
        assert!(!client2.borrow().has_packets());
    }

    #[test]
    fn test_dead_client_cleanup() {
        let mut router: Router<TestPacket> = Router::new();

        let client1 = Rc::new(RefCell::new(Client::new()));
        {
            let client2 = Rc::new(RefCell::new(Client::new()));
            router.register_client(Rc::downgrade(&client1));
            router.register_client(Rc::downgrade(&client2));
            assert_eq!(router.clients_by_address.len(), 2);
        } // client2 goes out of scope

        // After polling, dead client should be cleaned up
        router.poll();
        assert_eq!(router.clients_by_address.len(), 1);
    }

    #[test]
    fn test_invalid_address_routing() {
        let mut router: Router<TestPacket> = Router::new();

        let client1 = Rc::new(RefCell::new(Client::new()));
        router.register_client(Rc::downgrade(&client1));

        // Send packet to non-existent address
        let packet = TestPacket::new("test".to_string(), "lost".to_string()).with_to(999);
        client1.borrow_mut().send_packet(packet);

        router.poll();

        // No client should receive the packet
        assert!(!client1.borrow().has_packets());
    }

    #[test]
    fn test_multiple_packets_same_poll() {
        let mut router: Router<TestPacket> = Router::new();

        let client1 = Rc::new(RefCell::new(Client::new()));
        let client2 = Rc::new(RefCell::new(Client::new()));

        router.register_client(Rc::downgrade(&client1));
        router.register_client(Rc::downgrade(&client2));

        client2.borrow_mut().subscribe("test".to_string());

        // Send multiple packets in one poll cycle
        client1
            .borrow_mut()
            .send_packet(TestPacket::new("test".to_string(), "msg1".to_string()));
        client1
            .borrow_mut()
            .send_packet(TestPacket::new("test".to_string(), "msg2".to_string()));
        client1
            .borrow_mut()
            .send_packet(TestPacket::new("test".to_string(), "msg3".to_string()));

        router.poll();

        // Client 2 should receive all packets
        assert_eq!(client2.borrow().packet_count(), 3);
    }
}
