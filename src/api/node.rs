use actix_web::{AsyncResponder, FutureResponse, HttpRequest, HttpResponse, Json};
use crate::api::index::AppState;
use crate::handler::node::*;
use futures::future::Future;
use http::StatusCode;

pub fn list(req: &HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    req.state()
        .db
        .send(ListNodes)
        .from_err()
        .and_then(|res| match res {
            Ok(msg) => Ok(HttpResponse::Ok().json(msg)),
            Err(e) => {
                error!("Error while getting nodes list {:?}", e);
                Ok(HttpResponse::InternalServerError().into())
            }
        })
        .responder()
}

pub fn create(
    (req, node_update): (HttpRequest<AppState>, Json<NewNode>),
) -> FutureResponse<HttpResponse> {
    req.state()
        .db
        .send(NewNode {
            node_id: node_update.node_id,
            node_name: node_update.node_name.clone(),
            firmware_type: node_update.firmware_type,
            firmware_version: node_update.firmware_version,
            auto_update: node_update.auto_update,
            scheduled: node_update.scheduled,
        })
        .from_err()
        .and_then(|res| match res {
            Ok(msg) => Ok(HttpResponse::build(StatusCode::from_u16(msg.status).unwrap()).json(msg)),
            Err(e) => {
                error!("Error while creating node {:?}", e);
                Ok(HttpResponse::InternalServerError().into())
            }
        })
        .responder()
}

pub fn update(
    (req, node_update): (HttpRequest<AppState>, Json<NodeUpdate>),
) -> FutureResponse<HttpResponse> {
    req.state()
        .db
        .send(NodeUpdate {
            node_id: node_update.node_id,
            node_name: node_update.node_name.clone(),
            firmware_type: node_update.firmware_type,
            firmware_version: node_update.firmware_version,
            auto_update: node_update.auto_update,
            scheduled: node_update.scheduled,
        })
        .from_err()
        .and_then(|res| match res {
            Ok(msg) => Ok(HttpResponse::build(StatusCode::from_u16(msg.status).unwrap()).json(msg)),
            Err(e) => {
                error!("Error while getting nodes {:?}", e);
                Ok(HttpResponse::InternalServerError().into())
            }
        })
        .responder()
}

pub fn delete(
    (req, node_delete): (HttpRequest<AppState>, Json<DeleteNode>),
) -> FutureResponse<HttpResponse> {
    req.state()
        .db
        .send(DeleteNode {
            node_id: node_delete.node_id,
        })
        .from_err()
        .and_then(|res| match res {
            Ok(msg) => Ok(HttpResponse::build(StatusCode::from_u16(msg.status).unwrap()).json(msg)),
            Err(e) => {
                error!("Error while deleting node {:?}", e);
                Ok(HttpResponse::InternalServerError().into())
            }
        })
        .responder()
}

pub fn get_node(req: &HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    let node_id_path_param = req.match_info().get("node_id").unwrap();
    let node_id = node_id_path_param.to_string().parse::<i32>().unwrap();
    req.state()
        .db
        .send(GetNode { node_id: node_id })
        .from_err()
        .and_then(|res| match res {
            Ok(msg) => Ok(HttpResponse::Ok().json(msg)),
            Err(e) => {
                error!("Error while getting node {:?}", e);
                Ok(
                    HttpResponse::build(StatusCode::from_u16(400).unwrap())
                        .body("Node not present"),
                )
            }
        })
        .responder()
}

pub fn reboot_node(req: &HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    let node_id_path_param = req.match_info().get("node_id").unwrap();
    let reset_sender = req.state().reset_sender.clone();
    let node_id = node_id_path_param.to_string().parse::<i32>().unwrap();
    req.state()
        .db
        .send(GetNode { node_id: node_id })
        .from_err()
        .and_then(move |res| match res {
            Ok(_node) => {
                reset_sender
                    .send(format!("{};255;3;0;13;0", node_id))
                    .unwrap();
                Ok(HttpResponse::Ok().body("Sent reboot request to node"))
            }
            Err(e) => {
                error!("Error while rebooting node {:?}", e);
                Ok(
                    HttpResponse::build(StatusCode::from_u16(400).unwrap())
                        .body("Node not present"),
                )
            }
        })
        .responder()
}
