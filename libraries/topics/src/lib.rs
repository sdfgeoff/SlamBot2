#![no_std]
use packet_trait::PacketTrait;
use serde::{Deserialize, Serialize};

pub mod ros;
pub use ros::*;

mod packet_container;
pub use packet_container::{PacketFormat, PacketDataTrait};

mod packet_data;



#[derive(Serialize, Deserialize, Debug)]
pub struct ClockRequest {
    pub request_time: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClockResponse {
    pub request_time: u64,
    pub recieved_time: u64,
}


packet_data_enum!(
    ClockRequest,
    ClockResponse,
    DiagnosticMsg,
);


