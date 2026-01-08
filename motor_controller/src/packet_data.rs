use serde::{Deserialize, Serialize};
use topics::packet_data_enum;
use topics::{
    DiagnosticMsg,
    ClockRequest,
    ClockResponse,
    PacketDataTrait,
    OdometryDelta,
    SubscriptionRequest,
};


packet_data_enum! {
    ClockRequest,
    ClockResponse,
    DiagnosticMsg,
    OdometryDelta,
    SubscriptionRequest,
}
