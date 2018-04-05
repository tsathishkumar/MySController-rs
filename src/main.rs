extern crate myrcontroller;

use myrcontroller::gateway;
use myrcontroller::gateway::SerialGateway;
use std::env;
use std::thread;
use std::sync::mpsc;

fn main() {
    let gateway_port = env::var("SERIAL_PORT")
        .expect("Serial port is not specified. Ex: 'export SERIAL_PORT=/dev/tty'");

    let controller_port = env::var("PROXY_PORT")
        .expect("Serial port is not specified. Ex: 'export PROXY_PORT=/dev/tty'");
    println!("opening port: {}", gateway_port);

    let controller = SerialGateway::new(controller_port);

    let gateway = SerialGateway::new(gateway_port);

    let (gateway_tx, gateway_rx) = mpsc::channel();

    let (controller_tx, controller_rx) = mpsc::channel();

    let gateway_reader = thread::spawn(move || {
        gateway.connect(gateway_tx);
    });
    let controller_reader = thread::spawn(move || {
        controller.connect(controller_tx);
    });

    gateway_reader.join().unwrap();
    controller_reader.join().unwrap();
}
