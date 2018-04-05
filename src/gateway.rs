use serial;
use serial::prelude::*;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use std::sync::mpsc;
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
    pub fn new(serial_port: String) -> SerialGateway {
        SerialGateway {
            serial_port: serial_port,
        }
    }
    pub fn read(&self, serial_sender: mpsc::Sender<String>) {
        println!("opening port: {}", self.serial_port);
        let mut port = serial::open(&self.serial_port).unwrap();
        self.read_lines(&mut port, serial_sender).unwrap();
    }

    pub fn write(&self, serial_receiver: mpsc::Receiver<String>) {
        println!("Writing to {}", self.serial_port);
        let mut port = serial::open(&self.serial_port).unwrap();
        port.configure(&SETTINGS).unwrap();
        port.set_timeout(Duration::from_secs(1)).unwrap();
        loop {
            let received_value: String = serial_receiver.recv().unwrap();
            port.write(&received_value.as_bytes()).unwrap();
        }
    }

    pub fn read_lines<T: SerialPort>(
        &self,
        port: &mut T,
        serial_sender: mpsc::Sender<String>,
    ) -> serial::Result<()> {
        try!(port.configure(&SETTINGS));
        try!(port.set_timeout(Duration::from_secs(1)));

        // port.write(&String::from("test").as_bytes()).unwrap();

        loop {
            let mut buf_reader = BufReader::new(File::open(&self.serial_port).unwrap());
            let mut line_buf = String::new();
            let length = buf_reader.read_line(&mut line_buf).unwrap();
            if length > 0 {
                println!("{:?}", line_buf);
                serial_sender.send(line_buf).unwrap();
            }
        }
    }
}
