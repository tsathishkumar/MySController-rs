use actix_web::{AsyncResponder, FutureResponse, HttpRequest, HttpResponse, Json};
use api::index::AppState;
use futures::future::Future;
use handler::node::*;
use http::StatusCode;

pub fn list(req: HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    req.state()
        .db
        .send(ListNodes)
        .from_err()
        .and_then(|res| match res {
            Ok(msg) => Ok(HttpResponse::Ok().json(msg)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

pub fn create(
    node_update: Json<NewNode>,
    req: HttpRequest<AppState>,
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
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

pub fn update(
    node_update: Json<NodeUpdate>,
    req: HttpRequest<AppState>,
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
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

pub fn delete(
    node_delete: Json<DeleteNode>,
    req: HttpRequest<AppState>,
) -> FutureResponse<HttpResponse> {
    req.state()
        .db
        .send(DeleteNode {
            node_id: node_delete.node_id,
        })
        .from_err()
        .and_then(|res| match res {
            Ok(msg) => Ok(HttpResponse::build(StatusCode::from_u16(msg.status).unwrap()).json(msg)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

pub fn get_node(req: HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    let node_id_path_param = req.match_info().get("node_id").unwrap();
    let node_id = node_id_path_param.to_string().parse::<i32>().unwrap();
    req.state()
        .db
        .send(GetNode { node_id: node_id })
        .from_err()
        .and_then(|res| match res {
            Ok(msg) => Ok(HttpResponse::Ok().json(msg)),
            Err(()) => Ok(
                HttpResponse::build(StatusCode::from_u16(400).unwrap()).body("Node not present")
            ),
        })
        .responder()
}

pub fn reboot_node(req: HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
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
            Err(_) => Ok(
                HttpResponse::build(StatusCode::from_u16(400).unwrap()).body("Node not present")
            ),
        })
        .responder()
}
