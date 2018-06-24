use super::message::{presentation::*, set::*, stream::*, CommandMessage};
use channel::{Receiver, Sender};

pub fn intercept(
    receiver: &Receiver<String>,
    stream_sender: &Sender<StreamMessage>,
    node_sender: &Sender<String>,
    presentation_sender: &Sender<PresentationMessage>,
    set_sender: &Sender<SetMessage>,
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
        let command_message_result = CommandMessage::new(&request);

        match command_message_result {
            Ok(command_message) => match command_message {
                CommandMessage::Stream(stream_message) => {
                    match stream_sender.send(stream_message) {
                        Ok(_) => (),
                        Err(error) => error!("Error while sending to stream_sender {:?}", error),
                    }
                }
                CommandMessage::Presentation(presentation_message) => match presentation_sender
                    .send(presentation_message)
                {
                    Ok(_) => (),
                    Err(error) => error!("Error while sending to presentation_sender {:?}", error),
                },
                CommandMessage::Set(set_message) => match set_sender.send(set_message) {
                    Ok(_) => (),
                    Err(error) => error!("Error while sending to set_sender {:?}", error),
                },
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
