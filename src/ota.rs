use message::{CommandMessage, MessagePayloadType};

use std::sync::mpsc;

pub fn process_ota(ota_receiver: &mpsc::Receiver<CommandMessage>) {
  loop {
    let mut command_message = ota_receiver.recv().unwrap();
    // message::decode(ota_request);
    command_message = command_message.clone();
    command_message.to_response();
    match command_message.payload {
      MessagePayloadType::StreamPayload(payload) => println!("ota request payload {:?}", &payload),
      _ => (),
    }
  }
}