use super::response::Msgs;
use actix::*;
use actix_web::*;
use diesel;
use diesel::prelude::*;
use diesel::result::DatabaseErrorKind;
use diesel::result::Error::DatabaseError;
use model::db::ConnDsl;
use model::node::Node;

#[derive(Serialize, Deserialize)]
pub struct NewNode {
    pub node_id: i32,
    pub node_name: String,
    pub firmware_type: i32,
    pub firmware_version: i32,
    pub auto_update: bool,
    pub scheduled: bool,
}

pub struct GetNode {
    pub node_id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteNode {
    pub node_id: i32,
}

impl Message for GetNode {
    type Result = Result<Node, ()>;
}

impl Message for DeleteNode {
    type Result = Result<Msgs, diesel::result::Error>;
}

impl Message for NewNode {
    type Result = Result<Msgs, diesel::result::Error>;
}

pub struct ListNodes;

impl Message for ListNodes {
    type Result = Result<Vec<Node>, Error>;
}

impl Handler<ListNodes> for ConnDsl {
    type Result = Result<Vec<Node>, Error>;

    fn handle(&mut self, _list_nodes: ListNodes, _: &mut Self::Context) -> Self::Result {
        use model::node::nodes::dsl::*;
        let conn = &self.0.get().map_err(error::ErrorInternalServerError)?;
        let existing_nodes = nodes
            .load::<Node>(conn)
            .map_err(error::ErrorInternalServerError)?;
        Ok(existing_nodes)
    }
}

#[derive(Serialize, Deserialize)]
pub struct NodeUpdate {
    pub node_id: i32,
    pub node_name: String,
    pub firmware_type: i32,
    pub firmware_version: i32,
    pub auto_update: bool,
    pub scheduled: bool,
}

impl Message for NodeUpdate {
    type Result = Result<Msgs, diesel::result::Error>;
}

impl Handler<NodeUpdate> for ConnDsl {
    type Result = Result<Msgs, diesel::result::Error>;

    fn handle(&mut self, node_update: NodeUpdate, _: &mut Self::Context) -> Self::Result {
        use model::node::nodes::dsl::*;
        match &self.0.get() {
            Ok(conn) => {
                let updated = diesel::update(nodes)
                    .filter(&node_id.eq(&node_update.node_id))
                    .set((
                        node_name.eq(node_update.node_name),
                        desired_firmware_type.eq(node_update.firmware_type),
                        desired_firmware_version.eq(node_update.firmware_version),
                        auto_update.eq(node_update.auto_update),
                        scheduled.eq(node_update.scheduled),
                    ))
                    .execute(conn);
                match updated {
                    Ok(1) => Ok(Msgs {
                        status: 200,
                        message: "update node success.".to_string(),
                    }),
                    Ok(_) => Ok(Msgs {
                        status: 400,
                        message: "update failed. node id is not present".to_string(),
                    }),
                    Err(e) => Err(e),
                }
            }
            Err(_) => Ok(Msgs {
                status: 500,
                message: "update failed. internal server error".to_string(),
            }),
        }
    }
}

impl Handler<DeleteNode> for ConnDsl {
    type Result = Result<Msgs, diesel::result::Error>;

    fn handle(&mut self, delete_node: DeleteNode, _: &mut Self::Context) -> Self::Result {
        use model::node::nodes::dsl::*;
        match &self.0.get() {
            Ok(conn) => {
                let updated = diesel::delete(nodes)
                    .filter(&node_id.eq(&delete_node.node_id))
                    .execute(conn);
                match updated {
                    Ok(1) => Ok(Msgs {
                        status: 200,
                        message: "deleted node.".to_string(),
                    }),
                    Ok(_) => Ok(Msgs {
                        status: 400,
                        message: "delete failed. node id is not present".to_string(),
                    }),
                    Err(e) => Err(e),
                }
            }
            Err(_) => Ok(Msgs {
                status: 500,
                message: "delete failed. internal server error".to_string(),
            }),
        }
    }
}

impl Handler<NewNode> for ConnDsl {
    type Result = Result<Msgs, diesel::result::Error>;

    fn handle(&mut self, new_node: NewNode, _: &mut Self::Context) -> Self::Result {
        use model::node::nodes::dsl::*;
        match &self.0.get() {
            Ok(conn) => {
                let new_node = Node {
                    node_id: new_node.node_id,
                    node_name: new_node.node_name,
                    firmware_type: 0,
                    firmware_version: 0,
                    desired_firmware_type: new_node.firmware_type,
                    desired_firmware_version: new_node.firmware_version,
                    auto_update: new_node.auto_update,
                    scheduled: new_node.scheduled,
                };

                let result = diesel::insert_into(nodes).values(&new_node).execute(conn);

                match result {
                    Ok(_) => Ok(Msgs {
                        status: 200,
                        message: "create node success.".to_string(),
                    }),
                    Err(DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => Ok(Msgs {
                        status: 400,
                        message: "node_id already present.".to_string(),
                    }),
                    Err(e) => Err(e),
                }
            }
            Err(_) => Ok(Msgs {
                status: 500,
                message: "internal server error.".to_string(),
            }),
        }
    }
}

impl Handler<GetNode> for ConnDsl {
    type Result = Result<Node, ()>;

    fn handle(&mut self, node: GetNode, _: &mut Self::Context) -> Self::Result {
        use model::node::nodes::dsl::*;

        let conn = &self.0.get().map_err(|_| ())?;

        let existing_node = nodes
            .find(node.node_id)
            .first::<Node>(conn)
            .optional()
            .unwrap();
        match existing_node {
            Some(v) => Ok(v),
            None => return Err(()),
        }
    }
}
