use diesel::prelude::*;
use gateway::*;
use interceptor;
use node;
use ota;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;

pub fn start(firmwares_directory: String, mut mys_gateway_writer: Box<Gateway>,
             mut mys_controller_writer: Box<Gateway>, db_connection: SqliteConnection) {
    let stop_thread = Arc::new(Mutex::new(false));
    let mut mys_gateway_reader = mys_gateway_writer.clone();
    let mut mys_controller_reader = mys_controller_writer.clone();

    let (gateway_sender, gateway_receiver) = mpsc::channel();
    let (ota_sender, ota_receiver) = mpsc::channel();
    let (controller_in_sender, controller_in_receiver) = mpsc::channel();
    let (controller_out_sender, controller_out_receiver) = mpsc::channel();
    let (node_manager_sender, node_manager_in) = mpsc::channel();
    let ota_fw_sender = controller_in_sender.clone();
    let node_manager_out = controller_in_sender.clone();

    //TODO: better handle stop notifications to threads
    let (stop_thread_clone, stop_thread_clone1
        , stop_thread_clone2, stop_thread_clone3, stop_thread_clone4,
        stop_thread_clone5, stop_thread_clone6) = clone_stop_thread_flag(&stop_thread);
    let gateway_reader = thread::spawn(move || {
        mys_gateway_reader.read_loop(stop_thread_clone, &gateway_sender);
    });

    let gateway_writer = thread::spawn(move || {
        mys_gateway_writer.write_loop(stop_thread_clone1, &controller_in_receiver);
    });

    let controller_reader = thread::spawn(move || {
        mys_controller_reader.read_loop(stop_thread_clone2, &controller_in_sender);
    });

    let controller_writer = thread::spawn(move || {
        mys_controller_writer.write_loop(stop_thread_clone3, &controller_out_receiver);
    });

    let message_interceptor = thread::spawn(move || {
        interceptor::intercept(stop_thread_clone4, &gateway_receiver, &ota_sender, &node_manager_sender, &controller_out_sender);
    });

    let ota_processor = thread::spawn(move || {
        ota::process_ota(&firmwares_directory, stop_thread_clone5, &ota_receiver, &ota_fw_sender);
    });

    let node_manager = thread::spawn(move || {
        node::handle_node_id_request(stop_thread_clone6, &node_manager_in, &node_manager_out, db_connection);
    });

    match message_interceptor.join() {
        Ok(_) => (),
        Err(_) => *stop_thread.lock().unwrap() = true,
    };
    match gateway_reader.join() {
        Ok(_) => (),
        Err(_) => *stop_thread.lock().unwrap() = true,
    };
    match controller_reader.join() {
        Ok(_) => (),
        Err(_) => *stop_thread.lock().unwrap() = true,
    };
    match gateway_writer.join() {
        Ok(_) => (),
        Err(_) => *stop_thread.lock().unwrap() = true,
    };
    match controller_writer.join() {
        Ok(_) => (),
        Err(_) => *stop_thread.lock().unwrap() = true,
    };
    match ota_processor.join() {
        Ok(_) => (),
        Err(_) => *stop_thread.lock().unwrap() = true,
    };
    match node_manager.join() {
        Ok(_) => (),
        Err(_) => *stop_thread.lock().unwrap() = true,
    };
}

fn clone_stop_thread_flag(stop_thread: &Arc<Mutex<bool>>)
                          -> (Arc<Mutex<bool>>, Arc<Mutex<bool>>, Arc<Mutex<bool>>,
                              Arc<Mutex<bool>>, Arc<Mutex<bool>>, Arc<Mutex<bool>>, Arc<Mutex<bool>>) {
    let stop_thread_clone = Arc::clone(stop_thread);
    let stop_thread_clone1 = Arc::clone(stop_thread);
    let stop_thread_clone2 = Arc::clone(stop_thread);
    let stop_thread_clone3 = Arc::clone(stop_thread);
    let stop_thread_clone4 = Arc::clone(stop_thread);
    let stop_thread_clone5 = Arc::clone(stop_thread);
    let stop_thread_clone6 = Arc::clone(stop_thread);
    (stop_thread_clone, stop_thread_clone1, stop_thread_clone2, stop_thread_clone3, stop_thread_clone4,
     stop_thread_clone5, stop_thread_clone6)
}

