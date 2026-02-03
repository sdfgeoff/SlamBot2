use embedded_hal::pwm::SetDutyCycle;
use esp_hal::{ledc::{
    LowSpeed,
    channel::{Channel, ChannelHW},
}, time::{Duration, Instant}};

use crate::consts::{
    WHEEL_BASE_WIDTH,
    WHEEL_CIRCUMFERENCE,
    NOMINAL_MAX_RPM,
};
use topics::MotionVelocityRequest;

pub struct MotorDriver<'a> {
    pub a: Channel<'a, LowSpeed>,
    pub b: Channel<'a, LowSpeed>,
    pub invert: bool,
}

impl<'a> MotorDriver<'a> {
    pub fn set_speed(&mut self, speed: f32) {
        let clamped_speed = speed.clamp(-1.0, 1.0);
        let output_speed = if self.invert { -clamped_speed } else { clamped_speed };
        if output_speed < 0.01 && output_speed > -0.01 {
            self.a.set_duty_hw(0);
            self.b.set_duty_hw(0);
            return;
        }
        let amax = self.a.max_duty_cycle();
        let bmax = self.b.max_duty_cycle();
        if output_speed >= 0.0 {
            self.a
                .set_duty_hw(((1.0 - output_speed) * amax as f32) as u32);
            self.b.set_duty_hw(bmax as u32);
        } else {
            self.a.set_duty_hw(amax as u32);
            self.b
                .set_duty_hw(((1.0 + output_speed) * bmax as f32) as u32);
        }
    }
}

pub struct MotorControllers<'a> {
    pub left: MotorDriver<'a>,
    pub right: MotorDriver<'a>,
    pub set_velocity_time: Instant,
}


impl<'a> MotorControllers<'a> {

    pub fn velocity_to_speed_percent(&self, velocity: f32) -> f32 {
        let wheel_rpm = (velocity / WHEEL_CIRCUMFERENCE) * 60.0;
        let speed_percent = wheel_rpm / NOMINAL_MAX_RPM;
        speed_percent
    }

    pub fn handle_speed_request(&mut self, request: &MotionVelocityRequest) {
        let w = request.angular_velocity;

        let mut vel = f32::clamp(self.velocity_to_speed_percent(request.linear_velocity), -1.0, 1.0);
        let right_add_vel = f32::clamp(self.velocity_to_speed_percent(w * WHEEL_BASE_WIDTH / 2.0), -1.0, 1.0);
        let left_add_vel = -right_add_vel;

        // We want to prioritize angular velocity if the combined speeds exceed 100% or -100%
        let max_combined = vel.abs() + right_add_vel.abs(); // We know that left and right are inverse
        if max_combined > 1.0 {
            if vel > 0.0 {
                vel -= right_add_vel.abs();
            } else {
                vel += right_add_vel.abs();
            }
        }
        
        let left_speed = vel + left_add_vel;
        let right_speed = vel + right_add_vel;

        self.left.set_speed(left_speed);
        self.right.set_speed(right_speed);

        self.set_velocity_time = Instant::now();

    }

    pub fn tick(&mut self) {
        if self.set_velocity_time.elapsed() >= Duration::from_millis(1000) {
            self.left.set_speed(0.0);
            self.right.set_speed(0.0);
        }
    }
}