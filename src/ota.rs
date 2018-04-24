use diesel::prelude::*;
use firmware;
use message::{CommandMessage, CommandSubType};
use pool;
use schema::Node;
use schema::nodes::dsl::*;
use channel::{Receiver, Sender};

pub fn process_ota(firmwares_directory: &String,
                   ota_receiver: &Receiver<CommandMessage>,
                   sender: &Sender<String>, db_connection: pool::DbConn) {
    let firmware_repo = firmware::FirmwareRepo::new(firmwares_directory);
    loop {
        match ota_receiver.recv() {
            Ok(command_message_request) => match command_message_request.sub_type {
                CommandSubType::StFirmwareConfigRequest => send_response(
                    sender,
                    command_message_request.clone(),
                    &firmware_repo, &db_connection,
                ),
                CommandSubType::StFirmwareRequest => send_response(
                    sender,
                    command_message_request.clone(),
                    &firmware_repo, &db_connection,
                ),
                _ => (),
            },
            _ => (),
        }
    }
}

fn send_response(serial_sender: &Sender<String>,
                 mut command_message: CommandMessage,
                 _firmware_repo: &firmware::FirmwareRepo,
                 db_connection: &SqliteConnection) {
    let node = nodes.find(command_message.node_id as i32)
        .first::<Node>(db_connection)
        .optional().
        unwrap();
    match command_message.fw_type_version() {
        Some((requested_type, requested_version)) => {
            println!("Firmware requested by node {} - type {} ,version {}", command_message.node_id, requested_type, requested_version);
            match node.map(|n| (n.firmware_type as u16, n.firmware_version as u16)) {
                Some((_type, version)) => {
                    println!("Firmware assigned for node {} is - type {} ,version {}", command_message.node_id, _type, version);
                    match _firmware_repo.get_firmware(_type, version) {
                        Ok(firmware) => {
                            command_message.to_response(firmware);
                            let response = command_message.serialize();
                            serial_sender.send(response).unwrap();
                        }
                        Err(_message) => {
                            println!("no firmware found -- for type {} - version {}, trying to send default", _type, version);
                            match _firmware_repo.get_firmware(0, 0) {
                                Ok(firmware) => {
                                    command_message.to_response(firmware);
                                    let response = command_message.serialize();
                                    serial_sender.send(response).unwrap();
                                }
                                Err(_) => println!("no default firmware found")
                            }
                        }
                    }
                }
                None => ()
            }
        }
        None => ()
    }
}
