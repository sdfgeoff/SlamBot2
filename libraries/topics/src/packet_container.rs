use packet_trait::PacketTrait;
use serde::{Deserialize, Serialize};



pub trait PacketDataTrait {
    fn topic(&self) -> &'static str;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PacketFormat<T> {
    pub to: Option<u16>,
    pub from: Option<u16>,
    pub data: T,
    pub time: u64,
    pub id: u32,
}

impl<T: PacketDataTrait> PacketTrait for PacketFormat<T> {
    fn get_to(&self) -> Option<u16> {
        self.to
    }
    fn get_topic(&self) -> &str {
        self.data.topic()
    }
    fn set_from(&mut self, from: u16) {
        self.from = Some(from);
    }
}
