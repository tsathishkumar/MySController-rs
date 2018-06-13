#[derive(Deserialize,Serialize, Debug)]
pub struct Msgs {
    pub status: u16,
    pub message : String,
}