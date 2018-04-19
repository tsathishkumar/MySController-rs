use diesel::prelude::*;
use gateway::*;
use interceptor;
use node;
use ota;
use std::sync::mpsc;
use std::thread;

pub fn start(firmwares_directory: String, gateway_info: StreamInfo,
             controller_info: StreamInfo, db_connection: SqliteConnection) {
    let (gateway_sender, gateway_receiver) = mpsc::channel();
    let (ota_sender, ota_receiver) = mpsc::channel();
    let (controller_in_sender, controller_in_receiver) = mpsc::channel();
    let (controller_out_sender, controller_out_receiver) = mpsc::channel();
    let (node_manager_sender, node_manager_in) = mpsc::channel();
    let ota_fw_sender = controller_in_sender.clone();
    let node_manager_out = controller_in_sender.clone();


    let message_interceptor = thread::spawn(move || {
        interceptor::intercept(&gateway_receiver, &ota_sender, &node_manager_sender, &controller_out_sender);
    });

    let ota_processor = thread::spawn(move || {
        ota::process_ota(&firmwares_directory, &ota_receiver, &ota_fw_sender);
    });

    let node_manager = thread::spawn(move || {
        node::handle_node_id_request(&node_manager_in, &node_manager_out, db_connection);
    });
    let gateway_read_write = thread::spawn(move || {
        stream_read_write(gateway_info, gateway_sender, controller_in_receiver);
    });

    let controller_read_write = thread::spawn(move || {
        stream_read_write(controller_info, controller_in_sender, controller_out_receiver);
    });

    message_interceptor.join().unwrap();
    ota_processor.join().unwrap();
    node_manager.join().unwrap();
    gateway_read_write.join().unwrap();
    controller_read_write.join().unwrap();
}