use heapless::Vec;
use packet_router::Client;
use std::cell::RefCell;
use std::rc::Rc;
use topics::{DiagnosticStatus, PacketFormat, PacketData, PacketDataTrait};

pub struct Log {
    pub client: Rc<RefCell<Client<PacketFormat<PacketData>>>>,
}

impl Log {
    pub fn new(log_all: bool) -> Log {
        let client = Rc::new(RefCell::new(Client::<PacketFormat<PacketData>>::default()));
        if log_all {
            client.borrow_mut().subscriptions.insert("all".to_string());
        } else {
            client.borrow_mut().subscriptions.insert(
                topics::PacketData::DiagnosticMsg(topics::DiagnosticMsg {
                    level: DiagnosticStatus::Ok,
                    name: "".try_into().expect("Arge"),
                    message: "".try_into().expect("Arge"),
                    values: Vec::new(),
                })
                .topic()
                .to_string(),
            );
        };
        Log { client }
    }

    pub fn step(&mut self) {
        let log_packets = self.client.borrow_mut().fetch_all();
        for packet in log_packets {
            println!(
                "{}| {}",
                chrono::DateTime::from_timestamp_micros(packet.time as i64)
                    .expect("datetime overflow")
                    .to_rfc3339(),
                serde_json::to_string(&(*packet)).unwrap()
            );
        }
    }
}
