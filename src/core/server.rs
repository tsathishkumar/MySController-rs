use std::thread;

use diesel::prelude::SqliteConnection;
use diesel::r2d2::{ConnectionManager, Pool};

use crate::channel;
use crate::channel::{Receiver, Sender};
use crate::model::sensor::Sensor;

use super::connection::*;
use super::interceptor;
use super::message::set::SetMessage;
use super::message_handler::{internal, presentation, set, stream};

pub fn start(
    gateway_info: StreamInfo,
    controller_info: Option<StreamInfo>,
    pool: Pool<ConnectionManager<SqliteConnection>>,
    gateway_out_sender: Sender<String>,
    gateway_out_receiver: Receiver<String>,
    in_set_sender: Sender<SetMessage>,
    set_message_receiver: Receiver<SetMessage>,
    new_sensor_sender: Sender<(String, Sensor)>,
) {
    let (gateway_sender, gateway_receiver) = channel::unbounded();
    let (stream_sender, stream_receiver) = channel::unbounded();
    let (internal_sender, internal_receiver) = channel::unbounded();
    let (presentation_sender, presentation_receiver) = channel::unbounded();
    let (set_sender, set_receiver) = channel::unbounded();

    let (controller_out_sender, controller_out_receiver) = channel::unbounded();

    let stream_response_sender = gateway_out_sender.clone();
    let internal_response_sender = gateway_out_sender.clone();
    let set_response_sender = gateway_out_sender.clone();
    let presentation_forward_sender = controller_out_sender.clone();
    let set_forward_sender = controller_out_sender.clone();
    let internal_forward_sender = controller_out_sender.clone();

    let message_interceptor = thread::spawn(move || {
        interceptor::intercept(
            &gateway_receiver,
            &stream_sender,
            &internal_sender,
            &presentation_sender,
            &set_sender,
            &controller_out_sender,
        );
    });

    let set_message_writer = set::handle_from_controller(set_message_receiver, set_response_sender);

    let set_message_reader = set::handle_from_gateway(set_receiver, in_set_sender, set_forward_sender);


    let connection = pool.get().unwrap();

    let stream_message_processor = thread::spawn(move || {
        stream::handle(&stream_receiver, &stream_response_sender, connection);
    });

    let connection = pool.get().unwrap();

    let internal_message_processor = thread::spawn(move || {
        internal::handle(
            &internal_receiver,
            &internal_response_sender,
            &internal_forward_sender,
            connection,
        );
    });

    let connection = pool.get().unwrap();

    let presentation_message_processor = thread::spawn(move || {
        presentation::handle(
            &presentation_receiver,
            &presentation_forward_sender,
            connection,
            new_sensor_sender,
        );
    });

    let gateway_read_write = thread::spawn(move || {
        stream_read_write(gateway_info, gateway_sender, gateway_out_receiver);
    });

    let controller_read_write = thread::spawn(move || {
        if controller_info.is_some() {
            stream_read_write(
                controller_info.unwrap(),
                gateway_out_sender,
                controller_out_receiver,
            );
        } else {
            loop {
                match controller_out_receiver.recv() {
                    _ => debug!("Ignoring messages to controller, as no controller is configured"),
                }
            }
        }
    });

    gateway_read_write.join().unwrap();
    controller_read_write.join().unwrap();
    message_interceptor.join().unwrap();
    set_message_writer.join().unwrap();
    set_message_reader.join().unwrap();
    stream_message_processor.join().unwrap();
    internal_message_processor.join().unwrap();
    presentation_message_processor.join().unwrap();
}
