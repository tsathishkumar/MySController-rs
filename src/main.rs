extern crate myrcontroller;

use myrcontroller::proxy_controller::ProxyController;

use std::env;

fn main() {
    let gateway_port = env::var("SERIAL_PORT")
        .expect("Serial port is not specified. Ex: 'export SERIAL_PORT=/dev/tty'");

    let controller_port = env::var("PROXY_PORT")
        .expect("Serial port is not specified. Ex: 'export PROXY_PORT=/dev/tty'");

    let proxy_controller = ProxyController::new(gateway_port, controller_port);
    loop {
        match proxy_controller.start() {
            Ok(_) => (),
            Err(_) => (),
        };
    }
}
