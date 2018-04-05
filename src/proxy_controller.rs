use gateway::SerialGateway;
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
        let controller_in = SerialGateway::new(self.controller_port.clone());
        let gateway_in = SerialGateway::new(self.gateway_port.clone());
        let controller_out = SerialGateway::new(self.controller_port.clone());
        let gateway_out = SerialGateway::new(self.gateway_port.clone());
        let (gateway_tx, gateway_rx) = mpsc::channel();

        let (controller_tx, controller_rx) = mpsc::channel();
        let gateway_reader = thread::spawn(move || {
            gateway_in.read(gateway_tx);
        });
        let controller_reader = thread::spawn(move || {
            controller_in.read(controller_tx);
        });

        let gateway_writer = thread::spawn(move || {
            gateway_out.write(controller_rx);
        });
        let controller_writer = thread::spawn(move || {
            controller_out.write(gateway_rx);
        });

        gateway_reader.join().unwrap();
        controller_reader.join().unwrap();
        gateway_writer.join().unwrap();
        controller_writer.join().unwrap();
    }
}
