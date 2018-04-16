use diesel;
use diesel::prelude::*;
use schema::Node;
use schema::nodes::dsl::nodes;
use std::sync::mpsc;

pub fn create_node(conn: &SqliteConnection, id: i32) -> usize {
    let new_node = Node {
        node_id: id,
        firmware_type: 0,
        firmware_version: 0,
        auto_update: false,
    };

    diesel::insert_into(nodes)
        .values(&new_node)
        .execute(conn)
        .expect("Error saving new node")
}

pub fn get_next_node_id(conn: &SqliteConnection) -> u8 {
    let existing_nodes = nodes
        .load::<Node>(conn).expect("error while loading existing nodes");
    match existing_nodes.iter()
        .map(|node| node.node_id())
        .max() {
        Some(node_id) => node_id + 1,
        None => 1
    }
}

pub fn handle_node_id_request(
    receiver: &mpsc::Receiver<String>,
    sender: &mpsc::Sender<String>,
    db_connection: SqliteConnection,
) {
    loop {
        match receiver.recv() {
            Ok(command_message_request) => {
                let new_node_id = get_next_node_id(&db_connection);
                let new_id = format!("{}\n", new_node_id);
                let mut message_parts = command_message_request.trim().split(";").collect::<Vec<&str>>();
                message_parts[4] = "4";
                message_parts[5] = &new_id;
                sender.send(message_parts.join(";")).unwrap();
                create_node(&db_connection, new_node_id as i32);
            }
            _ => (),
        }
    }
}