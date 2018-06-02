use channel::{Receiver, Sender};
use diesel::prelude::*;
use model::firmware::Firmware;
use model::firmware::firmwares::dsl::firmwares;
use super::message::{CommandMessage, CommandSubType, MessagePayloadType};
use model::db;
use model::node::Node;
use model::node::nodes::dsl::*;

pub fn process_ota(ota_receiver: &Receiver<CommandMessage>,
                   sender: &Sender<String>, db_connection: db::DbConn) {
    loop {
        match ota_receiver.recv() {
            Ok(command_message_request) => match command_message_request.sub_type {
                CommandSubType::StFirmwareConfigRequest => send_response(
                    sender, command_message_request.clone(), &db_connection),
                CommandSubType::StFirmwareRequest => send_response(
                    sender, command_message_request.clone(), &db_connection),
                _ => (),
            },
            _ => (),
        }
    }
}

fn send_response(serial_sender: &Sender<String>,
                 mut command_message: CommandMessage,
                 db_connection: &SqliteConnection) {
    let node = nodes.find(command_message.node_id as i32)
        .first::<Node>(db_connection)
        .optional().
        unwrap();
    match response_fw_type_version(command_message, node) {
        Some((_type, version)) => {
            match firmwares.find((_type as i32, version as i32)).first::<Firmware>(&*db_connection) {
                Ok(firmware) => {
                    command_message.to_response(&firmware);
                    let response = command_message.serialize();
                    serial_sender.send(response).unwrap();
                }
                Err(_message) => {
                    println!("no firmware found -- for type {} - version {}", _type, version);
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
        MessagePayloadType::FwRequest(request) => Some((request.firmware_type, request.firmware_version)),
        _ => None,
    }
}