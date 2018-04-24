use diesel::prelude::*;
use rocket_contrib::Json;
use schema::Node;
use pool::DbConn;
use schema::nodes::dsl::nodes;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/nodes")]
fn get_nodes(db_connection: DbConn) -> Json<Vec<Node>> {
    let existing_nodes = nodes
        .load::<Node>(&*db_connection).expect("error while loading existing nodes");
    Json(existing_nodes)
}
