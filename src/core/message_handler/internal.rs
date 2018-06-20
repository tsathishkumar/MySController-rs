use channel::{Receiver, Sender};
use diesel;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use model::node::nodes::dsl;
use model::node::Node;
use r2d2::*;

const MIN_NODE_ID: u8 = 1;
const MAX_NODE_ID: u8 = 254;

pub fn handle(
    receiver: &Receiver<String>,
    sender: &Sender<String>,
    db_connection: PooledConnection<ConnectionManager<SqliteConnection>>,
) {
    loop {
        match receiver.recv() {
            Ok(command_message_request) => match get_next_node_id(&db_connection) {
                Some(new_node_id) => {
                    let new_id = format!("{}\n", new_node_id);
                    let mut node_id_response = command_message_request
                        .trim()
                        .split(";")
                        .collect::<Vec<&str>>();
                    node_id_response[4] = "4";
                    node_id_response[5] = &new_id;
                    match create_node(&db_connection, new_node_id as i32) {
                        Ok(_) => match sender.send(node_id_response.join(";")) {
                            Ok(_) => continue,
                            Err(_) => error!("Error while sending to node_handler"),
                        },
                        Err(_) => error!("Error while creating node with new id"),
                    }
                }
                None => error!("There is no free node id! All 254 id's are already reserved!"),
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
    };

    diesel::insert_into(dsl::nodes)
        .values(&new_node)
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
