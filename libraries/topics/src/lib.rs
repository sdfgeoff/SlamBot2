#![no_std]
use serde::{Deserialize, Serialize};
use heapless::String;
use packet_trait::PacketTrait;

#[derive(Serialize, Deserialize, Debug)]
pub struct ClockRequest {
    request_time: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClockResponse {
    request_time: u64,
    recieved_time: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PacketData{
    ClockRequest(ClockRequest),
    ClockResponse(ClockResponse),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PacketFormat{
    to: Option<u16>,
    from: Option<u16>,
    topic: String<32>,
    data: PacketData,
    time: u64,
    id: u32,
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
