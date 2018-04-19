#![feature(plugin)]
#![plugin(rocket_codegen)]
extern crate rocket;

extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate ini;
extern crate myscontroller_rs;

use diesel::prelude::*;
use ini::Ini;
use myscontroller_rs::gateway::ConnectionType;
use myscontroller_rs::{proxy, gateway, api};
use std::fs::create_dir_all;
use std::path::Path;
use std::thread;

fn main() {
    thread::spawn( || {
        rocket::ignite().mount("/", routes![api::index]).launch();
    });

    embed_migrations!("migrations");
    let conf = match Ini::load_from_file("/etc/myscontroller-rs/conf.ini") {
        Ok(_conf) => _conf,
        Err(_) => Ini::load_from_file("conf.ini").unwrap(),
    };

    let (db_connection, firmwares_directory) = server_configs(&conf);
    embedded_migrations::run_with_output(&db_connection, &mut std::io::stdout()).unwrap();

    proxy::start(firmwares_directory, get_mys_gateway(&conf), get_mys_controller(&conf), db_connection)
}


pub fn server_configs(config: &Ini) -> (SqliteConnection, String) {
    let server_conf = config.section(Some("Server".to_owned())).expect("Server configurations missing");
    let database_url = server_conf.get("database_url").expect("database_url is not specified. Ex:database_url=/var/lib/myscontroller-rs/sqlite.db");
    let firmwares_directory = server_conf.get("firmwares_directory").expect("firmwares_directory is not specified. Ex:firmwares_directory=/var/lib/myscontroller-rs/firmwares");
    let firmware_path = Path::new(firmwares_directory);
    let database_path = Path::new(database_url);
    create_dir_all(firmware_path).unwrap();
    create_dir_all(database_path.parent().unwrap()).unwrap();
    (SqliteConnection::establish(&database_url)
         .expect(&format!("Error connecting to {}", database_url)), firmwares_directory.to_owned())
}

fn get_mys_controller<'s>(config: &'s Ini) -> gateway::StreamInfo {
    let controller_conf = config.section(Some("Controller".to_owned())).unwrap();
    let controller_type = controller_conf.get("type").expect("Controller port is not specified. Ex:\n\
     [Controller]\n type=SERIAL\n port=/dev/tty1\n or \n\n[Controller]\n type=SERIAL\n port=port=0.0.0.0:5003");
    let controller_type = match ConnectionType::from_str(controller_type.as_str(), true) {
        Some(value) => value,
        None => panic!("Possible values for type is TCP or SERIAL"),
    };
    let controller_port = controller_conf.get("port").expect("Controller port is not specified. Ex:\n\
     [Controller]\n type=SERIAL\n port=/dev/tty1\n or \n\n[Controller]\n type=SERIAL\n port=port=0.0.0.0:5003");
    gateway::StreamInfo { port: controller_port.to_owned(), connection_type: controller_type }
}

fn get_mys_gateway<'s>(config: &'s Ini) -> gateway::StreamInfo {
    let gateway_conf = config.section(Some("Gateway".to_owned())).unwrap();
    let gateway_type = gateway_conf.get("type").expect("Gateway port is not specified. Ex:\n\
     [Gateway]\n type=SERIAL\n port=/dev/tty1\n or \n\n[Gateway]\n type=SERIAL\n port=port=10.137.120.250:5003");
    let gateway_type = match ConnectionType::from_str(gateway_type.as_str(), false) {
        Some(value) => value,
        None => panic!("Possible values for type is TCP or SERIAL"),
    };
    let gateway_port = gateway_conf.get("port").expect("Gateway port is not specified. Ex:\n\
     [Gateway]\n type=SERIAL\n port=/dev/tty1\n or \n\n[Gateway]\n type=SERIAL\n port=port=10.137.120.250:5003");
    gateway::StreamInfo { port: gateway_port.to_owned(), connection_type: gateway_type }
}