use std::time::Instant;
use std::{cell::RefCell, time::Duration};
use std::rc::Rc;
use packet_router::Client;
use topics::{PacketData, PacketFormat, PacketDataTrait};

use crate::nodes::clock::get_current_time;


pub struct PositionEstimator {
    pub client: Rc<RefCell<Client<PacketFormat<PacketData>>>>,
    position: [f32; 2],
    orientation: f32,

    last_estimate_send_time: Instant,
}


impl PositionEstimator {
    pub fn new() -> Self {
        let client = Rc::new(RefCell::new(Client::default()));
        
        let odom_topic = PacketData::OdometryDelta(topics::OdometryDelta { start_time: 0, end_time: 0,delta_position: [0.0, 0.0], delta_orientation: 0.0 })
            .topic()
            .to_string();
        client.borrow_mut().subscriptions.insert(odom_topic);

        PositionEstimator {
            client,
            position: [0.0, 0.0],
            orientation: 0.0,
            last_estimate_send_time: Instant::now(),
        }
    }

    pub fn tick(&mut self) {
        // Update position estimation logic here
        let packets = self.client.borrow_mut().fetch_all();
        for packet in packets {
            if let PacketData::OdometryDelta(odom) = &packet.data {
                // Simple dead-reckoning update
                let dx = odom.delta_position[0];
                let dy = odom.delta_position[1];
                let dtheta = odom.delta_orientation;

                // Update orientation
                self.orientation += dtheta;

                // Update position
                self.position[0] += dx * self.orientation.cos() - dy * self.orientation.sin();
                self.position[1] += dx * self.orientation.sin() + dy * self.orientation.cos();
            }
        }

        // Send position estimate at regular intervals
        if self.last_estimate_send_time.elapsed() >= Duration::from_millis(100) {
            let time = get_current_time();
            let estimate = topics::PositionEstimate {
                timestamp: time.clone(),
                position: self.position,
                orientation: self.orientation,
            };
            let packet = PacketData::PositionEstimate(estimate);
            self.client.borrow_mut().send(PacketFormat {
                to: None,
                from: None,
                data: packet,
                time: time,
                id: 0,
            });
            self.last_estimate_send_time = Instant::now();
        }


    }
}



