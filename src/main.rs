extern crate myrcontroller;

use myrcontroller::*;
use std::env;

fn main() {
    let serial_port = env::var("SERIAL_PORT")
        .expect("Serial port is not specified. Ex: 'export SERIAL_PORT=/dev/tty'");
    println!("opening port: {}", serial_port);
    let gateway = gateway::SerialGateway {
        serial_port: serial_port,
    };
    gateway.connect();
}
