use super::connection::*;
use super::interceptor;
use super::ota;
use channel;
use channel::{Receiver, Sender};
use handler::node;
use model::db;
use std::thread;

pub fn start(
    gateway_info: StreamInfo,
    controller_info: StreamInfo,
    pool: db::SqlitePool,
    controller_in_sender: Sender<String>,
    controller_in_receiver: Receiver<String>,
) {
    let (gateway_sender, gateway_receiver) = channel::unbounded();
    let (ota_sender, ota_receiver) = channel::unbounded();

    let (controller_out_sender, controller_out_receiver) = channel::unbounded();
    let (node_manager_sender, node_manager_in) = channel::unbounded();
    let ota_fw_sender = controller_in_sender.clone();
    let node_manager_out = controller_in_sender.clone();

    let message_interceptor = thread::spawn(move || {
        interceptor::intercept(
            &gateway_receiver,
            &ota_sender,
            &node_manager_sender,
            &controller_out_sender,
        );
    });

    let connection = db::DbConn(pool.get().unwrap());

    let ota_processor = thread::spawn(move || {
        ota::process_ota(&ota_receiver, &ota_fw_sender, connection);
    });

    let connection = db::DbConn(pool.get().unwrap());

    let node_manager = thread::spawn(move || {
        node::handle_node_id_request(&node_manager_in, &node_manager_out, connection);
    });

    let gateway_read_write = thread::spawn(move || {
        stream_read_write(gateway_info, gateway_sender, controller_in_receiver);
    });

    let controller_read_write = thread::spawn(move || {
        stream_read_write(
            controller_info,
            controller_in_sender,
            controller_out_receiver,
        );
    });

    message_interceptor.join().unwrap();
    ota_processor.join().unwrap();
    node_manager.join().unwrap();
    gateway_read_write.join().unwrap();
    controller_read_write.join().unwrap();
}
