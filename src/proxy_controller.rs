use serialport;
use serialport::prelude::*;

use gateway;
use interceptor;
use ota;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use std::net::{TcpListener, TcpStream};

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

    pub fn start(&self) -> Result<String, String> {
        let mut settings: SerialPortSettings = Default::default();
        settings.timeout = Duration::from_millis(10);
        settings.baud_rate = BaudRate::Baud38400;

        let mut gateway_port = TcpStream::connect("10.137.120.250:5003").unwrap();
        // let mut settings: SerialPortSettings = Default::default();
        //     settings.timeout = Duration::from_millis(10);
        //     settings.baud_rate = BaudRate::Baud38400;
        // let mut gateway_port =
        //     serialport::open_with_settings(&self.gateway_port, &settings).unwrap();

        // let mut controller_port =
        //     serialport::open_with_settings(&self.controller_port, &settings).unwrap();
        let mut gateway_write_port = gateway_port.try_clone().unwrap();

        let listener = TcpListener::bind("0.0.0.0:5003").unwrap();

        let (mut controller_port, _) = listener.accept().unwrap();
        let mut controller_write_port = controller_port.try_clone().unwrap();

        let (gateway_sender, gateway_receiver) = mpsc::channel();
        let (ota_sender, ota_receiver) = mpsc::channel();
        let (controller_in_sender, controller_in_receiver) = mpsc::channel();
        let (controller_out_sender, controller_out_receiver) = mpsc::channel();
        let ota_fw_sender = controller_in_sender.clone();

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
            ota::process_ota(&ota_receiver, &ota_fw_sender);
        });

        match message_interceptor.join() {
            Ok(_result) => (),
            Err(_error) => return Err(String::from("Error in Message interceptor")),
        }
        match gateway_reader.join() {
            Ok(_result) => (),
            Err(_error) => return Err(String::from("Error in Gateway reader")),
        };
        match controller_reader.join() {
            Ok(_) => (),
            _ => return Err(String::from("Error in Controller reader")),
        }
        match gateway_writer.join() {
            Ok(_) => (),
            _ => return Err(String::from("Error in Gateway writer")),
        };
        match controller_writer.join() {
            Ok(_) => (),
            _ => return Err(String::from("Error in Controller writer")),
        };
        match ota_processor.join() {
            Ok(_) => (),
            _ => return Err(String::from("Error in OTA processor")),
        };
        Ok(String::from("Done"))
    }
}
