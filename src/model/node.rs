table! {
    nodes (node_id) {
        node_id -> Integer,
        node_name -> Text,
        firmware_type -> Integer,
        firmware_version -> Integer,
        desired_firmware_type -> Integer,
        desired_firmware_version -> Integer,
        auto_update -> Bool,
        scheduled -> Bool,
    }
}

#[derive(Queryable, Serialize, Deserialize, Insertable)]
#[table_name = "nodes"]
pub struct Node {
    pub node_id: i32,
    pub node_name: String,
    pub firmware_type: i32,
    pub firmware_version: i32,
    pub desired_firmware_type: i32,
    pub desired_firmware_version: i32,
    pub auto_update: bool,
    pub scheduled: bool,
}

impl Node {
    pub fn node_id(&self) -> u8 {
        self.node_id as u8
    }
}
