use channel::{Receiver, Sender};
use diesel::prelude::*;
use firmware;
use message::{CommandMessage, CommandSubType, MessagePayloadType};
use pool;
use schema::Node;
use schema::nodes::dsl::*;

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
    match response_fw_type_version(command_message, node) {
        Some((_type, version)) => {
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
                        Err(_) => println!("no default firmware found with type 0 and version 0")
                    }
                }
            }
        }
        None => ()
    }
}

fn response_fw_type_version(command_message: CommandMessage, node: Option<Node>) -> Option<(u16, u16)> {
    match command_message.payload {
        MessagePayloadType::FwConfigRequest(_request) => {
            println!("Firmware requested by node {} - type {} ,version {}", command_message.node_id, _request._type, _request.version);
            match node {
                Some(_node) => Some((_node.firmware_type as u16, _node.firmware_version as u16)),
                None => None
            }
        }
        MessagePayloadType::FwRequest(request) => Some((request._type, request.version)),
        _ => None,
    }
}