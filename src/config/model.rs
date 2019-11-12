use serde_derive::Deserialize;

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Config {
    pub Server: Option<Server>,
    pub Gateway: Option<Gateway>,
    pub Controller: Option<Controller>,
}

#[derive(Deserialize, Debug)]
pub struct Controller {
    pub r#type: Option<String>,
    pub port: Option<String>,
    pub timeout_enabled: Option<String>,
    pub baud_rate: Option<String>,
    pub broker: Option<String>,
    pub publish_topic_prefix: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Server {
    pub database_url: Option<String>,
    pub log_level: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Gateway {
    pub r#type: Option<String>,
    pub port: Option<String>,
    pub timeout_enabled: Option<String>,
    pub baud_rate: Option<String>,
    pub broker: Option<String>,
    pub publish_topic_prefix: Option<String>,
}
