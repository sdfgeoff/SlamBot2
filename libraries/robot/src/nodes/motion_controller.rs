use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};
use packet_router::Client;
use topics::{PacketData, PacketDataTrait, PacketFormat, MotionRequestMode};

use crate::nodes::clock::get_current_time;

pub struct MotionController {
    pub client: Rc<RefCell<Client<PacketFormat<PacketData>>>>,
    
    // Current target
    current_target: Option<MotionTarget>,
    
    // Current position estimate
    current_position: [f64; 2],
    current_orientation: f64,
    position_updated: bool,
    
    // Rate limiting
    last_packet_time: Instant,
    packet_interval: Duration,
}

#[derive(Debug)]
struct MotionTarget {
    linear: [f64; 2],
    angular: f64,
    mode: MotionRequestMode,
}

impl MotionController {
    pub fn new() -> Self {
        let client = Rc::new(RefCell::new(Client::default()));
        
        // Subscribe to MotionTargetRequest
        let motion_target_topic = PacketData::MotionTargetRequest(topics::MotionTargetRequest {
            linear: [0.0, 0.0],
            angular: 0.0,
            motion_mode: MotionRequestMode::Velocity,
        })
        .topic()
        .to_string();
        client.borrow_mut().subscriptions.insert(motion_target_topic);

        // Subscribe to PositionEstimate
        let position_estimate_topic = PacketData::PositionEstimate(topics::PositionEstimate {
            timestamp: 0,
            position: [0.0, 0.0],
            orientation: 0.0,
        })
        .topic()
        .to_string();
        client.borrow_mut().subscriptions.insert(position_estimate_topic);

        MotionController {
            client,
            current_target: None,
            current_position: [0.0, 0.0],
            current_orientation: 0.0,
            position_updated: false,
            last_packet_time: Instant::now(),
            packet_interval: Duration::from_millis(100),
        }
    }

    pub fn tick(&mut self) {
        // Process incoming packets
        let packets = self.client.borrow_mut().fetch_all();
        for packet in packets {
            match &packet.data {
                PacketData::MotionTargetRequest(req) => {
                    // Update the current target
                    println!("Received Motion Target Request: {:?}", req);
                    self.current_target = Some(MotionTarget {
                        linear: req.linear,
                        angular: req.angular,
                        mode: req.motion_mode.clone(),
                    });
                }
                PacketData::PositionEstimate(estimate) => {
                    // Update current position estimate
                    self.current_position[0] = estimate.position[0] as f64;
                    self.current_position[1] = estimate.position[1] as f64;
                    self.current_orientation = estimate.orientation as f64;
                    self.position_updated = true;
                }
                _ => {}
            }
        }

        // Generate velocity commands based on current target
        if let Some(target) = &self.current_target && self.last_packet_time.elapsed() >= self.packet_interval {
            self.last_packet_time = Instant::now();
            println!("Sending Velocity Command for Target: {:?}", target);
            let velocity_cmd = match target.mode {
                MotionRequestMode::Velocity => {
                    // Direct velocity control - just pass through the target
                    Some(topics::MotionVelocityRequest {
                        linear_velocity: target.linear[0] as f32,
                        angular_velocity: target.angular as f32,
                    })
                }
                MotionRequestMode::Position => {
                    // Position control - compute velocity to reach target
                    if self.position_updated {
                        self.compute_velocity_to_target(target)
                    } else {
                        // No position estimate yet, don't move
                        None
                    }
                }
                MotionRequestMode::Stop => {
                    // Stop mode - send zero velocities
                    Some(topics::MotionVelocityRequest {
                        linear_velocity: 0.0,
                        angular_velocity: 0.0,
                    })
                }
            };

            // Send velocity command
            if let Some(cmd) = velocity_cmd {
                let packet = PacketFormat {
                    to: None,
                    from: None,
                    data: PacketData::MotionVelocityRequest(cmd),
                    time: get_current_time(),
                    id: 0,
                };
                self.client.borrow_mut().send(packet);
                self.last_packet_time = Instant::now();
            }
        }
    }

    fn compute_velocity_to_target(&self, target: &MotionTarget) -> Option<topics::MotionVelocityRequest> {
        // Calculate error in position
        let dx = target.linear[0] - self.current_position[0];
        let dy = target.linear[1] - self.current_position[1];
        
        let distance = (dx * dx + dy * dy).sqrt();
        
        // If we're close enough to target, stop
        const POSITION_TOLERANCE: f64 = 0.05; // 5cm tolerance
        if distance < POSITION_TOLERANCE {
            return Some(topics::MotionVelocityRequest {
                linear_velocity: 0.0,
                angular_velocity: 0.0,
            });
        }
        
        // Calculate desired heading to target
        let desired_heading = dy.atan2(dx);
        
        // Calculate heading error
        let mut heading_error = desired_heading - self.current_orientation;
        
        // Normalize heading error to [-PI, PI]
        while heading_error > std::f64::consts::PI {
            heading_error -= 2.0 * std::f64::consts::PI;
        }
        while heading_error < -std::f64::consts::PI {
            heading_error += 2.0 * std::f64::consts::PI;
        }
        
        // Simple proportional controller
        const KP_LINEAR: f64 = 0.5;  // Linear velocity gain
        const KP_ANGULAR: f64 = 2.0; // Angular velocity gain
        const MAX_LINEAR_VEL: f64 = 0.5;  // m/s
        const MAX_ANGULAR_VEL: f64 = 2.0; // rad/s
        
        // Compute velocities
        let mut linear_velocity = KP_LINEAR * distance;
        let mut angular_velocity = KP_ANGULAR * heading_error;
        
        // Clamp velocities to max values
        linear_velocity = linear_velocity.clamp(-MAX_LINEAR_VEL, MAX_LINEAR_VEL);
        angular_velocity = angular_velocity.clamp(-MAX_ANGULAR_VEL, MAX_ANGULAR_VEL);
        
        // Reduce linear velocity if we need to turn significantly
        if heading_error.abs() > std::f64::consts::PI / 4.0 {
            // If heading error is more than 45 degrees, slow down forward motion
            linear_velocity *= 0.3;
        }
        
        Some(topics::MotionVelocityRequest {
            linear_velocity: linear_velocity as f32,
            angular_velocity: angular_velocity as f32,
        })
    }
}
