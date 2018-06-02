use channel::Sender;
use diesel;
use diesel::prelude::*;
use model::db::DbConn;
use rocket;
use rocket_contrib::Json;
use model::node::Node;
use model::node::nodes::dsl::*;

#[get("/")]
fn index() -> &'static str {
    "Available api's \n \
        GET /nodes \n \
        PUT /node <node> \n \
        POST /reboot_node/<node_id>"
}

#[get("/nodes")]
fn list(conn: DbConn) -> Json<Vec<Node>> {
    let existing_nodes = nodes
        .load::<Node>(&*conn).expect("error while loading existing nodes");
    Json(existing_nodes)
}

#[put("/node", format = "application/json", data = "<node>")]
fn update_node(node: Json<Node>, conn: DbConn) -> &'static str {
    diesel::update(nodes.find(node.node_id))
        .set((firmware_type.eq(node.firmware_type),
              firmware_version.eq(node.firmware_version),
              auto_update.eq(node.auto_update)))
        .execute(&*conn).unwrap();
    "OK"
}

#[post("/reboot_node/<node_id_param>")]
fn reboot_node(node_id_param: u8, sender: rocket::State<Sender<String>>) -> &'static str {
    sender.send(format!("{};255;3;0;13;0", node_id_param)).unwrap();
    "OK"
}
