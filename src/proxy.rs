use gateway;
use gateway::*;
use interceptor;
use ota;
use std::sync::mpsc;
use std::thread;

pub struct Proxy {
    gateway_port: String,
    controller_port: String,
}

impl Proxy {
    pub fn new(gateway_port: String, controller_port: String) -> Proxy {
        Proxy {
            gateway_port,
            controller_port,
        }
    }

    pub fn start(
        &self,
        gateway_type: ConnectionType,
        controller_type: ConnectionType,
    ) -> Result<String, String> {
        let mut gateway = Gateway::new(gateway_type, self.gateway_port.clone());
        let mut controller = Gateway::new(controller_type, self.controller_port.clone());

        let mut gateway_port = gateway.clone();

        let mut controller_port = controller.clone();

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
            gateway::write(&mut gateway, &controller_in_receiver);
        });

        let controller_writer = thread::spawn(move || {
            gateway::write(&mut controller, &controller_out_receiver);
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
