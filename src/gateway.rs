use serialport;
use serialport::prelude::*;
use std::io;
use std::io::Read;
use std::io::Result;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::str;
use std::sync::mpsc;
use std::time::Duration;

#[derive(Debug, Copy, Clone)]
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

pub trait Gateway: Send {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn clone(&self) -> Box<Gateway>;

    fn write_loop(&mut self, serial_receiver: &mpsc::Receiver<String>) {
        loop {
            match serial_receiver.recv() {
                Ok(received_value) => {
                    self.write(&received_value.as_bytes()).unwrap();
                }
                Err(error) => eprintln!("{:?}", error),
            }
        }
    }

    fn read_loop(&mut self, serial_sender: &mpsc::Sender<String>) {
        loop {
            let mut line = String::new();
            let mut serial_buf: Vec<u8> = vec![0; 1];

            loop {
                match self.read(serial_buf.as_mut_slice()) {
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
}

pub struct SerialGateway {
    pub serial_port: Box<serialport::SerialPort>
}

pub struct TcpGateway {
    pub tcp_port: TcpStream
}

pub fn create_gateway(connection_type: ConnectionType, port: String) -> Box<Gateway> {
    let mut settings: SerialPortSettings = Default::default();
    settings.timeout = Duration::from_millis(10);
    settings.baud_rate = BaudRate::Baud38400;
    println!("connection to {} with type {:?}", port, connection_type);
    match connection_type {
        ConnectionType::Serial => Box::new(SerialGateway { serial_port: serialport::open_with_settings(&port, &settings).unwrap() }),
        ConnectionType::TcpServer => {
            let stream = TcpListener::bind(&port).unwrap();
            let (mut stream, _) = stream.accept().unwrap();
            Box::new(TcpGateway { tcp_port: stream })
        }
        ConnectionType::TcpClient => Box::new(TcpGateway { tcp_port: TcpStream::connect(&port).unwrap() }),
    }
}

impl Gateway for SerialGateway {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.serial_port.read(buf)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.serial_port.write(buf)
    }

    fn clone(&self) -> Box<Gateway> {
        Box::new(SerialGateway { serial_port: self.serial_port.try_clone().unwrap() })
    }
}

impl Gateway for TcpGateway {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.tcp_port.read(buf)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.tcp_port.write(buf)
    }

    fn clone(&self) -> Box<Gateway> {
        Box::new(TcpGateway { tcp_port: self.tcp_port.try_clone().unwrap() })
    }
}