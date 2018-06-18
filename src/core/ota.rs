use super::message::stream_message::*;
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
    ota_receiver: &Receiver<StreamMessage>,
    sender: &Sender<String>,
    db_connection: PooledConnection<ConnectionManager<SqliteConnection>>,
) {
    loop {
        match ota_receiver.recv() {
            Ok(stream_request) => send_response(sender, stream_request.clone(), &db_connection),
            _ => (),
        }
    }
}

fn send_response(
    serial_sender: &Sender<String>,
    mut stream_message: StreamMessage,
    db_connection: &SqliteConnection,
) {
    if let Ok(node) = nodes
        .find(stream_message.node_id as i32)
        .first::<Node>(db_connection)
        .optional()
    {
        match response_fw_type_version(stream_message, node, db_connection) {
            Some((_type, version)) => match firmwares
                .find((_type as i32, version as i32))
                .first::<Firmware>(&*db_connection)
            {
                Ok(firmware) => {
                    debug!("Request {:?}", stream_message);
                    stream_message.to_response(&firmware);
                    debug!("Response {:?}", stream_message);
                    let response = stream_message.to_string();
                    serial_sender.send(response).unwrap();
                }
                Err(_message) => {
                    warn!(
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
    stream_message: StreamMessage,
    node: Option<Node>,
    connection: &SqliteConnection,
) -> Option<(u16, u16)> {
    match stream_message.payload {
        StreamPayload::FwConfigRequest(request) => {
            info!(
                "Firmware requested by node {} - type {} ,version {}",
                stream_message.node_id, request.firmware_type, request.firmware_version
            );

            match node {
                Some(_node) => {
                    diesel::update(nodes.filter(node_id.eq(_node.node_id)))
                        .set((
                            firmware_type.eq(request.firmware_type as i32),
                            firmware_version.eq(request.firmware_version as i32),
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
        StreamPayload::FwRequest(request) => {
            Some((request.firmware_type, request.firmware_version))
        }
        _ => None,
    }
}
