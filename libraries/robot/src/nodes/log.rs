use packet_router::Client;
use std::cell::RefCell;
use std::rc::Rc;
use topics::{LogLevel, PacketFormat};

pub struct Log {
    pub client: Rc<RefCell<Client<PacketFormat>>>,
}

impl Log {
    pub fn new(log_all: bool) -> Log {
        let client = Rc::new(RefCell::new(Client::<PacketFormat>::default()));
        if log_all {
            client.borrow_mut().subscriptions.push("all".to_string());
        } else {
            client.borrow_mut().subscriptions.push(
                topics::PacketData::LogMessage(topics::LogMessage {
                    level: LogLevel::Info,
                    event: "".try_into().expect("Arge"),
                    json: None,
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
