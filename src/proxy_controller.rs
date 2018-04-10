use serialport;
use serialport::prelude::*;

use gateway;
use interceptor;
use ota;
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
        let mut gateway_port =
            serialport::open_with_settings(&self.gateway_port, &settings).unwrap();
        let mut gateway_write_port = gateway_port.try_clone().unwrap();
        let mut controller_port =
            serialport::open_with_settings(&self.controller_port, &settings).unwrap();
        let mut controller_write_port = controller_port.try_clone().unwrap();

        let (gateway_sender, gateway_receiver) = mpsc::channel();
        let (ota_sender, ota_receiver) = mpsc::channel();
        let (controller_in_sender, controller_in_receiver) = mpsc::channel();
        let (controller_out_sender, controller_out_receiver) = mpsc::channel();

        let gateway_reader = thread::spawn(move || {
            gateway::read(&mut gateway_port, &gateway_sender);
        });
        let controller_reader = thread::spawn(move || {
            gateway::read(&mut controller_port, &controller_in_sender);
        });

        let message_interceptor = thread::spawn(move || {
            interceptor::intercept(&gateway_receiver, &ota_sender, &controller_out_sender);
        });

        let gateway_writer = thread::spawn(move || {
            gateway::write(&mut gateway_write_port, &controller_in_receiver);
        });

        let controller_writer = thread::spawn(move || {
            gateway::write(&mut controller_write_port, &controller_out_receiver);
        });

        let ota_processor = thread::spawn(move || {
            ota::process_ota(&ota_receiver);
        });

        message_interceptor.join().unwrap();
        gateway_reader.join().unwrap();
        controller_reader.join().unwrap();
        gateway_writer.join().unwrap();
        controller_writer.join().unwrap();
        ota_processor.join().unwrap();
    }
}
