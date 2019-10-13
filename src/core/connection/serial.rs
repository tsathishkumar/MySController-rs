use serialport;
use serialport::prelude::*;
use std::time::Duration;
use std::io::Result;
use std::thread;
use super::StreamConnection;

pub struct SerialConnection {
    serial_port: String,
    stream: Box<dyn serialport::SerialPort>,
}

impl StreamConnection for SerialConnection {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.stream.read(buf)
    }

    fn port(&self) -> &String {
        &self.serial_port
    }

    fn write_line(&mut self, line: &str) -> Result<usize> {
        self.stream.write(&String::from(line).as_bytes())
    }

    fn timeout(&mut self, duration: Duration) {
        match self.stream.set_timeout(duration) {
            Ok(_) => (),
            Err(_) => error!(
                "Error while setting timeout for Serial connection {:?}",
                &self.serial_port
            ),
        }
    }

    fn clone(&self) -> Box<dyn super::Connection> {
        Box::new(SerialConnection {
            serial_port: self.serial_port.clone(),
            stream: self.stream.try_clone().unwrap(),
        })
    }
}

impl SerialConnection {
    pub fn new(port: &str, baud_rate: u32) -> SerialConnection {
        let mut settings: SerialPortSettings = Default::default();
        settings.timeout = Duration::from_millis(10);
        settings.baud_rate =BaudRate::from(baud_rate);
        let stream;
        info!("Waiting for serial connection in -- {} ...", port);
        loop {
            stream = match serialport::open_with_settings(&port, &settings) {
                Ok(stream) => stream,
                Err(_) => {
                    thread::sleep(Duration::from_secs(10));
                    continue;
                }
            };
            info!("Connected to -- {}", port);
            break;
        }
        SerialConnection {
            serial_port: port.to_string(),
            stream,
        }
    }
}