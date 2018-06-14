use super::message::{CommandMessage, CommandSubType, MessagePayloadType};
use channel::{Receiver, Sender};
use diesel;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use model::firmware::firmwares::dsl::firmwares;
use model::firmware::Firmware;
use model::node::nodes::dsl::*;
use model::node::Node;
use r2d2::*;

pub fn process_ota(
    ota_receiver: &Receiver<CommandMessage>,
    sender: &Sender<String>,
    db_connection: PooledConnection<ConnectionManager<SqliteConnection>>,
) {
    loop {
        match ota_receiver.recv() {
            Ok(command_message_request) => match command_message_request.sub_type {
                CommandSubType::StFirmwareConfigRequest => {
                    send_response(sender, command_message_request.clone(), &db_connection)
                }
                CommandSubType::StFirmwareRequest => {
                    send_response(sender, command_message_request.clone(), &db_connection)
                }
                _ => (),
            },
            _ => (),
        }
    }
}

fn send_response(
    serial_sender: &Sender<String>,
    mut command_message: CommandMessage,
    db_connection: &SqliteConnection,
) {
    if let Ok(node) = nodes
        .find(command_message.node_id as i32)
        .first::<Node>(db_connection)
        .optional()
    {
        match response_fw_type_version(command_message, node, db_connection) {
            Some((_type, version)) => match firmwares
                .find((_type as i32, version as i32))
                .first::<Firmware>(&*db_connection)
            {
                Ok(firmware) => {
                    command_message.to_response(&firmware);
                    let response = command_message.serialize();
                    serial_sender.send(response).unwrap();
                }
                Err(_message) => {
                    println!(
                        "no firmware found -- for type {} - version {}",
                        _type, version
                    );
                }
            },
            None => (),
        }
    }
}

fn response_fw_type_version(
    command_message: CommandMessage,
    node: Option<Node>,
    connection: &SqliteConnection,
) -> Option<(u16, u16)> {
    match command_message.payload {
        MessagePayloadType::FwConfigRequest(request) => {
            println!(
                "Firmware requested by node {} - type {} ,version {}",
                command_message.node_id, request._type, request.version
            );

            match node {
                Some(_node) => {
                    diesel::update(nodes.filter(node_id.eq(_node.node_id)))
                        .set((
                            firmware_type.eq(request._type as i32),
                            firmware_version.eq(request.version as i32),
                        ))
                        .execute(connection).unwrap();
                    Some((
                        _node.desired_firmware_type as u16,
                        _node.desired_firmware_version as u16,
                    ))
                }
                None => None,
            }
        }
        MessagePayloadType::FwRequest(request) => {
            Some((request.firmware_type, request.firmware_version))
        }
        _ => None,
    }
}
