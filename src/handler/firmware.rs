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
    pub blocks: i32,
    pub crc: i32,
}

pub enum CreateOrUpdate {
    Create(NewFirmware),
    Update(NewFirmware),
}

impl Message for CreateOrUpdate {
    type Result = Result<Msgs, Msgs>;
}

impl Handler<CreateOrUpdate> for ConnDsl {
    type Result = Result<Msgs, Msgs>;

    fn handle(&mut self, create_or_update: CreateOrUpdate, _: &mut Self::Context) -> Self::Result {
        use model::firmware::firmwares::dsl::*;
        match create_or_update {
            CreateOrUpdate::Create(new_firmware) => self.0
                .get()
                .map_err(|_| Msgs {
                    status: 500,
                    message: "internal server error.".to_string(),
                })
                .and_then(|conn| {
                    let new_firmware = Firmware {
                        firmware_type: new_firmware.firmware_type,
                        firmware_version: new_firmware.firmware_version,
                        name: new_firmware.name,
                        blocks: new_firmware.blocks,
                        crc: new_firmware.crc,
                        data: new_firmware.data,
                    };

                    diesel::insert_into(firmwares)
                        .values(&new_firmware)
                        .execute(&conn)
                        .map_err(|e| match e {
                            DatabaseError(DatabaseErrorKind::UniqueViolation, _) => Msgs {
                                status: 400,
                                message: "firmware already present.".to_string(),
                            },
                            _ => Msgs {
                                status: 500,
                                message: "internal server error.".to_string(),
                            },
                        })
                        .and_then(|_| {
                            info!("Created new firmware - {:?}", &new_firmware);
                            auto_update_nodes(&conn, new_firmware)
                        })
                }),
            CreateOrUpdate::Update(new_firmware) => self.0
                .get()
                .map_err(|_| Msgs {
                    status: 500,
                    message: "internal server error.".to_string(),
                })
                .and_then(|conn| {
                    diesel::update(firmwares)
                        .filter(
                            firmware_type
                                .eq(new_firmware.firmware_type)
                                .and(firmware_version.eq(new_firmware.firmware_version)),
                        )
                        .set((
                            name.eq(new_firmware.name),
                            blocks.eq(new_firmware.blocks),
                            crc.eq(new_firmware.crc),
                            data.eq(new_firmware.data),
                        ))
                        .execute(&conn)
                        .map_err(|_| Msgs {
                            status: 500,
                            message: "update failed. internal server error".to_string(),
                        })
                        .map(|updated_count| match updated_count {
                            1 => Msgs {
                                status: 200,
                                message: "update firmware success.".to_string(),
                            },
                            _ => Msgs {
                                status: 400,
                                message: "update firmware failed. type and version is not present"
                                    .to_string(),
                            },
                        })
                }),
        }
    }
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

impl NewFirmware {
    pub fn build(
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

fn auto_update_nodes(connection: &SqliteConnection, new_firmware: Firmware) -> Result<Msgs, Msgs> {
    use model::node::nodes::dsl::*;
    diesel::update(nodes)
        .filter(auto_update.eq(true))
        .filter(
            desired_firmware_type
                .eq(new_firmware.firmware_type)
                .and(desired_firmware_version.lt(new_firmware.firmware_version)),
        )
        .set((
            desired_firmware_version.eq(new_firmware.firmware_version),
            scheduled.eq(true),
        ))
        .execute(connection)
        .map(|update_count| Msgs {
            status: 200,
            message: format!(
                "create firmware success. upgraded for {} nodes",
                update_count
            ).to_string(),
        })
        .map_err(|_| Msgs {
            status: 400,
            message: "create firmware success. upgraded for nodes failed".to_string(),
        })
}

impl FirmwareDto {
    fn new(firmware: &Firmware) -> FirmwareDto {
        FirmwareDto {
            firmware_type: firmware.firmware_type,
            firmware_version: firmware.firmware_version,
            firmware_name: firmware.name.clone(),
            blocks: firmware.blocks,
            crc: firmware.crc,
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

        match firmwares
            .find((firmware.firmware_type, firmware.firmware_version))
            .first::<Firmware>(conn)
        {
            Ok(v) => Ok(v),
            Err(_) => return Err(()),
        }
    }
}
