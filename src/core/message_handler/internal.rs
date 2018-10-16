use crate::channel::{Receiver, Sender};
use crate::core::message::internal::*;
use diesel;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use crate::model::node::nodes::dsl;
use crate::model::node::Node;
use r2d2::*;

const MIN_NODE_ID: u8 = 1;
const MAX_NODE_ID: u8 = 254;

pub fn handle(
    receiver: &Receiver<InternalMessage>,
    response_sender: &Sender<String>,
    controller_forward_sender: &Sender<String>,
    db_connection: PooledConnection<ConnectionManager<SqliteConnection>>,
) {
    loop {
        match receiver.recv() {
            Ok(internal_message_request) => match internal_message_request {
                InternalMessage {node_id: 255, child_sensor_id: 255, ack: 0, sub_type: InternalType::IdRequest, ref payload } if payload == "0"
                => match get_next_node_id(&db_connection) {
                    Some(new_node_id) => {
                        let mut node_id_response = internal_message_request.clone();
                        node_id_response.sub_type = InternalType::IdResponse;
                        node_id_response.payload = new_node_id.to_string();
                        match create_node(&db_connection, new_node_id as i32) {
                            Ok(_) => match response_sender.send(node_id_response.to_string()) {
                                Ok(_) => continue,
                                Err(_) => error!("Error while sending to node_handler"),
                            },
                            Err(_) => error!("Error while creating node with new id"),
                        }
                    },
                    None => error!("There is no free node id! All 254 id's are already reserved!"),
                },
                InternalMessage {node_id, child_sensor_id: 255, ack, sub_type: InternalType::DiscoverResponse, ref payload } => {
                    let parent_node_id = payload.parse::<u8>().unwrap();
                    match update_network_topology(&db_connection, node_id as i32, parent_node_id as i32) {
                        Ok(_) => info!("Updated network topology"),
                        Err(e) => error!("Update network topology failed {:?}", e),
                    }
                },
                _ => (),
            },
            Ok(internal_message_request) => {
                match controller_forward_sender.send(internal_message_request.to_string()) {
                    Ok(_) => (),
                    Err(error) => error!("Error while forwarding internal message to controller {:?}", error),
                }
                ()
            },
            _ => (),
        }
    }
}

pub fn create_node(conn: &SqliteConnection, id: i32) -> Result<usize, diesel::result::Error> {
    let new_node = Node {
        node_id: id,
        node_name: "New Node".to_owned(),
        firmware_type: 0,
        firmware_version: 0,
        desired_firmware_type: 0,
        desired_firmware_version: 0,
        auto_update: false,
        scheduled: false,
        parent_node_id: 0,
    };

    diesel::insert_into(dsl::nodes)
        .values(&new_node)
        .execute(conn)
}

pub fn update_network_topology(conn: &SqliteConnection, _node_id: i32, _parent_node_id: i32) -> Result<usize, diesel::result::Error> {
    use crate::model::node::nodes::dsl::*;
    diesel::update(nodes)
        .filter(node_id.eq(_node_id))
        .set((parent_node_id.eq(_parent_node_id)))
        .execute(conn) 
}

pub fn get_next_node_id(conn: &SqliteConnection) -> Option<u8> {
    let existing_nodes = dsl::nodes
        .load::<Node>(conn)
        .expect("error while loading existing nodes");
    let used_node_ids: Vec<u8> = existing_nodes.iter().map(|node| node.node_id()).collect();
    for node_id in MIN_NODE_ID..=MAX_NODE_ID {
        if used_node_ids.contains(&node_id) {
            continue;
        }
        return Some(node_id);
    }
    None
}
