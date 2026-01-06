use embedded_hal::pwm::SetDutyCycle;
use esp_hal::{
    ledc::{
        LowSpeed,
        channel::{Channel, ChannelHW}
    },
};

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
}
