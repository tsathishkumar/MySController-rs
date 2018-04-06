use serialport;
use serialport::prelude::*;

use gateway;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;


pub struct ProxyController {
    gateway_port: String,
    controller_port: String,
}

impl ProxyController {
    pub fn new(gateway_port: String, controller_port: String) -> ProxyController {
        ProxyController {
            gateway_port: gateway_port,
            controller_port: controller_port,
        }
    }

    pub fn start(&self) {
        let mut settings: SerialPortSettings = Default::default();
        settings.timeout = Duration::from_millis(10);
        settings.baud_rate = BaudRate::Baud38400;
        let mut gateway_port = serialport::open_with_settings(&self.gateway_port, &settings).unwrap();
        let mut gateway_write_port = gateway_port.try_clone().unwrap();
        let mut controller_port = serialport::open_with_settings(&self.controller_port, &settings).unwrap();
        let mut controller_write_port = controller_port.try_clone().unwrap();

        let (gateway_tx, gateway_rx) = mpsc::channel();
        let (controller_tx, controller_rx) = mpsc::channel();
        
        let gateway_reader = thread::spawn(move || {
            gateway::read(&mut gateway_port, &gateway_tx);
        });
        let controller_reader = thread::spawn(move || {
            gateway::read(&mut controller_port, &controller_tx);
        });

        let gateway_writer = thread::spawn(move || {
            gateway::write(&mut gateway_write_port, &controller_rx);
        });
        
        let controller_writer = thread::spawn(move || {
            gateway::write(&mut controller_write_port, &gateway_rx);
        });

        gateway_reader.join().unwrap();
        controller_reader.join().unwrap();
        gateway_writer.join().unwrap();
        controller_writer.join().unwrap();
    }
}
