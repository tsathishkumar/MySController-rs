use super::response::Msgs;
use actix::*;
use actix_web::*;
use diesel;
use diesel::prelude::*;
use crate::model::db::ConnDsl;
use crate::model::sensor::Sensor;

pub struct GetSensor {
    pub node_id: i32,
    pub child_sensor_id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteSensor {
    pub node_id: i32,
    pub child_sensor_id: i32,
}

impl Message for GetSensor {
    type Result = Result<Sensor, ()>;
}

impl Message for DeleteSensor {
    type Result = Result<Msgs, diesel::result::Error>;
}

pub struct ListSensors;

impl Message for ListSensors {
    type Result = Result<Vec<Sensor>, Error>;
}

impl Handler<ListSensors> for ConnDsl {
    type Result = Result<Vec<Sensor>, Error>;

    fn handle(&mut self, _list_sensors: ListSensors, _: &mut Self::Context) -> Self::Result {
        use crate::model::sensor::sensors::dsl::*;
        let conn = &self.0.get().map_err(error::ErrorInternalServerError)?;
        let existing_sensors = sensors
            .load::<Sensor>(conn)
            .map_err(error::ErrorInternalServerError)?;
        Ok(existing_sensors)
    }
}

impl Handler<DeleteSensor> for ConnDsl {
    type Result = Result<Msgs, diesel::result::Error>;

    fn handle(&mut self, delete_sensor: DeleteSensor, _: &mut Self::Context) -> Self::Result {
        use crate::model::sensor::sensors::dsl::*;
        match &self.0.get() {
            Ok(conn) => {
                let updated = diesel::delete(sensors)
                    .filter(&node_id.eq(&delete_sensor.node_id))
                    .filter(&child_sensor_id.eq(&delete_sensor.child_sensor_id))
                    .execute(conn);
                match updated {
                    Ok(1) => Ok(Msgs {
                        status: 200,
                        message: "deleted sensor.".to_string(),
                    }),
                    Ok(_) => Ok(Msgs {
                        status: 400,
                        message: "delete failed. sensor not present".to_string(),
                    }),
                    Err(e) => Err(e),
                }
            }
            Err(_) => Ok(Msgs {
                status: 500,
                message: "delete failed. internal server error".to_string(),
            }),
        }
    }
}

impl Handler<GetSensor> for ConnDsl {
    type Result = Result<Sensor, ()>;

    fn handle(&mut self, sensor: GetSensor, _: &mut Self::Context) -> Self::Result {
        use crate::model::sensor::sensors::dsl::*;

        let conn = &self.0.get().map_err(|_| ())?;

        match sensors
            .find((sensor.node_id, sensor.child_sensor_id))
            .first::<Sensor>(conn)
        {
            Ok(v) => Ok(v),
            _ => return Err(()),
        }
    }
}
