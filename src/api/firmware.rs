use actix_web::{
    error, multipart, AsyncResponder, Error, FutureResponse, HttpMessage, HttpRequest,
    HttpResponse, Json,
};
use api::index::AppState;
use handler::firmware::*;
use http::StatusCode;
use model::firmware::Firmware;

use bytes::Bytes;
use futures::future;
use futures::{Future, Stream};
use ihex::record::Record;
use std;
use std::fs;
use std::io::Write;

pub fn upload_form(_req: HttpRequest<AppState>) -> Result<HttpResponse, error::Error> {
    let html = r#"<html>
        <head><title>Upload Test</title></head>
        <body>
            <form action="/firmwares" method="post" enctype="multipart/form-data">
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
    Box::new(
        req.clone()
            .multipart()
            .map_err(error::ErrorInternalServerError)
            .map(handle_multipart_item)
            .flatten()
            .collect()
            .map(|fields| {
                create_firmware(fields);
                HttpResponse::Ok().body("Ok")
            })
            .map_err(|e| {
                println!("failed: {}", e);
                e
            }),
    )
}

fn extract_single_field(
    fields: &Vec<(Option<String>, Option<Vec<Bytes>>)>,
    field_name: &str,
) -> Vec<Bytes> {
    fields
        .into_iter()
        .find(|(field_header, _field_value)| match field_header {
            Some(ref header) => header.contains(field_name),
            None => false,
        })
        .map(|(_field_header, field_value)| match field_value {
            Some(value) => value.to_owned(),
            None => Vec::new(),
        })
        .unwrap_or(Vec::new())
}

fn create_firmware(fields: Vec<(Option<String>, Option<Vec<Bytes>>)>) {
    let firmware_type = extract_single_field(&fields, "firmware_type");
    let firmware_version = extract_single_field(&fields, "firmware_version");
    let firmware_file = extract_single_field(&fields, "firmware_file");

    println!("firmware_type {:?}", to_string(firmware_type));
    println!("firmware_version {:?}", to_string(firmware_version));
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
    println!("binary_data {:?}", binary_data);
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
    println!("field: {:?}", field);
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
    // RFC 7578: 'Each part MUST contain a Content-Disposition header field
    // where the disposition type is "form-data".'
    field
        .headers()
        .get(::http::header::CONTENT_DISPOSITION)
        .and_then(|f| f.to_str().map(|string| string.to_owned()).ok())
}

