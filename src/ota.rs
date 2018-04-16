use firmware;
use message::{CommandMessage, CommandSubType};
use std::sync::mpsc;

pub fn process_ota(
    ota_receiver: &mpsc::Receiver<CommandMessage>,
    sender: &mpsc::Sender<String>,
) {
    let firmware_repo = firmware::FirmwareRepo::new();
    loop {
        match ota_receiver.recv() {
            Ok(command_message_request) => match command_message_request.sub_type {
                CommandSubType::StFirmwareConfigRequest => send_response(
                    sender,
                    command_message_request.clone(),
                    &firmware_repo,
                ),
                CommandSubType::StFirmwareRequest => send_response(
                    sender,
                    command_message_request.clone(),
                    &firmware_repo,
                ),
                _ => (),
            },
            _ => (),
        }
    }
}

fn send_response(
    serial_sender: &mpsc::Sender<String>,
    mut command_message: CommandMessage,
    _firmware_repo: &firmware::FirmwareRepo,
) {
    match command_message.fw_type_version() {
        Some((_type, version)) => {
            match _firmware_repo.get_firmware(_type, version) {
                Ok(firmware) => {
                    command_message.to_response(firmware);
                    let response = command_message.serialize();
                    println!("ota : {:?}", response);
                    serial_sender.send(response).unwrap();
                }
                Err(_message) => {
                    let firmware = _firmware_repo.get_firmware(1, 1).unwrap();
                    command_message.to_response(firmware);
                    let response = command_message.serialize();
                    println!("default ota : {:?}", response);
                    serial_sender.send(response).unwrap();
                }
            }
        }
        None => ()
    }
}
