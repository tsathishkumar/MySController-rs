use enum_primitive;
use message;
use std::sync::mpsc;

pub fn intercept(
    receiver: &mpsc::Receiver<String>,
    ota_sender: &mpsc::Sender<message::CommandMessage>,
    controller_sender: &mpsc::Sender<String>,
) {
    loop {
        let request = receiver.recv().unwrap();
        let command_message_result = message::CommandMessage::new(&request);

        match command_message_result {
            Ok(command_message) => {
                println!("command type is {:?}", command_message);
                match command_message.command {
                    message::CommandType::STREAM => {
                        ota_sender.send(command_message).unwrap()
                    }
                    _ => controller_sender.send(request).unwrap(),
                }
            }
            Err(message) => {
                println!("Error while parsing command message {:?}, simply forwarding to controller", message);
                controller_sender.send(request).unwrap();
            }
        }
    }
}
