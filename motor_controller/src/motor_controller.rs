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
}

impl<'a> MotorDriver<'a> {
    pub fn set_speed(&mut self, speed: f32) {
        let clamped_speed = speed.clamp(-1.0, 1.0);
        if speed < 0.01 && speed > -0.01 {
            self.a.set_duty_hw(0);
            self.b.set_duty_hw(0);
            return;
        }
        let amax = self.a.max_duty_cycle();
        let bmax = self.b.max_duty_cycle();
        if clamped_speed >= 0.0 {
            self.a
                .set_duty_hw(((1.0 - clamped_speed) * amax as f32) as u32);
            self.b.set_duty_hw(bmax as u32);
        } else {
            self.a.set_duty_hw(amax as u32);
            self.b
                .set_duty_hw(((1.0 + clamped_speed) * bmax as f32) as u32);
        }
    }
}

pub struct MotorControllers<'a> {
    pub left: MotorDriver<'a>,
    pub right: MotorDriver<'a>,
    pub set_velocity_time: Instant,
}


impl<'a> MotorControllers<'a> {
    pub fn handle_speed_request(&mut self, request: &MotionVelocityRequest) {
        let v = request.linear_velocity;
        let w = request.angular_velocity;

        let left_wheel_velocity = v - (w * WHEEL_BASE_WIDTH / 2.0);
        let right_wheel_velocity = v + (w * WHEEL_BASE_WIDTH / 2.0);

        let left_rpm = (left_wheel_velocity / WHEEL_CIRCUMFERENCE) * 60.0;
        let right_rpm = (right_wheel_velocity / WHEEL_CIRCUMFERENCE) * 60.0;

        let left_speed = left_rpm / NOMINAL_MAX_RPM;
        let right_speed = right_rpm / NOMINAL_MAX_RPM;

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