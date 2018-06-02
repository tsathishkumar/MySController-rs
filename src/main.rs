#![feature(plugin)]
#![plugin(rocket_codegen)]
extern crate crossbeam_channel as channel;
#[macro_use]
extern crate diesel_migrations;
extern crate ini;
extern crate myscontroller_rs;
extern crate rocket;

use ini::Ini;
use myscontroller_rs::api::{node, firmware};
use myscontroller_rs::model::db;
use myscontroller_rs::core::{connection, proxy};
use std::fs::create_dir_all;
use std::path::Path;
use std::thread;

fn main() {
    embed_migrations!("migrations");

    let conf = match Ini::load_from_file("/etc/myscontroller-rs/conf.ini") {
        Ok(_conf) => _conf,
        Err(_) => Ini::load_from_file("conf.ini").unwrap(),
    };

    let database_url= server_configs(&conf);
    let pool = db::init_pool(database_url);

    let pool_clone = pool.clone();
    let (controller_in_sender, controller_in_receiver) = channel::unbounded();
    let reset_signal_sender = controller_in_sender.clone();
    thread::spawn(|| {
        rocket::ignite()
            .manage(pool_clone)
            .manage(reset_signal_sender)
            .mount("/", routes![node::index, node::list, node::update_node, node::reboot_node])
            .mount("/", routes![firmware::upload, firmware::list, firmware::update, firmware::delete])
            .launch();
    });

    embedded_migrations::run_with_output(&pool.get().unwrap(), &mut std::io::stdout()).unwrap();
    proxy::start(get_mys_gateway(&conf),
                 get_mys_controller(&conf), pool, controller_in_sender, controller_in_receiver)
}


pub fn server_configs(config: &Ini) -> String {
    let server_conf = config.section(Some("Server".to_owned())).expect("Server configurations missing");
    let database_url = server_conf.get("database_url").expect("database_url is not specified. Ex:database_url=/var/lib/myscontroller-rs/sqlite.db");
    let database_path = Path::new(database_url);
    create_dir_all(database_path.parent().unwrap()).unwrap();
    database_url.to_owned()
}

fn get_mys_controller<'s>(config: &'s Ini) -> connection::StreamInfo {
    let controller_conf = config.section(Some("Controller".to_owned())).unwrap();
    let controller_type = controller_conf.get("type").expect("Controller port is not specified. Ex:\n\
     [Controller]\n type=SERIAL\n port=/dev/tty1\n or \n\n[Controller]\n type=SERIAL\n port=port=0.0.0.0:5003");
    let controller_type = match connection::ConnectionType::from_str(controller_type.as_str(), true) {
        Some(value) => value,
        None => panic!("Possible values for type is TCP or SERIAL"),
    };
    let controller_port = controller_conf.get("port").expect("Controller port is not specified. Ex:\n\
     [Controller]\n type=SERIAL\n port=/dev/tty1\n or \n\n[Controller]\n type=SERIAL\n port=port=0.0.0.0:5003");
    connection::StreamInfo { port: controller_port.to_owned(), connection_type: controller_type }
}

fn get_mys_gateway<'s>(config: &'s Ini) -> connection::StreamInfo {
    let gateway_conf = config.section(Some("Gateway".to_owned())).unwrap();
    let gateway_type = gateway_conf.get("type").expect("Gateway port is not specified. Ex:\n\
     [Gateway]\n type=SERIAL\n port=/dev/tty1\n or \n\n[Gateway]\n type=SERIAL\n port=port=10.137.120.250:5003");
    let gateway_type = match connection::ConnectionType::from_str(gateway_type.as_str(), false) {
        Some(value) => value,
        None => panic!("Possible values for type is TCP or SERIAL"),
    };
    let gateway_port = gateway_conf.get("port").expect("Gateway port is not specified. Ex:\n\
     [Gateway]\n type=SERIAL\n port=/dev/tty1\n or \n\n[Gateway]\n type=SERIAL\n port=port=10.137.120.250:5003");
    connection::StreamInfo { port: gateway_port.to_owned(), connection_type: gateway_type }
}