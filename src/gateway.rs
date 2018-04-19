use serialport;
use serialport::prelude::*;
use std::io;
use std::io::Read;
use std::io::Result;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::str;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;
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

    fn write_loop(&mut self, stop_thread: Arc<Mutex<bool>>, serial_receiver: &mpsc::Receiver<String>) {
        loop {
            if *stop_thread.lock().unwrap() {
                break;
            }
            match serial_receiver.recv() {
                Ok(received_value) => {
                    match self.write(&received_value.as_bytes()) {
                        Ok(_) => (),
                        Err(e) => {
                            eprintln!("Error while writing -- {:?}", e);
                        }
                    }
                }
                Err(error) => eprintln!("Error while receiving -- {:?}", error),
            }
        }
    }

    fn read_loop(&mut self, stop_thread: Arc<Mutex<bool>>, serial_sender: &mpsc::Sender<String>) {
        loop {
            if *stop_thread.lock().unwrap() {
                break;
            }
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
                    Err(e) => {
                        panic!("Error while reading -- {:?}", e);
                    }
                }
            }
            println!("{:?}", line);
            serial_sender.send(line).unwrap();
        }
    }
}

pub struct SerialGateway {
    pub serial_port: String,
    pub stream: Box<serialport::SerialPort>,
}

pub struct TcpGateway {
    pub tcp_port: String,
    pub tcp_stream: TcpStream,
}

pub fn create_gateway(connection_type: ConnectionType, port: &String) -> Box<Gateway> {
    match connection_type {
        ConnectionType::Serial => create_serial_gateway(port),
        ConnectionType::TcpClient => {
            let stream: TcpStream;
            loop {
                println!("Connecting to server -- {}", port);
                stream = match TcpStream::connect(port) {
                    Ok(stream) => stream,
                    Err(_) => {
                        thread::sleep(Duration::from_secs(10));
                        continue;
                    }
                };
                break;
            }
            Box::new(TcpGateway { tcp_port: port.clone(), tcp_stream: stream })
        }
        ConnectionType::TcpServer => {
            let stream = TcpListener::bind(port).unwrap();
            println!("Server listening on -- {}", port);
            let (mut stream, _socket) = stream.accept().unwrap();
            println!("Accepted connection from {:?}", _socket);
            Box::new(TcpGateway { tcp_port: port.clone(), tcp_stream: stream })
        }
    }
}

fn create_serial_gateway(port: &String) -> Box<Gateway> {
    let mut settings: SerialPortSettings = Default::default();
    settings.timeout = Duration::from_millis(10);
    settings.baud_rate = BaudRate::Baud38400;
    Box::new(SerialGateway { serial_port: port.clone(), stream: serialport::open_with_settings(&port, &settings).unwrap() })
}

impl Gateway for SerialGateway {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.stream.read(buf)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.stream.write(buf)
    }

    fn clone(&self) -> Box<Gateway> {
        Box::new(SerialGateway { serial_port: self.serial_port.clone(), stream: self.stream.try_clone().unwrap() })
    }
}

impl Gateway for TcpGateway {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.tcp_stream.read(buf)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.tcp_stream.write(buf)
    }

    fn clone(&self) -> Box<Gateway> {
        Box::new(TcpGateway { tcp_port: self.tcp_port.clone(),  tcp_stream: self.tcp_stream.try_clone().unwrap() })
    }
}