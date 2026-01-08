#![no_std]
use serde::{Deserialize, Serialize};

pub mod ros;
pub use ros::*;

mod packet_container;
pub use packet_container::{PacketDataTrait, PacketFormat};

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

#[derive(Serialize, Deserialize, Debug)]
pub struct SubscriptionRequest {
    pub topics: heapless::Vec<heapless::String<32>, 8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OdometryDelta {
    pub start_time: u64,
    pub end_time: u64,
    pub delta_position: [f32; 2],
    pub delta_orientation: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MotionVelocityRequest {
    pub linear_velocity: f32,
    pub angular_velocity: f32,
}

packet_data_enum!(
    ClockRequest,
    ClockResponse,
    DiagnosticMsg,
    OdometryDelta,
    SubscriptionRequest,
    MotionVelocityRequest
);
