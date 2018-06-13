use super::response::Msgs;
use actix::*;
use actix_web::*;
use diesel;
use diesel::prelude::*;
use diesel::result::DatabaseErrorKind;
use diesel::result::Error::DatabaseError;
use model;
use model::db::ConnDsl;
use model::firmware::Firmware;

#[derive(Serialize, Deserialize)]
pub struct FirmwareDto {
    pub firmware_type: i32,
    pub firmware_version: i32,
    pub firmware_name: String,
}

#[derive(Debug)]
pub struct NewFirmware {
    pub firmware_type: i32,
    pub firmware_version: i32,
    pub name: String,
    pub blocks: i32,
    pub crc: i32,
    pub data: Vec<u8>,
}

impl Message for NewFirmware {
    type Result = Result<Msgs, diesel::result::Error>;
}

impl NewFirmware {
    pub fn prepare_in_memory(
        fimware_type: i32,
        version: i32,
        name: String,
        mut binary_data: Vec<u8>,
    ) -> NewFirmware {
        let pads: usize = binary_data.len() % 128; // 128 bytes per page for atmega328
        for _ in 0..(128 - pads) {
            binary_data.push(255);
        }
        let blocks: i32 = binary_data.len() as i32 / model::firmware::FIRMWARE_BLOCK_SIZE;
        let crc = Firmware::compute_crc(&binary_data) as i32;
        NewFirmware {
            firmware_type: fimware_type,
            firmware_version: version,
            blocks: blocks,
            data: binary_data,
            name: name,
            crc: crc,
        }
    }
}

impl Handler<NewFirmware> for ConnDsl {
    type Result = Result<Msgs, diesel::result::Error>;

    fn handle(&mut self, new_firmware: NewFirmware, _: &mut Self::Context) -> Self::Result {
        use model::firmware::firmwares::dsl::*;
        match &self.0.get() {
            Ok(conn) => {
                let new_firmware = Firmware {
                    firmware_type: new_firmware.firmware_type,
                    firmware_version: new_firmware.firmware_version,
                    name: new_firmware.name,
                    blocks: new_firmware.blocks,
                    crc: new_firmware.crc,
                    data: new_firmware.data,
                };

                let result = diesel::insert_into(firmwares)
                    .values(&new_firmware)
                    .execute(conn);

                match result {
                    Ok(_) => Ok(Msgs {
                        status: 200,
                        message: "create firmware success.".to_string(),
                    }),
                    Err(DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => Ok(Msgs {
                        status: 400,
                        message: "firmware already present.".to_string(),
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

pub struct UpdateFirmware(pub NewFirmware);

impl Message for UpdateFirmware {
    type Result = Result<Msgs, diesel::result::Error>;
}

impl Handler<UpdateFirmware> for ConnDsl {
    type Result = Result<Msgs, diesel::result::Error>;

    fn handle(&mut self, update_firmware: UpdateFirmware, _: &mut Self::Context) -> Self::Result {
        use model::firmware::firmwares::dsl::*;
        match &self.0.get() {
            Ok(conn) => {
                let updated = diesel::update(firmwares)
                    .filter(&firmware_type.eq(&update_firmware.0.firmware_type))
                    .set((
                        firmware_type.eq(update_firmware.0.firmware_type),
                        firmware_version.eq(update_firmware.0.firmware_version),
                        name.eq(update_firmware.0.name),
                        blocks.eq(update_firmware.0.blocks),
                        crc.eq(update_firmware.0.crc),
                        data.eq(update_firmware.0.data),
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
