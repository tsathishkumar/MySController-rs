use message;
use std::sync::mpsc;

pub fn intercept(
    receiver: &mpsc::Receiver<String>,
    ota_sender: &mpsc::Sender<message::CommandMessage>,
    node_sender: &mpsc::Sender<String>,
    controller_sender: &mpsc::Sender<String>,
) {
    loop {
        let request = receiver.recv().unwrap();
        if request == String::from("255;255;3;0;3;0\n") {
            node_sender.send(request).unwrap();
            continue;
        }
        let command_message_result = message::CommandMessage::new(&request);

        match command_message_result {
            Ok(command_message) => match command_message.command {
                message::CommandType::STREAM => ota_sender.send(command_message).unwrap(),
                _ => match controller_sender.send(request) {
                    Ok(_) => (),
                    Err(error) => eprintln!("{:?}", error),
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
