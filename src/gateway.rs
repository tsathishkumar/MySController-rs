use std::io;
use std::str;

use serialport;
use serialport::prelude::*;
use std::io::Read;
use std::io::Result;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::time::Duration;

#[derive(Copy, Clone)]
pub enum ConnectionType {
    Serial,
    TcpServer,
    TcpClient,
}

impl ConnectionType {
    pub fn from_str(s: &str, server: bool) -> Option<ConnectionType> {
        match s {
            "SERIAL" => Some(ConnectionType::Serial),
            "TCP" => if server {
                Some(ConnectionType::TcpServer)
            } else {
                Some(ConnectionType::TcpClient)
            },
            _ => None,
        }
    }
}

pub struct Gateway {
    pub serial_port: Option<Box<serialport::SerialPort>>,
    pub tcp_port: Option<TcpStream>,
}
impl Gateway {
    pub fn clone(&self) -> Gateway {
        Gateway {
            serial_port: match self.serial_port {
                Some(ref port) => Some(port.try_clone().unwrap()),
                _ => None,
            },
            tcp_port: match self.tcp_port {
                Some(ref port) => Some(port.try_clone().unwrap()),
                _ => None,
            },
        }
    }

    pub fn new(connection_type: ConnectionType, port: String) -> Gateway {
        let mut settings: SerialPortSettings = Default::default();
        settings.timeout = Duration::from_millis(10);
        settings.baud_rate = BaudRate::Baud38400;
        Gateway {
            serial_port: match connection_type {
                ConnectionType::Serial => {
                    Some(serialport::open_with_settings(&port, &settings).unwrap())
                }
                _ => None,
            },
            tcp_port: match connection_type {
                ConnectionType::TcpServer => {
                    let stream = TcpListener::bind(&port).unwrap();
                    let (mut stream, _) = stream.accept().unwrap();
                    Some(stream)
                }
                ConnectionType::TcpClient => Some(TcpStream::connect(&port).unwrap()),
                _ => None,
            },
        }
    }
}

impl Gateway {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self.tcp_port {
            Some(ref mut stream) => stream.read(buf),
            None => match self.serial_port {
                Some(ref mut port) => port.read(buf),
                _ => Ok(0),
            },
        }
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match self.tcp_port {
            Some(ref mut stream) => stream.write(buf),
            None => match self.serial_port {
                Some(ref mut port) => port.write(buf),
                _ => Ok(0),
            },
        }
    }
}

pub fn write(port: &mut Gateway, serial_receiver: &mpsc::Receiver<String>) {
    loop {
        match serial_receiver.recv() {
            Ok(received_value) => {
                port.write(&received_value.as_bytes()).unwrap();
            }
            Err(error) => eprintln!("{:?}", error),
        }
    }
}

pub fn read(port: &mut Gateway, serial_sender: &mpsc::Sender<String>) {
    loop {
        let mut line = String::new();
        let mut serial_buf: Vec<u8> = vec![0; 1];

        loop {
            match port.read(serial_buf.as_mut_slice()) {
                Ok(_t) => {
                    let s = match str::from_utf8(&serial_buf) {
                        Ok(v) => v,
                        Err(_e) => break,
                    };
                    line.push_str(&s);
                    if s.contains("\n") {
                        break;
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                Err(e) => panic!("{:?}", e),
            }
        }
        println!("{:?}", line);
        serial_sender.send(line).unwrap();
    }
}
