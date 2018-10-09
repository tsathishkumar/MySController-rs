use crate::core::message::presentation::PresentationType;

table! {
    use diesel::sql_types::Integer;
    use diesel::sql_types::Text;
    use crate::core::message::presentation::PresentationTypeMapping;

    sensors (node_id, child_sensor_id) {
        node_id -> Integer,
        child_sensor_id -> Integer,
        sensor_type -> PresentationTypeMapping,
        description -> Text,
    }
}

#[derive(Queryable, Serialize, Deserialize, Insertable, Debug, PartialEq, Clone)]
#[table_name = "sensors"]
pub struct Sensor {
    pub node_id: i32,
    pub child_sensor_id: i32,
    pub sensor_type: PresentationType,
    pub description: String,
}
