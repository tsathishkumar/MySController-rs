use diesel;
use diesel::prelude::*;
use pool::DbConn;
use rocket;
use rocket_contrib::Json;
use schema::Node;
use schema::nodes::dsl::*;
use channel::Sender;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/nodes")]
fn get_nodes(conn: DbConn) -> Json<Vec<Node>> {
    let existing_nodes = nodes
        .load::<Node>(&*conn).expect("error while loading existing nodes");
    Json(existing_nodes)
}

#[put("/node", format = "application/json", data = "<node>")]
fn update_node(node: Json<Node>, conn: DbConn) {
    diesel::update(nodes.find(node.node_id))
        .set((firmware_type.eq(node.firmware_type),
              firmware_version.eq(node.firmware_version),
              auto_update.eq(node.auto_update)))
        .execute(&*conn).unwrap();
}

#[post("/reboot_node", format = "application/json", data = "<node>")]
fn reboot_node(node: Json<Node>, sender: rocket::State<Sender<String>>) {
    sender.send(format!("{};255;3;0;13;0", node.node_id)).unwrap();
}
