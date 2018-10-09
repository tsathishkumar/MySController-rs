use crate::channel::{Receiver, Sender};
use crate::core::message::set::*;

pub fn handle_from_controller(
    set_message_receiver: &Receiver<SetMessage>,
    gateway_out_sender: &Sender<String>,
) {
    loop {
        match set_message_receiver.recv() {
            Ok(set_message) => match gateway_out_sender.send(set_message.to_string()) {
                Ok(_) => (),
                Err(e) => error!("Error while sending set message to gateway {:?}", e),
            },
            _ => (),
        }
    }
}

pub fn handle_from_gateway(
    receiver: &Receiver<SetMessage>,
    property_notify_sender: &Sender<SetMessage>,
    controller_sender: &Sender<String>,
) {
    loop {
        match receiver.recv() {
            Ok(set_message) => {
                match controller_sender.send(set_message.to_string()) {
                    Ok(_) => (),
                    Err(error) => error!("Error while sending to controller_sender {:?}", error),
                };
                match property_notify_sender.send(set_message) {
                    Ok(_) => (),
                    Err(error) => {
                        error!("Error while sending to property_notify_sender {:?}", error)
                    }
                };
            }
            _ => (),
        }
    }
}