fn to_string(value: Vec<Bytes>) -> String {
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

// #[put("/firmwares", data = "<_data>")]
// fn update(
//     cont_type: &ContentType,
//     _data: Data,
//     conn: DbConn,
// ) -> Result<Stream<Cursor<Vec<u8>>>, Custom<String>> {
//     if !cont_type.is_form_data() {
//         return Err(Custom(
//             Status::BadRequest,
//             "Content-Type not multipart/form-data".into(),
//         ));
//     }
//     let (_, boundary) = cont_type
//         .params()
//         .find(|&(k, _)| k == "boundary")
//         .ok_or_else(|| {
//             Custom(
//                 Status::BadRequest,
//                 "`Content-Type: multipart/form-data` boundary param not provided".into(),
//             )
//         })?;

//     match process_upload(boundary, _data) {
//         Ok((mut resp, firmware)) => {
//             if let Some(firmware) = firmware {
//                 let firmware_clone = firmware.clone();
//                 let update_result = diesel::update(
//                     firmwares.find((firmware.firmware_type, firmware.firmware_version)),
//                 ).set((
//                     blocks.eq(firmware.blocks),
//                     name.eq(firmware.name),
//                     crc.eq(firmware.crc),
//                     data.eq(firmware.data),
//                 ))
//                     .execute(&*conn);
//                 match update_result {
//                     Ok(0) => Err(Custom(Status::BadRequest, format!("Update failed. There is no firmware with type {}, version {}",
//                         firmware_clone.firmware_type, firmware_clone.firmware_version).to_owned())),
//                     Ok(_) => {
//                         writeln!(resp, "Updated firmware type: {}, version: {}", firmware_clone.firmware_type, firmware_clone.firmware_version).unwrap();
//                         Ok(Stream::from(Cursor::new(resp)))
//                     },
//                     Err(error) => Err(Custom(Status::InternalServerError, error.to_string())),
//                 }
//             } else {
//                 Ok(Stream::from(Cursor::new(resp)))
//             }
//         }
//         Err(err) => Err(Custom(Status::InternalServerError, err.to_string())),
//     }
// }

// #[post("/firmwares", data = "<_data>")]
// fn upload(
//     cont_type: &ContentType,
//     _data: Data,
//     conn: DbConn,
// ) -> Result<Stream<Cursor<Vec<u8>>>, Custom<String>> {
//     if !cont_type.is_form_data() {
//         return Err(Custom(
//             Status::BadRequest,
//             "Content-Type not multipart/form-data".into(),
//         ));
//     }
//     let (_, boundary) = cont_type
//         .params()
//         .find(|&(k, _)| k == "boundary")
//         .ok_or_else(|| {
//             Custom(
//                 Status::BadRequest,
//                 "`Content-Type: multipart/form-data` boundary param not provided".into(),
//             )
//         })?;
//     match process_upload(boundary, _data) {
//         Ok((resp, firmware)) => {
//             if let Some(firmware) = firmware {
//                 let firmware_clone = firmware.clone();
//                 match diesel::insert_into(firmwares)
//                     .values(firmware)
//                     .execute(&*conn)
//                 {
//                     Ok(_) => Ok(Stream::from(Cursor::new(resp))),
//                     Err(DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, msg)) => {
//                         println!(
//                             "Unique constraint violated while inserting firmware {:?}",
//                             msg
//                         );
//                         Err(Custom(Status::BadRequest, format!("Already there is firmware with firmware_type {:?}, firmware_version {:?}",
//                          firmware_clone.firmware_type, firmware_clone.firmware_version).to_owned()))
//                     }
//                     Err(err) => Err(Custom(
//                         Status::BadRequest,
//                         format!("Error while inserting firmware {:?}", err).to_owned(),
//                     )),
//                 }
//             } else {
//                 Ok(Stream::from(Cursor::new(resp)))
//             }
//         }
//         Err(err) => Err(Custom(Status::InternalServerError, err.to_string())),
//     }
// }

// fn process_upload(boundary: &str, _data: Data) -> io::Result<(Vec<u8>, Option<Firmware>)> {
//     let mut out = Vec::new();

//     // saves all fields, any field longer than 10kB goes to a temporary directory
//     // Entries could implement FromData though that would give zero control over
//     // how the files are saved; Multipart would be a good impl candidate though
//     let firmware = match Multipart::with_body(_data.open(), boundary).save().temp() {
//         Full(entries) => process_entries(entries, &mut out),
//         Partial(partial, reason) => {
//             writeln!(out, "Request partially processed: {:?}", reason)?;
//             if let Some(field) = partial.partial {
//                 writeln!(out, "Stopped on field: {:?}", field.source.headers)?;
//             }

//             process_entries(partial.entries, &mut out)
//         }
//         Error(e) => return Err(e),
//     };

//     Ok((out, firmware))
// }

// // having a streaming output would be nice; there's one for returning a `Read` impl
// // but not one that you can `write()` to
// fn process_entries(entries: Entries, mut out: &mut Vec<u8>) -> Option<Firmware> {
//     let stdout = io::stdout();
//     let mut _tee = StdoutTee::new(&mut out, &stdout);
//     //        entries.write_debug(tee)?;

//     fn extract_entry<'a>(
//         entries: &'a Entries,
//         key: &'a str,
//         tee: &mut StdoutTee<&mut &mut Vec<u8>>,
//     ) -> Option<&'a SavedData> {
//         match entries.fields.get(&String::from(key)) {
//             Some(field) => field.first().map(|f| &f.data),
//             None => {
//                 writeln!(tee, "Missing field {}", key).unwrap();
//                 None
//             }
//         }
//     }

//     fn extract_firmware_entries<'a>(
//         entries: &'a Entries,
//         tee: &mut StdoutTee<&mut &mut Vec<u8>>,
//     ) -> (
//         Option<&'a SavedData>,
//         Option<&'a SavedData>,
//         Option<&'a SavedData>,
//         Option<&'a SavedData>,
//     ) {
//         (
//             extract_entry(entries, "firmware_type", tee),
//             extract_entry(entries, "firmware_version", tee),
//             extract_entry(entries, "firmware_name", tee),
//             extract_entry(entries, "file", tee),
//         )
//     }
//     if let (
//         Some(SavedData::Text(ref _firmware_type)),
//         Some(SavedData::Text(ref _firmware_version)),
//         Some(SavedData::Text(ref firmware_name)),
//         Some(SavedData::File(ref file, _)),
//     ) = extract_firmware_entries(&entries, &mut _tee)
//     {
//         println!("Loading firmware {:?}", firmware_name);
//         println!("firmware_type {:?}", _firmware_type);
//         println!("firmware_version {:?}", _firmware_version);
//         let firmware = Firmware::prepare_fw(
//             _firmware_type.parse().unwrap(),
//             _firmware_version.parse().unwrap(),
//             firmware_name.clone(),
//             file,
//         );
//         writeln!(&mut _tee, "<Success>").unwrap();
//         Some(firmware)
//     } else {
//         writeln!(&mut _tee, "<Error>").unwrap();
//         None
//     }
// }
