#![no_std]
use heapless::String;
use packet_trait::PacketTrait;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ClockRequest {
    pub request_time: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClockResponse {
    pub request_time: u64,
    pub recieved_time: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PacketData {
    ClockRequest(ClockRequest),
    ClockResponse(ClockResponse),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PacketFormat {
    pub to: Option<u16>,
    pub from: Option<u16>,
    pub topic: String<32>,
    pub data: PacketData,
    pub time: u64,
    pub id: u32,
}

impl PacketTrait for PacketFormat {
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
