use diesel;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use r2d2::*;

use crate::channel::{Receiver, Sender};
use crate::core::message::stream::*;
use crate::model::firmware::Firmware;
use crate::model::firmware::firmwares::dsl::firmwares;
use crate::model::node::Node;
use crate::model::node::nodes::dsl::*;

pub fn handle(
    ota_receiver: &Receiver<StreamMessage>,
    sender: &Sender<String>,
    db_connection: PooledConnection<ConnectionManager<SqliteConnection>>,
) {
    loop {
        if let Ok(stream_request) = ota_receiver.recv() {
            send_response(sender, stream_request, &db_connection)
        }
    }
}

fn send_response(
    stream_response_sender: &Sender<String>,
    mut stream: StreamMessage,
    db_connection: &SqliteConnection,
) {
    if let Ok(node) = nodes
        .find(i32::from(stream.node_id))
        .first::<Node>(db_connection)
        .optional()
    {
        if let Some((_type, version)) = response_fw_type_version(stream, node, db_connection) {
            match firmwares
                .find((i32::from(_type), i32::from(version)))
                .first::<Firmware>(&*db_connection) {
                Ok(firmware) => {
                    debug!("Request {:?}", stream);
                    stream.response(&firmware);
                    debug!("Response {:?}", stream);
                    let response = stream.to_string();
                    match stream_response_sender.send(response) {
                        Ok(_) => (),
                        Err(_) => error!("Error sending to stream response sender"),
                    }
                }
                Err(_message) => {
                    warn!(
                        "no firmware found -- for type {} - version {}",
                        _type, version
                    );
                }
            }
        }
    }
}

fn response_fw_type_version(
    stream: StreamMessage,
    node: Option<Node>,
    connection: &SqliteConnection,
) -> Option<(u16, u16)> {
    match stream.payload {
        StreamPayload::FwConfigRequest(request) => {
            info!(
                "Firmware requested by node {} - type {} ,version {}",
                stream.node_id, request.firmware_type, request.firmware_version
            );

            match node {
                Some(_node) => {
                    match diesel::update(nodes.filter(node_id.eq(_node.node_id)))
                        .set((
                            firmware_type.eq(i32::from(request.firmware_type)),
                            firmware_version.eq(i32::from(request.firmware_version)),
                        ))
                        .execute(connection)
                        {
                            Ok(_) => (),
                            Err(_) => error!("Error while updating node with advertised firmware"),
                        }
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
