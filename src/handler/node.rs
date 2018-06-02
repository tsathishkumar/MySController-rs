use diesel;
use diesel::prelude::*;
use channel::{Receiver, Sender};
use model::node::nodes::dsl;
use model::db;
use model::node::Node;

pub fn create_node(conn: &SqliteConnection, id: i32) -> usize {
    let new_node = Node {
        node_id: id,
        node_name: "New Node".to_owned(),
        firmware_type: 0,
        firmware_version: 0,
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
        .load::<Node>(conn).expect("error while loading existing nodes");
    match existing_nodes.iter()
        .map(|node| node.node_id())
        .max() {
        Some(node_id) => node_id + 1,
        None => 1
    }
}

pub fn handle_node_id_request(
    receiver: &Receiver<String>,
    sender: &Sender<String>,
    db_connection: db::DbConn,
) {
    loop {
        match receiver.recv() {
            Ok(command_message_request) => {
                let new_node_id = get_next_node_id(&db_connection);
                let new_id = format!("{}\n", new_node_id);
                let mut node_id_response = command_message_request.trim().split(";").collect::<Vec<&str>>();
                node_id_response[4] = "4";
                node_id_response[5] = &new_id;
                sender.send(node_id_response.join(";")).unwrap();
                create_node(&db_connection, new_node_id as i32);
            }
            _ => (),
        }
    }
}