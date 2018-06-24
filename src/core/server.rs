use super::connection::*;
use super::interceptor;
use super::message::set::SetMessage;
use super::message_handler::{internal, presentation, set, stream};
use channel;
use channel::{Receiver, Sender};
use diesel::prelude::SqliteConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use std::thread;

pub fn start(
    gateway_info: StreamInfo,
    controller_info: StreamInfo,
    pool: Pool<ConnectionManager<SqliteConnection>>,

    gateway_out_sender: Sender<String>,
    gateway_out_receiver: Receiver<String>,
    in_set_sender: Sender<SetMessage>,
    set_message_receiver: Receiver<SetMessage>,
) {
    let (gateway_sender, gateway_receiver) = channel::unbounded();
    let (stream_sender, stream_receiver) = channel::unbounded();
    let (internal_sender, internal_receiver) = channel::unbounded();
    let (presentation_sender, presentation_receiver) = channel::unbounded();

    let (controller_out_sender, controller_out_receiver) = channel::unbounded();

    let stream_response_sender = gateway_out_sender.clone();
    let internal_response_sender = gateway_out_sender.clone();
    let set_response_sender = gateway_out_sender.clone();
    let presentation_forward_sender = controller_out_sender.clone();

    let message_interceptor = thread::spawn(move || {
        interceptor::intercept(
            &gateway_receiver,
            &stream_sender,
            &internal_sender,
            &presentation_sender,
            &in_set_sender,
            &controller_out_sender,
        );
    });

    let set_message_processor = thread::spawn(move || {
        set::handle_from_controller(&set_message_receiver, &set_response_sender);
    });

    let connection = pool.get().unwrap();

    let stream_message_processor = thread::spawn(move || {
        stream::handle(&stream_receiver, &stream_response_sender, connection);
    });

    let connection = pool.get().unwrap();

    let internal_message_processor = thread::spawn(move || {
        internal::handle(&internal_receiver, &internal_response_sender, connection);
    });

    let connection = pool.get().unwrap();

    let presentation_message_processor = thread::spawn(move || {
        presentation::handle(
            &presentation_receiver,
            &presentation_forward_sender,
            connection,
        );
    });

    let gateway_read_write = thread::spawn(move || {
        stream_read_write(gateway_info, gateway_sender, gateway_out_receiver);
    });

    let controller_read_write = thread::spawn(move || {
        stream_read_write(controller_info, gateway_out_sender, controller_out_receiver);
    });

    gateway_read_write.join().unwrap();
    controller_read_write.join().unwrap();
    message_interceptor.join().unwrap();
    set_message_processor.join().unwrap();
    stream_message_processor.join().unwrap();
    internal_message_processor.join().unwrap();
    presentation_message_processor.join().unwrap();
}
