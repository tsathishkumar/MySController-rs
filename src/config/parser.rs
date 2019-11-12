use std::fs::File;
use std::io::Read;
use crate::config::model::Config;
use toml::de::Error;

pub fn parse() -> Result<Config, Error> {
    let mut conf_file: File = match File::open("/etc/myscontroller-rs/conf.toml"){
        Ok(_conf_file) => _conf_file,
        Err(_) => File::open("conf.toml").unwrap(),
    };

    let mut conf_string: String = String::new();
    conf_file.read_to_string(&mut conf_string).expect("Unable to read the config file");

    let config: Config = match toml::from_str(&conf_string) {
        Ok(_config) => _config,
        Err(err) => return Err(err)
    };

    Ok(config)
}