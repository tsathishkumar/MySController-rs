use diesel;
use diesel::prelude::*;
use firmware::Firmware;
use firmware::firmwares::dsl::firmwares;
use multipart::mock::StdoutTee;
use multipart::server::Multipart;
use multipart::server::save::{Entries, SavedData};
use multipart::server::save::SaveResult::*;
use pool::DbConn;
use rocket::Data;
use rocket::http::{ContentType, Status};
use rocket::response::status::Custom;
use rocket::response::Stream;
use rocket_contrib::Json;
use std::io::{self, Cursor, Write};

#[derive(Serialize)]
struct FirmwareDto {
    pub firmware_type: i32,
    pub firmware_version: i32,
    pub name: String,
}

impl FirmwareDto {
    fn new(firmware: &Firmware) -> FirmwareDto {
        FirmwareDto { firmware_type: firmware.firmware_type, firmware_version: firmware.firmware_version, name: firmware.name.clone() }
    }
}

#[get("/firmwares")]
fn get_firmwares(conn: DbConn) -> Json<Vec<FirmwareDto>> {
    let existing_firmwares = firmwares
        .load::<Firmware>(&*conn).expect("error while loading existing firmwares");
    let existing_firmwares = existing_firmwares
        .iter()
        .map(|f| FirmwareDto::new(f))
        .collect();
    Json(existing_firmwares)
}


#[post("/firmwares/upload", data = "<data>")]
fn upload(cont_type: &ContentType, data: Data, conn: DbConn) -> Result<Stream<Cursor<Vec<u8>>>, Custom<String>> {
    if !cont_type.is_form_data() {
        return Err(Custom(
            Status::BadRequest,
            "Content-Type not multipart/form-data".into(),
        ));
    }
    let (_, boundary) = cont_type.params().find(|&(k, _)| k == "boundary").ok_or_else(
        || Custom(
            Status::BadRequest,
            "`Content-Type: multipart/form-data` boundary param not provided".into(),
        )
    )?;

    match process_upload(boundary, data) {
        Ok((resp, firmware)) => {
            if let Some(firmware) = firmware {
                diesel::insert_into(firmwares)
                    .values(firmware)
                    .execute(&*conn)
                    .unwrap();
            }
            Ok(Stream::from(Cursor::new(resp)))
        }
        Err(err) => Err(Custom(Status::InternalServerError, err.to_string()))
    }
}

fn process_upload(boundary: &str, data: Data) -> io::Result<(Vec<u8>, Option<Firmware>)> {
    let mut out = Vec::new();

    // saves all fields, any field longer than 10kB goes to a temporary directory
    // Entries could implement FromData though that would give zero control over
    // how the files are saved; Multipart would be a good impl candidate though
    let firmware = match Multipart::with_body(data.open(), boundary).save().temp() {
        Full(entries) => process_entries(entries, &mut out),
        Partial(partial, reason) => {
            writeln!(out, "Request partially processed: {:?}", reason)?;
            if let Some(field) = partial.partial {
                writeln!(out, "Stopped on field: {:?}", field.source.headers)?;
            }

            process_entries(partial.entries, &mut out)
        }
        Error(e) => return Err(e),
    };

    Ok((out, firmware))
}

// having a streaming output would be nice; there's one for returning a `Read` impl
// but not one that you can `write()` to
fn process_entries(entries: Entries, mut out: &mut Vec<u8>) -> Option<Firmware> {
    let stdout = io::stdout();
    let mut _tee = StdoutTee::new(&mut out, &stdout);
    //        entries.write_debug(tee)?;

    fn extract_entry<'a>(entries: &'a Entries, key: &'a str, tee: &mut StdoutTee<&mut &mut Vec<u8>>) -> Option<&'a SavedData> {
        match entries.fields.get(&String::from(key)) {
            Some(field) => field.first().map(|f| &f.data),
            None => {
                writeln!(tee, "Missing field {}", key).unwrap();
                None
            }
        }
    }

    fn extract_firmware_entries<'a>(entries: &'a Entries, tee: &mut StdoutTee<&mut &mut Vec<u8>>) -> (Option<&'a SavedData>, Option<&'a SavedData>, Option<&'a SavedData>, Option<&'a SavedData>) {
        (extract_entry(entries, "firmware_type", tee), extract_entry(entries, "firmware_version", tee),
         extract_entry(entries, "firmware_name", tee), extract_entry(entries, "file", tee))
    }
    if let (Some(SavedData::Text(ref firmware_type)), Some(SavedData::Text(ref firmware_version)), Some(SavedData::Text(ref firmware_name)),
        Some(SavedData::File(ref file, _))) = extract_firmware_entries(&entries, &mut _tee) {
        println!("Loading firmware {:?}", firmware_name);
        println!("firmware_type {:?}", firmware_type);
        println!("firmware_version {:?}", firmware_version);
        let firmware = Firmware::prepare_fw(firmware_type.parse().unwrap(), firmware_version.parse().unwrap(), firmware_name.clone(), file);
        writeln!(&mut _tee, "<Success>").unwrap();
        Some(firmware)
    } else {
        writeln!(&mut _tee, "<Error>").unwrap();
        None
    }
}