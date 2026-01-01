use core::cell::RefCell;
use critical_section::Mutex;
use esp_hal::gpio::{Event, Input};
use esp_hal::handler;

/**
 * Represents a single quadrature encoder connected to two GPIO pins.
 */
pub struct Encoder<'a> {
    pub a_input: Input<'a>,
    pub b_input: Input<'a>,

    /**
     * How many pulses have been coutned by this encoder. This is the quadrature value, so
     * there are 4 for every slot in the encoder disk.
     *
     * Why i64? We have the cycles/memory, and with 64, even at
     * 1 pulse per nanosecond (10,000 revs per second, 100,000 pulses per rev), it will take 292 years to overflow.
     * AKA: we can ignore overflows for the life of the silicon in the processor.
     */
    pub count: i64,
}

/**
 * The encoders configured as part of this hardware.
 */
pub struct Encoders<'a> {
    pub left: Encoder<'a>,
    pub right: Encoder<'a>,
}

pub static ENCODER_STATE: Mutex<RefCell<Option<Encoders>>> = Mutex::new(RefCell::new(None));

impl<'a> Encoders<'a> {
    /**
     * Set up interrupts for the encoders.
     * Note that it is expected that the interrupt itself is bound to
     * `encoder_interrupt_handler` elsewhere, ie via:
     * ```rust
     * io.set_interrupt_handler(encoder_interrupt_handler);
     * ```
     *
     * Or called from the interrupt handler if there are other potential sources of interrupts.
     */
    pub fn configure(&mut self) {
        self.left.a_input.listen(Event::AnyEdge);
        self.left.b_input.listen(Event::AnyEdge);

        self.right.a_input.listen(Event::AnyEdge);
        self.right.b_input.listen(Event::AnyEdge);
    }
}

/**
 * The direction value (+1 or -1) based on the A and B channel values.
 *
 * This is the fastest way to do this, but does not check for errors/out of sequence values.
 */
#[inline]
fn val_to_dir(a: bool, b: bool) -> i64 {
    if a != b { 1 } else { -1 }
}

/**
 * Updates the encoder value based on the current state of the inputs if the interrupt has fired.
 * Clears the interrupt if it was set.
 */
#[inline]
fn check_encoder(encoder: &mut Encoder) {
    if encoder.a_input.is_interrupt_set() {
        encoder.a_input.clear_interrupt();
        encoder.count += val_to_dir(encoder.a_input.is_high(), encoder.b_input.is_high());
    }
    if encoder.b_input.is_interrupt_set() {
        encoder.b_input.clear_interrupt();
        encoder.count -= val_to_dir(encoder.a_input.is_high(), encoder.b_input.is_high());
    }
}

#[handler]
pub fn encoder_interrupt_handler() {
    critical_section::with(|cs| {
        if let Some(encoders) = ENCODER_STATE.borrow(cs).borrow_mut().as_mut() {
            // Left motor encoder handling
            check_encoder(&mut encoders.left);
            check_encoder(&mut encoders.right);
        }
    });
}
