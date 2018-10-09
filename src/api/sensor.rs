use actix_web::{AsyncResponder, FutureResponse, HttpRequest, HttpResponse, Json};
use crate::api::index::AppState;
use futures::future::Future;
use crate::handler::sensor::*;
use http::StatusCode;

pub fn list(req: HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    req.state()
        .db
        .send(ListSensors)
        .from_err()
        .and_then(|res| match res {
            Ok(msg) => Ok(HttpResponse::Ok().json(msg)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

pub fn delete(
    sensor_delete: Json<DeleteSensor>,
    req: HttpRequest<AppState>,
) -> FutureResponse<HttpResponse> {
    req.state()
        .db
        .send(DeleteSensor {
            node_id: sensor_delete.node_id,
            child_sensor_id: sensor_delete.child_sensor_id,
        })
        .from_err()
        .and_then(|res| match res {
            Ok(msg) => Ok(HttpResponse::build(StatusCode::from_u16(msg.status).unwrap()).json(msg)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

pub fn get_sensor(req: HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    let node_id_path_param = req.match_info().get("node_id").unwrap();
    let child_sensor_id_path_param = req.match_info().get("child_sensor_id").unwrap();
    let node_id = node_id_path_param.to_string().parse::<i32>().unwrap();
    let child_sensor_id = child_sensor_id_path_param
        .to_string()
        .parse::<i32>()
        .unwrap();
    req.state()
        .db
        .send(GetSensor {
            node_id,
            child_sensor_id,
        })
        .from_err()
        .and_then(|res| match res {
            Ok(msg) => Ok(HttpResponse::Ok().json(msg)),
            Err(()) => Ok(HttpResponse::build(StatusCode::from_u16(400).unwrap()).body("Sensor not present")),
        })
        .responder()
}
