use super::message;
use channel::{Receiver, Sender};

pub fn intercept(
    receiver: &Receiver<String>,
    ota_sender: &Sender<message::stream_message::StreamMessage>,
    node_sender: &Sender<String>,
    controller_sender: &Sender<String>,
) {
    let node_id_request: String = "255;255;3;0;3;0\n".to_string();
    loop {
        let request = match receiver.recv() {
            Ok(req) => req,
            Err(_e) => {
                info!("Error while trying to receive in interceptor {:?}", _e);
                break;
            }
        };

        if request == node_id_request {
            match node_sender.send(request) {
                Ok(_) => continue,
                Err(_) => break,
            }
        }
        let command_message_result = message::CommandMessage::new(&request);

        match command_message_result {
            Ok(command_message) => match command_message {
                message::CommandMessage::Stream(stream_message) => {
                    match ota_sender.send(stream_message) {
                        Ok(_) => (),
                        Err(error) => error!("Error while sending to ota_sender {:?}", error),
                    }
                }
                _ => match controller_sender.send(request) {
                    Ok(_) => (),
                    Err(error) => error!("Error while sending to controller {:?}", error),
                },
            },
            Err(message) => {
                error!(
                    "Error while parsing command message {:?}, simply forwarding to controller",
                    message
                );
                match controller_sender.send(request) {
                    Ok(_) => (),
                    Err(error) => error!("Error while sending to controller {:?}", error),
                }
            }
        }
    }
}
