use serialport;
use serialport::prelude::*;
use std::io;
use std::io::Read;
use std::io::Result;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::str;
use channel;
use channel::{Receiver, Sender};
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

pub trait Connection: Send {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn clone(&self) -> Box<Connection>;
    fn port(&self) -> &String;

    fn write_loop(&mut self, receiver: Receiver<String>, stop_receiver: Receiver<String>) -> Receiver<String> {
        loop {
            match stop_receiver.recv_timeout(Duration::from_millis(10)) {
                Ok(_) => break,
                Err(_) => ()
            }
            match receiver.recv_timeout(Duration::from_secs(5)) {
                Ok(received_value) => {
                    match self.write(&received_value.as_bytes()) {
                        Ok(_) => info!("{} << {:?}", self.port(), received_value),
                        Err(e) => {
                            error!("Error while writing -- {:?}", e);
                            break;
                        }
                    }
                }
                Err(channel::RecvTimeoutError::Timeout) => (),
                Err(_error) => error!("Error while receiving -- {:?}", _error),
            }
        }
        (receiver)
    }

    fn read_loop(&mut self, sender: Sender<String>) -> Sender<String> {
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
                            error!("Error while reading -- reached EOF");
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
                        error!("Error while reading -- {:?}", e);
                        broken_connection = true;
                        break;
                    }
                }
            }
            if broken_connection {
                break;
            }
            info!("{} >> {:?}", self.port(), line);
            match sender.send(line) {
                Ok(_) => (),
                Err(_) => break,
            }
        }
        (sender)
    }
}

pub struct SerialConnection {
    pub serial_port: String,
    pub stream: Box<serialport::SerialPort>,
}

pub struct TcpConnection {
    pub tcp_port: String,
    pub tcp_stream: TcpStream,
}

pub fn stream_read_write(stream_info: StreamInfo,
                         mut sender: Sender<String>,
                         mut receiver: Receiver<String>) {
    loop {
        let (cancel_token_sender, cancel_token_receiver) = channel::unbounded();
        let simple_consumer = thread::spawn(move || {
            consume(receiver, cancel_token_receiver)
        });
        let mut read_connection = create_connection(stream_info.connection_type, &stream_info.port);
        cancel_token_sender.send(String::from("stop")).unwrap();
        receiver = simple_consumer.join().unwrap();

        let (cancel_token_sender, cancel_token_receiver) = channel::unbounded();
        let mut write_connection = read_connection.clone();
        let reader = thread::spawn(move || {
            read_connection.read_loop(sender)
        });
        let writer = thread::spawn(move || {
            write_connection.write_loop(receiver, cancel_token_receiver)
        });
        sender = reader.join().unwrap();
        cancel_token_sender.send(String::from("reader stopped")).unwrap();
        receiver = writer.join().unwrap();
    }
}

fn consume(receiver: Receiver<String>, cancel_token_receiver: Receiver<String>) -> Receiver<String> {
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


pub fn create_connection(connection_type: ConnectionType, port: &String) -> Box<Connection> {
    match connection_type {
        ConnectionType::Serial => create_serial_connection(port),
        ConnectionType::TcpClient => {
            let stream: TcpStream;
            info!("Waiting for server connection -- {} ...", port);
            loop {
                stream = match TcpStream::connect(port) {
                    Ok(stream) => stream,
                    Err(_) => {
                        thread::sleep(Duration::from_secs(10));
                        continue;
                    }
                };
                info!("Connected to -- {}", port);
                break;
            }
            Box::new(TcpConnection { tcp_port: port.clone(), tcp_stream: stream })
        }
        ConnectionType::TcpServer => {
            let stream = TcpListener::bind(port).unwrap();
            info!("Server listening on -- {}", port);
            let (mut stream, _socket) = stream.accept().unwrap();
            info!("Accepted connection from {:?}", _socket);
            Box::new(TcpConnection { tcp_port: port.clone(), tcp_stream: stream })
        }
    }
}

fn create_serial_connection(port: &String) -> Box<Connection> {
    let mut settings: SerialPortSettings = Default::default();
    settings.timeout = Duration::from_millis(10);
    settings.baud_rate = BaudRate::Baud38400;
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
    Box::new(SerialConnection { serial_port: port.clone(), stream })
}

impl Connection for SerialConnection {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.stream.read(buf)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.stream.write(buf)
    }

    fn clone(&self) -> Box<Connection> {
        Box::new(SerialConnection { serial_port: self.serial_port.clone(), stream: self.stream.try_clone().unwrap() })
    }

    fn port(&self) -> &String {
        &self.serial_port
    }
}

impl Connection for TcpConnection {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.tcp_stream.read(buf)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.tcp_stream.write(buf)
    }

    fn clone(&self) -> Box<Connection> {
        Box::new(TcpConnection { tcp_port: self.tcp_port.clone(), tcp_stream: self.tcp_stream.try_clone().unwrap() })
    }

    fn port(&self) -> &String {
        &self.tcp_port
    }
}