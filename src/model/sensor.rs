use core::message::presentation::PresentationType;
use core::message::set::{SetMessage, SetReqType, Value};

table! {
    use diesel::sql_types::Integer;
    use diesel::sql_types::Text;
    use core::message::presentation::PresentationTypeMapping;

    sensors (node_id, child_sensor_id) {
        node_id -> Integer,
        child_sensor_id -> Integer,
        sensor_type -> PresentationTypeMapping,
        description -> Text,
    }
}

#[derive(Queryable, Serialize, Deserialize, Insertable, Debug, PartialEq)]
#[table_name = "sensors"]
pub struct Sensor {
    pub node_id: i32,
    pub child_sensor_id: i32,
    pub sensor_type: PresentationType,
    pub description: String,
}

impl Sensor {
    pub fn to_set_status_message(&self, status: bool) -> SetMessage {
        let status = match status {
            true => "1",
            false => "0",
        };
        SetMessage {
            node_id: self.node_id as u8,
            child_sensor_id: self.child_sensor_id as u8,
            ack: 0,
            value: Value {
                set_type: SetReqType::Status,
                value: status.to_owned(),
            },
        }
    }
}
