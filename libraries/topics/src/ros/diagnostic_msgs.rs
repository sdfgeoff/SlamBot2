use heapless::{String, Vec};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum DiagnosticStatus {
    Ok = 0,
    Warn = 1,
    Error = 2,
    Stale = 3,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DiagnosticKeyValue {
    pub key: String<16>,
    pub value: String<16>, // about enough to store a double
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DiagnosticMsg {
    pub level: DiagnosticStatus,
    pub name: String<16>,
    pub message: String<32>,
    pub values: Vec<DiagnosticKeyValue, 8>,
}
