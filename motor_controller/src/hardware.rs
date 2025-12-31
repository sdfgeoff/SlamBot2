use core::cell::RefCell;
use esp_hal::gpio::{Input, Output, Event};
use critical_section::Mutex;
use esp_hal::handler;


pub struct MotorUnit<'a> {
    pub encoder_a: Input<'a>,
    pub encoder_b: Input<'a>,
    pub encoder_count: i64,
}

pub struct Hardware<'a> {
    pub motor_left: MotorUnit<'a>,
    pub motor_right: MotorUnit<'a>,

}

pub static HARDWARE: Mutex<RefCell<Option<Hardware>>> = Mutex::new(RefCell::new(None));

impl<'a> Hardware<'a> {
    pub fn configure(&mut self) {
        self.motor_left.encoder_a.listen(Event::AnyEdge);
        self.motor_left.encoder_b.listen(Event::AnyEdge);

        self.motor_right.encoder_a.listen(Event::AnyEdge);
        self.motor_right.encoder_b.listen(Event::AnyEdge);
    }
}


#[inline]
fn val_to_dir(a: bool, b: bool) -> i64 {
    if a != b {
        1
    } else {
        -1
    }
}

#[handler]
pub fn encoder_interrupt_handler() {
    critical_section::with(|cs| {
        if let Some(hardware) = HARDWARE.borrow(cs).borrow_mut().as_mut() {
            // Left motor encoder handling
            if hardware.motor_left.encoder_a.is_interrupt_set() {
                hardware.motor_left.encoder_a.clear_interrupt();
                hardware.motor_left.encoder_count += val_to_dir(
                    hardware.motor_left.encoder_a.is_high(),
                    hardware.motor_left.encoder_b.is_high(),
                );
            }
            if hardware.motor_left.encoder_b.is_interrupt_set() {
                hardware.motor_left.encoder_b.clear_interrupt();
                hardware.motor_left.encoder_count -= val_to_dir(
                    hardware.motor_left.encoder_a.is_high(),
                    hardware.motor_left.encoder_b.is_high(),
                );
            }

            // Right motor encoder handling
            if hardware.motor_right.encoder_a.is_interrupt_set() {
                hardware.motor_right.encoder_a.clear_interrupt();
                hardware.motor_right.encoder_count += val_to_dir(
                    hardware.motor_right.encoder_a.is_high(),
                    hardware.motor_right.encoder_b.is_high(),
                );
            }
            if hardware.motor_right.encoder_b.is_interrupt_set() {
                hardware.motor_right.encoder_b.clear_interrupt();
                hardware.motor_right.encoder_count -= val_to_dir(
                    hardware.motor_right.encoder_a.is_high(),
                    hardware.motor_right.encoder_b.is_high(),
                );
            }
        }
    });

}