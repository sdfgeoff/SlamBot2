use packet_trait::PacketTrait;
use std::collections::HashSet;
use std::rc::Rc;

pub struct Client<T: PacketTrait> {
    pub client_to_router: Vec<T>,
    pub router_to_client: Vec<Rc<T>>,
    pub subscriptions: HashSet<String>,
}

impl<T: PacketTrait> Default for Client<T> {
    fn default() -> Self {
        Client::<T> {
            client_to_router: Vec::new(),
            router_to_client: Vec::new(),
            subscriptions: HashSet::new(),
        }
    }
}

impl<T: PacketTrait> Client<T> {

    pub fn send(&mut self, packet: T) {
        self.client_to_router.push(packet);
    }
    pub fn fetch_all(&mut self) -> Vec<Rc<T>> {
        self.router_to_client.drain(..).collect()
    }

    pub fn fetch_client_to_router(&mut self) -> Vec<T> {
        self.client_to_router.drain(..).collect()
    }
    pub fn write_router_to_client(&mut self, packets: Vec<Rc<T>>) {
        self.router_to_client.extend(packets);
    }
    pub fn get_subscriptions(&self) -> &HashSet<String> {
        &self.subscriptions
    }
}
