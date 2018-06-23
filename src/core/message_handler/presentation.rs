use channel::{Receiver, Sender};
use core::message::presentation::PresentationMessage;
use diesel;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use model::sensor::sensors::dsl::*;
use model::sensor::Sensor;
use r2d2::*;

pub fn handle(
    receiver: &Receiver<PresentationMessage>,
    sender: &Sender<String>,
    db_connection: PooledConnection<ConnectionManager<SqliteConnection>>,
) {
    loop {
        match receiver.recv() {
            Ok(presentation_message) => {
                create_or_update_sensor(&db_connection, &presentation_message);
                match sender.send(presentation_message.to_string()) {
                    Ok(_) => (),
                    Err(_) => error!("Error while forwarding presentation message"),
                }
            }
            _ => (),
        }
    }
}

pub fn create_or_update_sensor(
    conn: &SqliteConnection,
    presentation_message: &PresentationMessage,
) {
    let sensor_message = Sensor {
        node_id: presentation_message.node_id as i32,
        child_sensor_id: presentation_message.child_sensor_id as i32,
        sensor_type: presentation_message.sub_type,
        description: presentation_message.payload.clone(),
    };

    match sensors
        .find((sensor_message.node_id, sensor_message.child_sensor_id))
        .first::<Sensor>(conn)
    {
        Ok(existing_sensor) => if existing_sensor != sensor_message {
            match diesel::update(sensors)
                .filter(node_id.eq(sensor_message.node_id))
                .filter(child_sensor_id.eq(sensor_message.child_sensor_id))
                .set((
                    sensor_type.eq(sensor_message.sensor_type),
                    description.eq(&sensor_message.description),
                ))
                .execute(conn)
            {
                Ok(_) => info!("Updated sensor {:?}", &sensor_message),
                Err(e) => error!("Update sensor failed {:?}", e),
            }
        },
        Err(diesel::result::Error::NotFound) => match diesel::insert_into(sensors)
            .values(&sensor_message)
            .execute(conn)
        {
            Ok(_) => info!("Created sensor {:?}", &sensor_message),
            Err(e) => error!("Create sensor failed {:?}", e),
        },
        Err(e) => error!(
            "Error while checking for existing sensor{:?} {:?}",
            &sensor_message, e
        ),
    }
}
