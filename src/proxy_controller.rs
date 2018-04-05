use gateway::{SerialReader, SerialWriter};
use std::sync::mpsc;
use std::thread;

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
        let (gateway_tx, gateway_rx) = mpsc::channel();
        let (controller_tx, controller_rx) = mpsc::channel();
        let controller_in = SerialReader::new(self.controller_port.clone(),controller_tx);
        let gateway_in = SerialReader::new(self.gateway_port.clone(), gateway_tx);
        let controller_out = SerialWriter::new(self.controller_port.clone(), gateway_rx);
        let gateway_out = SerialWriter::new(self.gateway_port.clone(), controller_rx);
        
        let gateway_reader = thread::spawn(move || {
            gateway_in.read();
        });
        let controller_reader = thread::spawn(move || {
            controller_in.read();
        });

        // let gateway_writer = thread::spawn(move || {
        //     gateway_out.write();
        // });
        let controller_writer = thread::spawn(move || {
            controller_out.write();
        });

        gateway_reader.join().unwrap();
        controller_reader.join().unwrap();
        // gateway_writer.join().unwrap();
        controller_writer.join().unwrap();
    }
}
