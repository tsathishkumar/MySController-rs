#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate log;

use std::fs::create_dir_all;
use std::path::Path;
use std::thread;

use actix;
use actix::*;
use actix_web::{App, http::Method, middleware, middleware::cors::Cors, server};
use crossbeam_channel as channel;
use diesel::prelude::SqliteConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use env_logger;
use num_cpus;

use myscontroller_rs::api::{firmware, index, node, sensor};
use myscontroller_rs::api::index::AppState;
use myscontroller_rs::core::{connection, server as mys_controller};
use myscontroller_rs::core::connection::ConnectionType;
use myscontroller_rs::model::db;
use myscontroller_rs::wot;

mod config;
use crate::config::model::Config;

fn main() {
    embed_migrations!("migrations");

    let sys = actix::System::new("webapp");

    let conf: Config = match config::parser::parse() {
        Ok(_conf) => _conf,
        Err(err) => panic!(format!("The configuration file could not be parsed properly {}", err))
    };

    ::std::env::set_var("RUST_LOG", log_level(&conf));
    ::std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let database_url = server_configs(&conf);
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let conn = Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");
    let conn_clone = conn.clone();
    let database_addr = SyncArbiter::start(num_cpus::get() * 4, move || db::ConnDsl(conn.clone()));

    let (controller_in_sender, controller_in_receiver) = channel::unbounded();
    let (out_set_sender, out_set_receiver) = channel::unbounded();
    let (in_set_sender, in_set_receiver) = channel::unbounded();
    let (new_sensor_sender, new_sensor_receiver) = channel::unbounded();
    let reset_signal_sender = controller_in_sender.clone();
    server::new(move || {
        App::with_state(AppState {
            db: database_addr.clone(),
            reset_sender: reset_signal_sender.clone(),
        })
            .middleware(middleware::Logger::default())
            .configure(|app| {
                Cors::for_app(app)
                    .allowed_methods(vec!["GET", "PUT", "POST", "DELETE"])
                    .resource("/", |r| {
                        r.method(Method::GET).h(index::home);
                    })
                    .resource("/nodes", |r| {
                        r.method(Method::GET).h(node::list);
                        r.method(Method::POST).with(node::create);
                        r.method(Method::PUT).with(node::update);
                        r.method(Method::DELETE).with(node::delete);
                    })
                    .resource("/nodes/{node_id}", |r| {
                        r.method(Method::GET).h(node::get_node);
                    })
                    .resource("/nodes/{node_id}/reboot", |r| {
                        r.method(Method::POST).h(node::reboot_node);
                    })
                    .resource("/sensors", |r| {
                        r.method(Method::GET).h(sensor::list);
                        r.method(Method::DELETE).with(node::delete);
                    })
                    .resource("/sensors/{node_id}/{child_sensor_id}", |r| {
                        r.method(Method::GET).h(sensor::get_sensor);
                    })
                    .resource("/firmwares", |r| {
                        r.method(Method::GET).h(firmware::list);
                    })
                    .resource("/firmwares/{firmware_type}/{firmware_version}", |r| {
                        r.method(Method::POST).with(firmware::create);
                        r.method(Method::PUT).with(firmware::update);
                        r.method(Method::DELETE).f(firmware::delete);
                    })
                    .resource("/firmwares/upload", |r| {
                        r.method(Method::GET).f(firmware::upload_form);
                    })
                    .register()
            })
    })
        .bind("0.0.0.0:8000")
        .unwrap()
        .shutdown_timeout(3)
        .start();

    match conn_clone.get() {
        Ok(conn) => embedded_migrations::run_with_output(&conn, &mut std::io::stdout()).unwrap(),
        Err(e) => error!("Error while running migration {:?}", e),
    };


    info!("Starting proxy server");

    let conn_pool_clone = conn_clone.clone();

    thread::spawn(move || {
        mys_controller::start(
            get_mys_gateway(&conf),
            get_mys_controller(&conf),
            conn_clone,
            controller_in_sender,
            controller_in_receiver,
            in_set_sender,
            out_set_receiver,
            new_sensor_sender,
        );
    });

    info!("Started proxy server");

    thread::spawn(move || {
        let (restart_sender, restart_receiver) = channel::unbounded();
        let restart_sender_clone = restart_sender.clone();
        let conn_pool = conn_pool_clone.clone();
        let out_set_sender_clone = out_set_sender.clone();
        let in_set_receiver_clone = in_set_receiver.clone();
        let new_sensor_receiver_clone = new_sensor_receiver.clone();
        wot::start_server(
            conn_pool,
            out_set_sender,
            in_set_receiver,
            new_sensor_receiver,
            restart_sender,
        );
        loop {
            let conn_pool_clone = conn_pool_clone.clone();
            let restart_sender_clone = restart_sender_clone.clone();
            let out_set_sender_clone = out_set_sender_clone.clone();
            let in_set_receiver_clone = in_set_receiver_clone.clone();
            let new_sensor_receiver_clone = new_sensor_receiver_clone.clone();
            match restart_receiver.recv() {
                Ok(_token) => wot::start_server(
                    conn_pool_clone,
                    out_set_sender_clone,
                    in_set_receiver_clone,
                    new_sensor_receiver_clone,
                    restart_sender_clone,
                ),
                Err(_e) => (),
            }
        }
    });

    info!("Started WoT server");

    sys.run();
}

