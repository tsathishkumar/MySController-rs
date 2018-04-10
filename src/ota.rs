use message;
use ihex::record::Record;
use ihex::writer;

use std::sync::mpsc;

pub fn process_ota(ota_receiver: &mpsc::Receiver<message::CommandMessage>) {
  loop {
    let command_message = ota_receiver.recv().unwrap();
    // message::decode(ota_request);
    println!("ota request {:?}", &command_message);
  }
}
