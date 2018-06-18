use channel::{Receiver, Sender};
use diesel;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use model::node::nodes::dsl;
use model::node::Node;
use r2d2::*;

pub fn create_node(conn: &SqliteConnection, id: i32) -> usize {
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
        .expect("Error saving new node")
}

pub fn get_next_node_id(conn: &SqliteConnection) -> u8 {
    let existing_nodes = dsl::nodes
        .load::<Node>(conn)
        .expect("error while loading existing nodes");
    match existing_nodes.iter().map(|node| node.node_id()).max() {
        Some(node_id) => node_id + 1,
        None => 1,
    }
}

pub fn handle_node_id_request(
    receiver: &Receiver<String>,
    sender: &Sender<String>,
    db_connection: PooledConnection<ConnectionManager<SqliteConnection>>,
) {
    loop {
        match receiver.recv() {
            Ok(command_message_request) => {
                let new_node_id = get_next_node_id(&db_connection);
                let new_id = format!("{}\n", new_node_id);
                let mut node_id_response = command_message_request
                    .trim()
                    .split(";")
                    .collect::<Vec<&str>>();
                node_id_response[4] = "4";
                node_id_response[5] = &new_id;
                create_node(&db_connection, new_node_id as i32);
                match sender.send(node_id_response.join(";")) {
                    Ok(_) => continue,
                    Err(_) => error!("Error while sending to node_handler"),
                }
            }
            _ => (),
        }
    }
}
