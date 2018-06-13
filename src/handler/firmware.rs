use super::response::Msgs;
use actix::*;
use actix_web::*;
use diesel;
use diesel::prelude::*;
use diesel::result::DatabaseErrorKind;
use diesel::result::Error::DatabaseError;
use model::db::ConnDsl;
use model::firmware::Firmware;

#[derive(Serialize, Deserialize)]
pub struct FirmwareDto {
    pub firmware_type: i32,
    pub firmware_version: i32,
    pub firmware_name: String,
}

pub struct NewFirmware {
    pub firmware_type: i32,
    pub firmware_version: i32,
    pub name: String,
    pub data: Vec<u8>,
}

impl FirmwareDto {
    fn new(firmware: &Firmware) -> FirmwareDto {
        FirmwareDto {
            firmware_type: firmware.firmware_type,
            firmware_version: firmware.firmware_version,
            firmware_name: firmware.name.clone(),
        }
    }
}

pub struct ListFirmwares;

impl Message for ListFirmwares {
    type Result = Result<Vec<FirmwareDto>, Error>;
}

impl Handler<ListFirmwares> for ConnDsl {
    type Result = Result<Vec<FirmwareDto>, Error>;

    fn handle(&mut self, _list_firmwares: ListFirmwares, _: &mut Self::Context) -> Self::Result {
        use model::firmware::firmwares::dsl::*;
        let conn = &self.0.get().map_err(error::ErrorInternalServerError)?;
        let existing_firmwares = firmwares
            .load::<Firmware>(conn)
            .map_err(error::ErrorInternalServerError)?;
        Ok(existing_firmwares
            .iter()
            .map(|firmware| FirmwareDto::new(firmware))
            .collect())
    }
}

#[derive(Serialize, Deserialize)]
pub struct DeleteFirmware {
    pub firmware_type: i32,
    pub firmware_version: i32,
}

impl Message for DeleteFirmware {
    type Result = Result<Msgs, diesel::result::Error>;
}

impl Handler<DeleteFirmware> for ConnDsl {
    type Result = Result<Msgs, diesel::result::Error>;

    fn handle(&mut self, delete_firmware: DeleteFirmware, _: &mut Self::Context) -> Self::Result {
        use model::firmware::firmwares::dsl::*;
        match &self.0.get() {
            Ok(conn) => {
                let updated = diesel::delete(firmwares)
                    .filter(&firmware_type.eq(&delete_firmware.firmware_type))
                    .filter(&firmware_version.eq(&delete_firmware.firmware_version))
                    .execute(conn);
                match updated {
                    Ok(1) => Ok(Msgs {
                        status: 200,
                        message: "deleted firmware.".to_string(),
                    }),
                    Ok(_) => Ok(Msgs {
                        status: 400,
                        message: "delete failed. firmware is not present".to_string(),
                    }),
                    Err(e) => Err(e),
                }
            }
            Err(_) => Ok(Msgs {
                status: 500,
                message: "delete firmware failed. internal server error".to_string(),
            }),
        }
    }
}

pub struct GetFirmware {
    pub firmware_type: i32,
    pub firmware_version: i32,
}

impl Message for GetFirmware {
    type Result = Result<Firmware, ()>;
}

impl Handler<GetFirmware> for ConnDsl {
    type Result = Result<Firmware, ()>;

    fn handle(&mut self, firmware: GetFirmware, _: &mut Self::Context) -> Self::Result {
        use model::firmware::firmwares::dsl::*;

        let conn = &self.0.get().map_err(|_| ())?;

        let existing_firmware = firmwares
            .find((firmware.firmware_type, firmware.firmware_version))
            .first::<Firmware>(conn)
            .optional()
            .unwrap();
        match existing_firmware {
            Some(v) => Ok(v),
            None => return Err(()),
        }
    }
}
