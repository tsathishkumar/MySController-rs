use serial;
use serial::prelude::*;
use std::io::prelude::*;
use std::str;

use std::sync::mpsc;
use std::time::Duration;

const MAX_TIMEOUT: u64 = 99999999;
const SETTINGS: serial::PortSettings = serial::PortSettings {
    baud_rate: serial::Baud38400,
    char_size: serial::Bits8,
    parity: serial::ParityNone,
    stop_bits: serial::Stop1,
    flow_control: serial::FlowNone,
};

pub struct SerialReader {
    serial_port: String,
    serial_sender: mpsc::Sender<String>,
}

pub struct SerialWriter {
    serial_port: String,
    serial_receiver: mpsc::Receiver<String>,
}

impl SerialReader {
    pub fn new(serial_port: String, serial_sender: mpsc::Sender<String>) -> SerialReader {
        SerialReader {
            serial_port: serial_port,
            serial_sender: serial_sender
        }
    }
    pub fn read(&self) {
        println!("opening port: {}", self.serial_port);
        loop {
        let mut port = serial::open(&self.serial_port).unwrap();
        match self.read_lines(&mut port, &self.serial_sender) {
            _ => continue
        }
        }
    }

    pub fn read_lines<T: SerialPort>(
        &self,
        port: &mut T,
        serial_sender: &mpsc::Sender<String>,
    ) -> serial::Result<()> {
        try!(port.configure(&SETTINGS));
        try!(port.set_timeout(Duration::from_secs(MAX_TIMEOUT)));

        loop {
            // let mut buf_reader = BufReader::new(File::open(&self.serial_port).unwrap());
            // let mut line_buf = String::new();
            // let length = buf_reader.read_line(&mut line_buf).unwrap();
            // if length > 0 {
            //     println!("{:?}", line_buf);
            //     serial_sender.send(line_buf).unwrap();
            // }
            let mut line = String::new();
            let mut buf = [0; 1];
            loop {
                try!(port.read(&mut buf));
                    
                let s = match str::from_utf8(&buf) {
                    Ok(v) => v,
                    Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                };
                line.push_str(&s);
                if s.contains("\n") {
                    break;
                }
            }
            println!("{:?}", line);
            serial_sender.send(line).unwrap();
        }
    }
}

impl SerialWriter {
    pub fn new(serial_port: String, serial_receiver: mpsc::Receiver<String>) -> SerialWriter {
        SerialWriter {
            serial_port: serial_port,
            serial_receiver: serial_receiver
        }
    }

    pub fn write(&self) {
        println!("Writing to {}", self.serial_port);
        let mut port = serial::open(&self.serial_port).unwrap();
        port.configure(&SETTINGS).unwrap();
        port.set_timeout(Duration::from_secs(1)).unwrap();
        loop {
            let received_value: String = self.serial_receiver.recv().unwrap();
            port.write(&received_value.as_bytes()).unwrap();
        }
    }
}
