use actix::*;
use actix_web::*;
use diesel::prelude::*;
use model::db::ConnDsl;
use model::firmware::Firmware;

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
