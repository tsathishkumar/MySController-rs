use message;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;

pub fn intercept(
    stop_thread: Arc<Mutex<bool>>,
    receiver: &mpsc::Receiver<String>,
    ota_sender: &mpsc::Sender<message::CommandMessage>,
    node_sender: &mpsc::Sender<String>,
    controller_sender: &mpsc::Sender<String>,
) {
    let node_id_request: String = "255;255;3;0;3;0\n".to_string();
    loop {
        if *stop_thread.lock().unwrap() {
            break;
        }
        let request = match receiver.recv() {
            Ok(req) => req,
            Err(_) => panic!("Error while trying to receive in interceptor"),
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
