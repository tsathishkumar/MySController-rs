use diesel::prelude::*;
use firmware;
use message::{CommandMessage, CommandSubType};
use pool;
use schema::Node;
use schema::nodes::dsl::*;
use std::sync::mpsc;

pub fn process_ota(firmwares_directory: &String,
                   ota_receiver: &mpsc::Receiver<CommandMessage>,
                   sender: &mpsc::Sender<String>, db_connection: pool::DbConn) {
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

#[derive(Debug)]
pub enum OtaError {
    NodeNotRegistered,
}

fn send_response(serial_sender: &mpsc::Sender<String>,
                 mut command_message: CommandMessage,
                 _firmware_repo: &firmware::FirmwareRepo,
                 db_connection: &SqliteConnection) {
    //TODO: get the type and version from database and send the firmware for nodes instead of the requested type and version
//    nodes.find(command_message.node_id as i32)
//        .first::<Node>(db_connection)
//        .optional().map_err(OtaError::NodeNotRegistered);
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
                    println!("no firmware found -- for type {} - version {}", _type, version);
                    match _firmware_repo.get_firmware(0, 0) {
                        Ok(firmware) => {
                            command_message.to_response(firmware);
                            let response = command_message.serialize();
                            println!("default ota : {:?}", response);
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
