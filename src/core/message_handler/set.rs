use channel::{Receiver, Sender};
use core::message::set::*;

pub fn handle_from_controller(set_message_receiver: &Receiver<SetMessage>, gateway_out_sender: &Sender<String>) {
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
