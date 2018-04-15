extern crate ini;
extern crate myrcontroller;

use ini::Ini;
use myrcontroller::gateway::{ConnectionType, Gateway};
use myrcontroller::gateway;
use myrcontroller::proxy;

fn main() {
    let conf = Ini::load_from_file("conf.ini").unwrap();

    loop {
        let mys_gateway = get_mys_gateway(&conf);
        let mys_controller = get_mys_controller(&conf);

        match proxy::start(mys_gateway, mys_controller) {
            Ok(_) => (),
            Err(_) => (),
        };
        println!("main loop ended");
    }
}

fn get_mys_controller<'s>(config: &'s Ini) -> Box<Gateway> {
    let controller_conf = config.section(Some("Controller".to_owned())).unwrap();
    let controller_type = controller_conf.get("type").expect("Controller port is not specified. Ex:\n\
     [Controller]\n type=SERIAL\n port=/dev/tty1\n or \n\n[Controller]\n type=SERIAL\n port=port=0.0.0.0:5003");
    let controller_type = match ConnectionType::from_str(controller_type.as_str(), true) {
        Some(value) => value,
        None => panic!("Possible values for type is TCP or SERIAL"),
    };
    let controller_port = controller_conf.get("port").expect("Controller port is not specified. Ex:\n\
     [Controller]\n type=SERIAL\n port=/dev/tty1\n or \n\n[Controller]\n type=SERIAL\n port=port=0.0.0.0:5003");
    gateway::create_gateway(controller_type, controller_port.clone())
}

fn get_mys_gateway<'s>(config: &'s Ini) -> Box<Gateway> {
    let gateway_conf = config.section(Some("Gateway".to_owned())).unwrap();
    let gateway_type = gateway_conf.get("type").expect("Gateway port is not specified. Ex:\n\
     [Gateway]\n type=SERIAL\n port=/dev/tty1\n or \n\n[Gateway]\n type=SERIAL\n port=port=10.137.120.250:5003");
    let gateway_type = match ConnectionType::from_str(gateway_type.as_str(), false) {
        Some(value) => value,
        None => panic!("Possible values for type is TCP or SERIAL"),
    };
    let gateway_port = gateway_conf.get("port").expect("Gateway port is not specified. Ex:\n\
     [Gateway]\n type=SERIAL\n port=/dev/tty1\n or \n\n[Gateway]\n type=SERIAL\n port=port=10.137.120.250:5003");
    gateway::create_gateway(gateway_type, gateway_port.clone())
}
