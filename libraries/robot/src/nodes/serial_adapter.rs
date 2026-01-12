use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::time::{Duration, Instant};
use serialport::{available_ports, SerialPortType};

use packet_router::Router;
use topics::{PacketData, PacketFormat};

use crate::nodes::serial_client::SerialClient;

pub struct SerialAdapter {
    pub router: Rc<RefCell<Router<PacketFormat<PacketData>>>>,
    pub clients_by_path: HashMap<String, SerialClient>,
    pub last_scan_time: Instant,
    pub scan_interval: Duration,
}

impl SerialAdapter {
    pub fn new(router: Rc<RefCell<Router<PacketFormat<PacketData>>>>, scan_interval: Duration) -> Self {
        println!("SerialAdapter initialized with scan interval: {:?}", scan_interval);
        SerialAdapter {
            router,
            clients_by_path: HashMap::new(),
            last_scan_time: Instant::now(),
            scan_interval,
        }
    }

    fn is_target_device(port_info: &serialport::SerialPortInfo) -> bool {
        // Filter for Espressif USB JTAG serial debug units
        // You can customize this filter based on your specific devices
        match &port_info.port_type {
            SerialPortType::UsbPort(usb_info) => {
                // Espressif USB JTAG typically has VID 0x303A
                if usb_info.vid == 0x303A {
                    return true;
                }
            }
            _ => {}
        }
        
        // Also check if the path contains the expected pattern
        if port_info.port_name.contains("usb-Espressif_USB_JTAG_serial_debug_unit") {
            return true;
        }
        
        false
    }

    fn scan_ports(&mut self) {
        match available_ports() {
            Ok(ports) => {
                for port_info in ports {
                    let port_path = port_info.port_name.clone();
                    
                    // Skip if not a target device
                    if !Self::is_target_device(&port_info) {
                        continue;
                    }
                    
                    // Skip if we already have a client for this port
                    if self.clients_by_path.contains_key(&port_path) {
                        continue;
                    }
                    
                    // Try to open the serial port
                    match serialport::new(&port_path, 115_200)
                        .timeout(Duration::from_millis(1))
                        .open()
                    {
                        Ok(serial_port) => {
                            println!("New serial device connected: {}", port_path);
                            let client = SerialClient::new(serial_port);
                            self.router.borrow_mut().register_client(Rc::downgrade(&client.client));
                            self.clients_by_path.insert(port_path.clone(), client);
                        }
                        Err(e) => {
                            println!("Failed to open serial port {}: {:?}", port_path, e);
                        }
                    }
                }
            }
            Err(e) => {
                println!("Error scanning for serial ports: {:?}", e);
            }
        }
    }

    pub fn tick(&mut self) {
        // Periodically scan for new ports
        if self.last_scan_time.elapsed() >= self.scan_interval {
            self.scan_ports();
            self.last_scan_time = Instant::now();
        }

        // Tick all existing clients
        for (_path, client) in self.clients_by_path.iter_mut() {
            client.tick();
        }

        // Remove dead clients
        self.clients_by_path.retain(|path, client| {
            if !client.is_alive {
                println!("Serial device disconnected: {}", path);
                false
            } else {
                true
            }
        });
    }
}
