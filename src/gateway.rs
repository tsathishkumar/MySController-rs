use ota;

use serial;
use serial::prelude::*;
use std::io::prelude::*;

use std::str;
use std::time::Duration;


const SETTINGS: serial::PortSettings = serial::PortSettings {
    baud_rate: serial::Baud9600,
    char_size: serial::Bits8,
    parity: serial::ParityNone,
    stop_bits: serial::Stop1,
    flow_control: serial::FlowNone,
};

pub struct SerialGateway {
    pub serial_port: String,
}

impl SerialGateway {
    pub fn connect(&self) {
        let mut port = serial::open(&self.serial_port).unwrap();
        SerialGateway::interact(&self, &mut port).unwrap(); //TODO: make it async
        ota::some_funtion();
    }

    pub fn interact<T: SerialPort>(&self, port: &mut T) -> serial::Result<()> {
        try!(port.configure(&SETTINGS));
        try!(port.set_timeout(Duration::from_secs(10)));

        let mut buf: Vec<u8> = (1..255).collect();
        port.write(&String::from("Hello").as_bytes());

        loop {
            let mut read_buf: Vec<u8> = Vec::new();

            port.read_to_end(&mut read_buf).unwrap(); //TODO: error handling

            if (read_buf.len() > 1) {
                println!("{:?}", read_buf);
                println!("{:?}", str::from_utf8(&read_buf).unwrap());
            }
        }
        Ok(())
    }
}
