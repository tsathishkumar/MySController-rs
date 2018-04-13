extern crate myrcontroller;

use myrcontroller::gateway::ConnectionType;
use myrcontroller::proxy_controller::ProxyController;

use std::env;

fn main() {
    let gateway_connection = env::var("GATEWAY_CONNECTION")
        .expect("Serial port is not specified. Possible values 'TCP_CLIENT' or 'SERIAL' \n Ex: 'export GATEWAY_CONNECTION=TCP'");
    let gateway_type = match ConnectionType::from_str(gateway_connection.as_str()) {
        Some(value) => value,
        None => panic!("Possible values for GATEWAY_CONNECTION is 'TCP_CLIENT' or 'SERIAL'"),
    };
    let controller_connection = env::var("CONTROLLER_CONNECTION")
        .expect("Serial port is not specified. Possible values 'TCP_SERVER' or 'SERIAL' \n Ex: 'export CONTROLLER_CONNECTION=SERIAL'");
    let controller_type = match ConnectionType::from_str(controller_connection.as_str()) {
        Some(value) => value,
        None => panic!("Possible values for CONTROLLER_CONNECTION is 'TCP_SERVER' or 'SERIAL'"),
    };
    let gateway_port = env::var("GATEWAY_PORT")
        .expect("Gateway port is not specified. Ex: 'export GATEWAY_PORT=/dev/tty1' or 'export SERIAL_PORT=10.137.120.250:5003'");

    let controller_port = env::var("CONTROLLER_PORT")
        .expect("Controller port is not specified. Ex: 'export CONTROLLER_PORT=/dev/tty2' or 'export CONTROLLER_PORT=0.0.0.0:5003'");

    let proxy_controller = ProxyController::new(gateway_port, controller_port);
    loop {
        match proxy_controller.start(gateway_type, controller_type) {
            Ok(_) => (),
            Err(_) => (),
        };
    }
}
