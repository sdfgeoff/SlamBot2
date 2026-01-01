use esp_hal::ledc::{LowSpeed, channel::Channel};
// use esp_hal::time::{Duration, Instant};


// pub struct EncoderState {
//     pub time: Instant,
//     pub count: i64,
// }



// pub struct MotorCommand {
//     pub target_rad_per_second: i16,
//     pub issued_time: Instant,
// }

// pub struct MotorController {
//     pub command: Option<MotorCommand>,
//     pub encoder_ticks_per_revolution: f32,
//     pub prev_encoder_state: Option<EncoderState>,
//     pub channel_a: ledc.channel
// }


// impl MotorController {
//     pub fn new(encoder_ticks_per_revolution: f32) -> Self {
//         MotorController {
//             command: None,
//             encoder_ticks_per_revolution,
//             prev_encoder_state: None,
//         }
//     }


//     pub fn update(&mut self, time: Instant, encoder_count: i64) {
//         if let Some(prev_state) = &self.prev_encoder_state {
//             let delta_time_us = (time - prev_state.time).as_micros();
//             let delta_counts = (encoder_count - prev_state.count) as f32;
//             let revolutions = delta_counts / self.encoder_ticks_per_revolution;
//             let rps = revolutions / (delta_time_us as f32 / 1_000_000.0);

//             self.prev_encoder_state = Some(EncoderState {
//                 time,
//                 count: encoder_count,
//             });
//             if let Some(command) = &self.command {
//                 if time - command.issued_time > Duration::from_millis(1000) {
//                     // Command expired
//                     self.command = None;
//                     self.write_to_motor(0.0);
//                     return;
//                 }
//                 let error = command.target_rad_per_second as f32 / (2.0 * core::f32::consts::PI) - rps;
//                 self.write_to_motor(-error);
//                 // Implement control algorithm here (e.g., PID controller)
//             } else {
//                 self.write_to_motor(0.0);
//             }
//         }
//     }

//     pub fn write_to_motor(&mut self, pwm_value: f32) {
//         // Write a PWM value to the motor driver from -1 to 1
//         unimplemented!()
//     }
// }


pub struct MotorDriver<'a> {
    a: Channel<'a, LowSpeed>,
    b: Channel<'a, LowSpeed>,
}

impl<'a> MotorDriver<'a> {
    pub fn set_speed(&mut self, speed: f32) {
        let clamped_speed = if speed > 1.0 {
            1.0
        } else if speed < -1.0 {
            -1.0
        } else {
            speed
        };

        if clamped_speed >= 0.0 {
            self.a.set_duty_hw((clamped_speed * self.a.get_max_duty() as f32) as u32);
            self.b.set_duty_hw(0);
        } else {
            self.a.set_duty_hw(0);
            self.b.set_duty_hw((-clamped_speed * self.b.get_max_duty() as f32) as u32);
        }
    }
}