pub fn server_configs(config: &Config) -> String {
    let server_conf = match &config.Server {
        Some(_config) => _config,
        None => panic!("Server Configurations missing"),
    };

    let database_url = match &server_conf.database_url {
        Some(_database_url) => _database_url,
        None => panic!("database_url is not specified. Ex:database_url=/var/lib/myscontroller-rs/sqlite.db"),
    };

    let database_path = Path::new(&database_url);
    create_dir_all(database_path.parent().unwrap()).unwrap();
    database_url.to_owned()
}

pub fn log_level(config: &Config) -> String {
    let default_log_level = String::from("myscontroller_rs=info,actix_web=info");

    let server_conf = match &config.Server {
        Some(_config) => _config,
        None => return default_log_level,
    };

    let log_level = match &server_conf.log_level {
        Some(_log_level) => _log_level,
        None => return default_log_level,
    };
    log_level.to_owned()
}


fn get_mys_controller(config: &Config) -> Option<connection::ConnectionType> {
    let controller_conf = match &config.Controller {
        Some(_controller_conf) => _controller_conf,
        None => return None
    };

    let controller_type = match &controller_conf.r#type {
        Some(_controller_type) => _controller_type,
        None => panic!("Controller type is not specified. Ex:\n\
     [Controller]\n type=SERIAL\n port=/dev/tty1\n or \n\n[Controller]\n type=SERIAL\n port=port=0.0.0.0:5003)"),
    };

    let port = match &controller_conf.port {
        Some(_port) => _port,
        None => panic!("Controller port is not specified. Ex:\n\
     [Controller]\n type=SERIAL\n port=/dev/tty1\n or \n\n[Controller]\n type=SERIAL\n port=port=0.0.0.0:5003)"),
    };

    let timeout_enabled = match &controller_conf.timeout_enabled {
        Some(_timeout_enabled) => _timeout_enabled.parse::<bool>().unwrap(),
        None => false
    };

    let baud_rate = match &controller_conf.baud_rate {
        Some(_baud_rate) => _baud_rate.parse::<u32>().unwrap(),
        None => 9600
    };

    if controller_type == "Serial" {
        return Some(ConnectionType::Serial { port: port.to_owned(), baud_rate });
    }
    if controller_type == "TCP" {
        return Some(ConnectionType::TcpServer { port: port.to_owned(), timeout_enabled });
    }
    let broker = controller_conf.broker.as_ref().unwrap();
    let port_number = port.parse::<u16>().unwrap();
    let publish_topic_prefix = controller_conf.publish_topic_prefix.as_ref().unwrap();
    Some(ConnectionType::MQTT { broker: broker.to_owned(), port: port_number, publish_topic_prefix: publish_topic_prefix.to_owned() })
}

fn get_mys_gateway(config: &Config) -> connection::ConnectionType {
    let gateway_conf = match &config.Gateway {
        Some(_controller_conf) => _controller_conf,
        None => panic!("Gateway configuration is missing"),
    };

    let gateway_type = match &gateway_conf.r#type {
        Some(_controller_type) => _controller_type,
        None => panic!("Gateway type is not specified. Ex:\n\
     [Gateway]\n type=SERIAL\n port=/dev/tty1\n or \n\n[Gateway]\n type=SERIAL\n port=port=0.0.0.0:5003)"),
    };

    let port = match &gateway_conf.port {
        Some(_port) => _port,
        None => panic!("Gateway port is not specified. Ex:\n\
     [Gateway]\n type=SERIAL\n port=/dev/tty1\n or \n\n[Gateway]\n type=SERIAL\n port=port=0.0.0.0:5003)"),
    };

    let timeout_enabled = match &gateway_conf.timeout_enabled {
        Some(_timeout_enabled) => _timeout_enabled.parse::<bool>().unwrap(),
        None => false
    };

    let baud_rate = match &gateway_conf.baud_rate {
        Some(_baud_rate) => _baud_rate.parse::<u32>().unwrap(),
        None => 9600
    };

    if gateway_type == "Serial" {
        return ConnectionType::Serial { port: port.to_owned(), baud_rate };
    }
    if gateway_type == "TCP" {
        return ConnectionType::TcpClient { port: port.to_owned(), timeout_enabled };
    }
    let broker = gateway_conf.broker.as_ref().unwrap();
    let port_number = port.parse::<u16>().unwrap();
    let publish_topic_prefix = gateway_conf.publish_topic_prefix.as_ref().unwrap();
    ConnectionType::MQTT { broker: broker.to_owned(), port: port_number, publish_topic_prefix: publish_topic_prefix.to_owned() }
}