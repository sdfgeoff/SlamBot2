use esp_hal::time::Instant;
use topics::{ClockRequest, ClockResponse};
use crate::packet_data::PacketData;

pub struct Clock {
    pub offset: Option<u64>,
    pub average_round_trip_time: Option<u64>,
}

impl Clock {
    pub fn new() -> Self {
        Clock {
            offset: None,
            average_round_trip_time: None,
        }
    }

    fn get_raw_time(&self) -> u64 {
        Instant::now().duration_since_epoch().as_micros()
    }

    pub fn get_time(&self) -> u64 {
        let now = self.get_raw_time();
        if let Some(offset) = self.offset {
            now.wrapping_add(offset)
        } else {
            now
        }
    }

    pub fn generate_request_data(&self) -> PacketData {
        PacketData::ClockRequest(ClockRequest {
            request_time: self.get_raw_time(),
        })
    }

    pub fn handle_clock_response(&mut self, response: &ClockResponse) -> u64 {
        let current_time = self.get_raw_time();
        let this_round_trip_time = current_time.wrapping_sub(response.request_time);

        let rtt = if let Some(avg_rtt) = self.average_round_trip_time {
            let new_avg = (avg_rtt * 19 + this_round_trip_time) / 20;
            self.average_round_trip_time = Some(new_avg);
            new_avg
        } else {
            self.average_round_trip_time = Some(this_round_trip_time);
            this_round_trip_time
        };

        let estimated_offset = response.recieved_time.wrapping_add(rtt / 2);

        if let Some(offset) = self.offset {
            let new_offset = (offset * 7 + estimated_offset) / 8;
            self.offset = Some(new_offset);
        } else {
            self.offset = Some(estimated_offset);
        }

        rtt
    }
}
