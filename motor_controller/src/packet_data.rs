use serde::{Deserialize, Serialize};
use topics::packet_data_enum;
use topics::{
    ClockRequest, ClockResponse, DiagnosticMsg, OdometryDelta, PacketDataTrait, SubscriptionRequest, MotionVelocityRequest
};

packet_data_enum! {
    ClockRequest,
    ClockResponse,
    DiagnosticMsg,
    OdometryDelta,
    SubscriptionRequest,
    MotionVelocityRequest,
}
