use actix_web::{error, multipart, AsyncResponder, Error, FutureResponse, HttpMessage, HttpRequest,
                HttpResponse, Json};
use api::index::AppState;
use bytes::Bytes;
use futures::future;
use futures::{Future, Stream};
use handler::firmware::UpdateFirmware;
use handler::firmware::*;
use handler::response::Msgs;
use http::StatusCode;
use ihex::record::Record;
use model::firmware::Firmware;
use std;

pub fn upload_form(_req: HttpRequest<AppState>) -> Result<HttpResponse, error::Error> {
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

pub fn list(req: HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
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

pub fn delete(
    firmware_delete: Json<DeleteFirmware>,
    req: HttpRequest<AppState>,
) -> FutureResponse<HttpResponse> {
    req.state()
        .db
        .send(DeleteFirmware {
            firmware_type: firmware_delete.firmware_type,
            firmware_version: firmware_delete.firmware_version,
        })
        .from_err()
        .and_then(|res| match res {
            Ok(msg) => Ok(HttpResponse::build(StatusCode::from_u16(msg.status).unwrap()).json(msg)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

pub fn create(req: HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    let req_clone = req.clone();
    req_clone
        .multipart()
        .map_err(error::ErrorInternalServerError)
        .map(handle_multipart_item)
        .flatten()
        .collect()
        .and_then(move |fields| match get_firmware(fields) {
            Ok(firmware) => req.state()
                .db
                .send(firmware)
                .from_err()
                .map(|res| match res {
                    Ok(msg) => {
                        HttpResponse::build(StatusCode::from_u16(msg.status).unwrap()).json(msg)
                    }

                    Err( e) => {
                        error!("Error while uploading firmware {:?}", e);
                        HttpResponse::build(StatusCode::from_u16(e.status).unwrap()).json(e)
                    }
                })
                .responder(),
            Err(msg) => Box::new(future::result(Ok(HttpResponse::build(
                StatusCode::from_u16(msg.status).unwrap(),
            ).json(msg)))),
        })
        .responder()
}

pub fn update(req: HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    let req_clone = req.clone();
    req_clone
        .multipart()
        .map_err(error::ErrorInternalServerError)
        .map(handle_multipart_item)
        .flatten()
        .collect()
        .and_then(move |fields| match get_firmware(fields) {
            Ok(firmware) => req.state()
                .db
                .send(UpdateFirmware(firmware))
                .from_err()
                .and_then(|res| match res {
                    Ok(msg) => Ok(
                        HttpResponse::build(StatusCode::from_u16(msg.status).unwrap()).json(msg),
                    ),
                    Err(e) => {
                        error!("Error while uploading firmware {:?}", e);
                        Ok(HttpResponse::InternalServerError().into())
                    }
                })
                .responder(),
            Err(msg) => Box::new(future::result(Ok(HttpResponse::build(
                StatusCode::from_u16(msg.status).unwrap(),
            ).json(msg)))),
        })
        .responder()
}

fn extract_single_field<'a>(
    fields: &Vec<(Option<String>, Option<Vec<Bytes>>)>,
    field_name: &'a str,
) -> (&'a str, Option<Vec<Bytes>>) {
    (
        field_name,
        fields
            .into_iter()
            .find(|(field_header, _field_value)| match field_header {
                Some(ref header) => header.contains(field_name),
                None => false,
            })
            .map(|(_field_header, field_value)| match field_value {
                Some(value) => value.to_owned(),
                None => Vec::new(),
            }),
    )
}

fn get_firmware(fields: Vec<(Option<String>, Option<Vec<Bytes>>)>) -> Result<NewFirmware, Msgs> {
    let firmware_name = extract_single_field(&fields, "firmware_name");
    let firmware_type = extract_single_field(&fields, "firmware_type");
    let firmware_version = extract_single_field(&fields, "firmware_version");
    let firmware_file = extract_single_field(&fields, "firmware_file");

    let expected_fields = vec![
        firmware_name,
        firmware_type,
        firmware_version,
        firmware_file,
    ];

    if let [(_, Some(firmware_name)), (_, Some(firmware_type)), (_, Some(firmware_version)), (_, Some(firmware_file))] =
        &expected_fields.as_slice()
    {
        let firmware_name = to_string(&firmware_name);
        let firmware_type = to_string(&firmware_type);
        let firmware_version = to_string(&firmware_version);
        let binary_data: Vec<u8> = firmware_file
            .into_iter()
            .map(|b| {
                String::from(std::str::from_utf8(b.as_ref()).unwrap())
                    .trim()
                    .to_owned()
            })
            .filter(|line| !line.is_empty())
            .flat_map(|line| Firmware::ihex_to_bin(&Record::from_record_string(&line).unwrap()))
            .collect();

        error!(
            "upload request for new firmware {} type {}, version {}",
            firmware_name, firmware_type, firmware_version
        );
        let new_firmware = NewFirmware::prepare_in_memory(
            firmware_type.parse::<i32>().unwrap(),
            firmware_version.parse::<i32>().unwrap(),
            firmware_name,
            binary_data,
        );

        return Ok(new_firmware);
    }
    let missing_fields: Vec<&str> = expected_fields
        .into_iter()
        .filter(|(_, v)| v.is_none())
        .map(|(name, _)| name)
        .collect();
    Err(Msgs {
        status: 400,
        message: String::from(format!("Missing fields [{}]", missing_fields.join(", "))),
    })
}

fn handle_multipart_item(
    item: multipart::MultipartItem<HttpRequest<AppState>>,
) -> Box<Stream<Item = (Option<String>, Option<Vec<Bytes>>), Error = Error>> {
    match item {
        multipart::MultipartItem::Field(field) => {
            Box::new(extract_field_value(field).into_stream())
        }
        multipart::MultipartItem::Nested(mp) => Box::new(
            mp.map_err(error::ErrorInternalServerError)
                .map(handle_multipart_item)
                .flatten(),
        ),
    }
}

fn extract_field_value(
    field: multipart::Field<HttpRequest<AppState>>,
) -> Box<Future<Item = (Option<String>, Option<Vec<Bytes>>), Error = Error>> {
    Box::new(future::result(Ok((
        content_disposition(&field),
        field
            .map_err(|e| error::ErrorInternalServerError(e))
            .collect()
            .wait()
            .ok(),
    ))))
}

fn content_disposition(field: &multipart::Field<HttpRequest<AppState>>) -> Option<String> {
    //TODO: refactor to use from actix-web after upgrade
    field
        .headers()
        .get(::http::header::CONTENT_DISPOSITION)
        .and_then(|f| f.to_str().map(|string| string.to_owned()).ok())
}

fn to_string(value: &Vec<Bytes>) -> String {
    String::from(
        std::str::from_utf8(
            value
                .into_iter()
                .flat_map(|b| b.as_ref().to_owned())
                .collect::<Vec<u8>>()
                .as_ref(),
        ).unwrap(),
    )
}
