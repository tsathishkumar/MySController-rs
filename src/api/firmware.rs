use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use actix_web::{
    AsyncResponder, dev, error, Error, FutureResponse, HttpMessage, HttpRequest, HttpResponse,
    multipart, Query,
};
use futures::{Future, Stream};
use futures::future;
use http::StatusCode;

use crate::api::index::AppState;
use crate::handler::firmware::*;
use crate::handler::response::Msgs;
use crate::model::firmware::Firmware;

pub fn upload_form(_req: &HttpRequest<AppState>) -> Result<HttpResponse, error::Error> {
    let html = r#"<html>
        <head><title>Upload Test</title></head>
        <body>
            <form action="/firmwares" method="post" enctype="multipart/form-data">
                Name: <input type="text" name="firmware_name"/><br>
                Type: <input type="text" name="firmware_type"/><br>
                Version: <input type="text" name="firmware_version"/><br>
                <input type="file" name="firmware_file"/><br>
                <input type="submit" value="Submit"/>
            </form>
        </body>
    </html>"#;

    Ok(HttpResponse::Ok().body(html))
}

pub fn list(req: &HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    req.state()
        .db
        .send(ListFirmwares)
        .from_err()
        .and_then(|res| match res {
            Ok(msg) => Ok(HttpResponse::Ok().json(msg)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

pub fn delete(req: &HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    let firmware_type = match req.match_info().get("firmware_type") {
        Some(firmware_type) => match firmware_type.parse::<i32>() {
            Ok(value) => value,
            Err(_) => {
                return invalid_request("firmware_type should be a number with max value of 255");
            }
        },
        None => return invalid_request("firmware_type path param is missing"),
    };
    let firmware_version = match req.match_info().get("firmware_version") {
        Some(firmware_type) => match firmware_type.parse::<i32>() {
            Ok(value) => value,
            Err(_) => {
                return invalid_request("firmware_version should be a number with max value of 255");
            }
        },
        None => return invalid_request("firmware_version path param is missing"),
    };
    req.state()
        .db
        .send(DeleteFirmware {
            firmware_type,
            firmware_version,
        })
        .from_err()
        .and_then(|res| match res {
            Ok(msg) => Ok(HttpResponse::build(StatusCode::from_u16(msg.status).unwrap()).json(msg)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

pub fn create(
    query: Query<HashMap<String, String>>,
    req: HttpRequest<AppState>,
) -> FutureResponse<HttpResponse> {
    create_or_update(query, req, false)
}

pub fn update(
    query: Query<HashMap<String, String>>,
    req: HttpRequest<AppState>,
) -> FutureResponse<HttpResponse> {
    create_or_update(query, req, true)
}

pub fn create_or_update(
    query: Query<HashMap<String, String>>,
    req: HttpRequest<AppState>,
    update: bool,
) -> FutureResponse<HttpResponse> {
    let firmware_name = match query.get("firmware_name") {
        Some(firmware_name) => firmware_name.to_owned(),
        None => return invalid_request("firmware name is not present"),
    };
    let firmware_type = match req.match_info().get("firmware_type") {
        Some(firmware_type) => match firmware_type.parse::<u8>() {
            Ok(value) => value,
            Err(_) => {
                return invalid_request("firmware_type should be a number with max value of 255");
            }
        },
        None => return invalid_request("firmware_type path param is missing"),
    };
    let firmware_version = match req.match_info().get("firmware_version") {
        Some(firmware_type) => match firmware_type.parse::<u8>() {
            Ok(value) => value,
            Err(_) => {
                return invalid_request("firmware_version should be a number with max value of 255");
            }
        },
        None => return invalid_request("firmware_version path param is missing"),
    };
    let req_clone = req.clone();
    Box::new(
        req_clone
            .multipart()
            .map_err(error::ErrorInternalServerError)
            .map(handle_multipart_item)
            .flatten()
            .collect()
            .and_then(move |file_paths| match file_paths.get(0) {
                Some(file_path) => {
                    let firmware = get_firmware(
                        file_path.to_owned(),
                        firmware_type,
                        firmware_version,
                        firmware_name,
                    );
                    match fs::remove_file(file_path) {
                        Ok(_) => info!("Cleared temp firmware file"),
                        Err(e) => info!("Error in clearing temp firmware file {:?}", e),
                    };
                    match firmware {
                        Ok(firmware) => req
                            .state()
                            .db
                            .send(if update {
                                CreateOrUpdate::Update(firmware)
                            } else {
                                CreateOrUpdate::Create(firmware)
                            })
                            .from_err()
                            .and_then(|res| match res {
                                Ok(msg) => Ok(HttpResponse::build(
                                    StatusCode::from_u16(msg.status).unwrap(),
                                )
                                    .json(msg)),
                                Err(e) => {
                                    Ok(HttpResponse::build(StatusCode::from_u16(e.status).unwrap())
                                        .json(e))
                                }
                            })
                            .responder(),

                        Err(msg) => Box::new(future::result(Ok(HttpResponse::build(
                            StatusCode::from_u16(msg.status).unwrap(),
                        )
                            .json(msg)))),
                    }
                }
                None => Box::new(future::result(Ok(
                    HttpResponse::InternalServerError().into()
                ))),
            }),
    )
}

fn invalid_request(msg: &str) -> FutureResponse<HttpResponse> {
    Box::new(future::result(Ok(HttpResponse::build(
        StatusCode::from_u16(400).unwrap(),
    )
        .json(msg))))
}

fn get_firmware(
    file_name: String,
    firmware_type: u8,
    firmware_version: u8,
    firmware_name: String,
) -> Result<NewFirmware, Msgs> {
    match Firmware::prepare_fw(
        i32::from(firmware_type),
        i32::from(firmware_version),
        firmware_name,
        &PathBuf::from(file_name),
    ) {
        Some(firmware) => Ok(NewFirmware::build(
            firmware.firmware_type,
            firmware.firmware_version,
            firmware.name,
            firmware.data,
        )),
        _ => Err(Msgs {
            status: 400,
            message: "Error uploading firmware, Missing file".to_string(),
        }),
    }
}

fn handle_multipart_item(
    item: actix_web::multipart::MultipartItem<dev::Payload>,
) -> Box<dyn Stream<Item=String, Error=Error>> {
    match item {
        multipart::MultipartItem::Field(field) => Box::new(save_file(field).into_stream()),
        multipart::MultipartItem::Nested(mp) => Box::new(
            mp.map_err(error::ErrorInternalServerError)
                .map(handle_multipart_item)
                .flatten(),
        ),
    }
}

pub fn save_file(
    field: actix_web::multipart::Field<dev::Payload>,
) -> Box<dyn Future<Item=String, Error=Error>> {
    //TODO: create unique temp files for each upload to handle concurrent uploads
    let file_path_string = "firmware.hex";
    let file_path: String = file_path_string.to_owned();
    let mut file = match fs::File::create(file_path_string) {
        Ok(file) => file,
        Err(e) => return Box::new(future::err(error::ErrorInternalServerError(e))),
    };
    Box::new(
        field
            .fold(0i64, move |acc, bytes| {
                let rt = file
                    .write_all(bytes.as_ref())
                    .map(|_| acc + bytes.len() as i64)
                    .map_err(|e| {
                        error!("file.write_all failed: {:?}", e);
                        error::MultipartError::Payload(error::PayloadError::Io(e))
                    });
                future::result(rt)
            })
            .and_then(|size| {
                info!("file size {}", size);
                future::result(Ok(file_path))
            })
            .map_err(|e| {
                error!("save_file failed, {:?}", e);
                error::ErrorInternalServerError(e)
            }),
    )
}
