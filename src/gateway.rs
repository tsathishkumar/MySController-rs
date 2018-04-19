use serialport;
use serialport::prelude::*;
use std::io;
use std::io::Read;
use std::io::Result;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::str;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub struct StreamInfo {
    pub port: String,
    pub connection_type: ConnectionType,
}

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

    fn write_loop(&mut self, serial_receiver: mpsc::Receiver<String>, stop_receiver: mpsc::Receiver<String>) -> mpsc::Receiver<String> {
        loop {
            match stop_receiver.recv_timeout(Duration::from_millis(10)) {
                Ok(_) => break,
                Err(_) => ()
            }
            match serial_receiver.recv_timeout(Duration::from_secs(5)) {
                Ok(received_value) => {
                    match self.write(&received_value.as_bytes()) {
                        Ok(_) => (),
                        Err(e) => {
                            eprintln!("Error while writing -- {:?}", e);
                            break;
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => (),
                Err(_error) => eprintln!("Error while receiving -- {:?}", _error),
            }
        }
        (serial_receiver)
    }

    fn read_loop(&mut self, serial_sender: mpsc::Sender<String>) -> mpsc::Sender<String> {
        loop {
            let mut broken_connection = false;
            let mut line = String::new();
            let mut serial_buf: Vec<u8> = vec![0; 1];

            loop {
                match self.read(serial_buf.as_mut_slice()) {
                    Ok(_t) => {
                        let s = match str::from_utf8(&serial_buf) {
                            Ok(v) => v,
                            Err(_e) => break,
                        };
                        if s == "\u{0}" {
                            println!("Error while reading -- reached EOF");
                            broken_connection = true;
                            break;
                        }
                        line.push_str(&s);
                        if s.contains("\n") {
                            break;
                        }
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => {
                        println!("Error while reading -- {:?}", e);
                        broken_connection = true;
                        break;
                    }
                }
            }
            if broken_connection {
                break;
            }
            println!("{:?}", line);
            serial_sender.send(line).unwrap();
        }
        (serial_sender)
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

pub fn stream_read_write(stream_info: StreamInfo,
                         mut sender: mpsc::Sender<String>,
                         mut receiver: mpsc::Receiver<String>) {
    loop {
        let (cancel_token_sender, cancel_token_receiver) = mpsc::channel();
        let simple_consumer = thread::spawn(move || {
            consume(receiver, cancel_token_receiver)
        });
        let mut mys_gateway_reader = create_gateway(stream_info.connection_type, &stream_info.port);
        cancel_token_sender.send(String::from("stop")).unwrap();
        receiver = simple_consumer.join().unwrap();

        let (cancel_token_sender, cancel_token_receiver) = mpsc::channel();
        let mut mys_gateway_writer = mys_gateway_reader.clone();
        let gateway_reader = thread::spawn(move || {
            mys_gateway_reader.read_loop(sender)
        });
        let gateway_writer = thread::spawn(move || {
            mys_gateway_writer.write_loop(receiver, cancel_token_receiver)
        });
        sender = gateway_reader.join().unwrap();
        cancel_token_sender.send(String::from("reader stoppped")).unwrap();
        receiver = gateway_writer.join().unwrap();
    }
}

fn consume(receiver: mpsc::Receiver<String>, cancel_token_receiver: mpsc::Receiver<String>) -> mpsc::Receiver<String> {
    loop {
        match cancel_token_receiver.recv_timeout(Duration::from_millis(500)) {
            Ok(_) => break,
            Err(_) => ()
        }
        match receiver.recv_timeout(Duration::from_millis(500)) {
            _ => continue,
        }
    }
    receiver
}


pub fn create_gateway(connection_type: ConnectionType, port: &String) -> Box<Gateway> {
    match connection_type {
        ConnectionType::Serial => create_serial_gateway(port),
        ConnectionType::TcpClient => {
            let stream: TcpStream;
            loop {
                println!("Waiting for server connection -- {} ...", port);
                stream = match TcpStream::connect(port) {
                    Ok(stream) => stream,
                    Err(_) => {
                        thread::sleep(Duration::from_secs(10));
                        continue;
                    }
                };
                println!("Connected to -- {}", port);
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
        Box::new(TcpGateway { tcp_port: self.tcp_port.clone(), tcp_stream: self.tcp_stream.try_clone().unwrap() })
    }
}