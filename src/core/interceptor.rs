use super::message;
use channel::{Receiver, Sender};

pub fn intercept(
    receiver: &Receiver<String>,
    ota_sender: &Sender<message::CommandMessage>,
    node_sender: &Sender<String>,
    controller_sender: &Sender<String>,
) {
    let node_id_request: String = "255;255;3;0;3;0\n".to_string();
    loop {
        let request = match receiver.recv() {
            Ok(req) => req,
            Err(_e) => {
                println!("Error while trying to receive in interceptor {:?}", _e);
                break
            },
        };

        if request == node_id_request {
            node_sender.send(request).unwrap();
            continue;
        }
        let command_message_result = message::CommandMessage::new(&request);

        match command_message_result {
            Ok(command_message) => match command_message.command {
                message::CommandType::STREAM => ota_sender.send(command_message).unwrap(),
                _ => match controller_sender.send(request) {
                    Ok(_) => (),
                    Err(error) => eprintln!("Error while sending to controller {:?}", error),
                },
            },
            Err(message) => {
                eprintln!(
                    "Error while parsing command message {:?}, simply forwarding to controller",
                    message
                );
                controller_sender.send(request).unwrap();
            }
        }
    }
}
