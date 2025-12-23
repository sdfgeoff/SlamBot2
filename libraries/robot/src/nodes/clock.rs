use chrono::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use packet_router::Client;
use topics::{PacketFormat, PacketData};

fn get_current_time() -> u64 {
    let now: DateTime<Utc> = Utc::now();
    now.timestamp_micros() as u64
}


pub struct Clock {
    pub client: Rc<RefCell<Client<PacketFormat>>>,
}

impl Clock {
    pub fn new()  -> Clock {
        let client = Rc::new(RefCell::new(Client::<PacketFormat>::default()));

        let request_topic = PacketData::ClockRequest( topics::ClockRequest { request_time: 0 }).topic().to_string();
        client.borrow_mut().subscriptions.push(request_topic);
        Clock {
            client,
        }
    }

    pub fn tick(&mut self) {
        let incoming_packets = self.client.borrow_mut().fetch_all();
        for packet in incoming_packets {
            match &packet.data {
                PacketData::ClockRequest(req) => {
                    let response = PacketFormat {
                        to: packet.from,
                        from: None,
                        data: PacketData::ClockResponse(topics::ClockResponse {
                            request_time: req.request_time,
                            recieved_time: get_current_time(),
                        }),
                        time: get_current_time(),
                        id: packet.id,
                    };
                    self.client
                        .borrow_mut()
                        .send(response);
                }
                _ => {}
            }
        }
    }
}