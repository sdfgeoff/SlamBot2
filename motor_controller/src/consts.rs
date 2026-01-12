use core::f32::consts::PI;

pub const WHEEL_CIRCUMFERENCE: f32 = PI * 2.0 * 0.02; // meters
pub const WHEEL_BASE_WIDTH: f32 = 0.2; // meters
pub const ENCODER_TICKS_PER_REVOLUTION: f32 = 11.0 * 4.0 * 35.0; // encoder * quadrature * gearbox
pub const NOMINAL_MAX_RPM: f32 = 120.0; // RPM
