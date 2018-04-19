use serialport;
use serialport::prelude::*;
use std::io;
use std::thread;
use std::io::{Error, ErrorKind};
use std::io::Read;
use std::io::Result;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::str;
use std::sync::{Arc, Mutex};
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

pub trait Gateway: Send + Sync {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn clone(&self) -> Box<Gateway>;
    fn connect(&mut self);

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
                        },
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
                        eprintln!("Error while reading -- {:?}", e);
                        self.connect();
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
    pub tcp_stream: Option<TcpStream>,
}

pub struct TcpServerGateway {
    pub tcp_port: String,
    pub tcp_listener: TcpListener,
    pub tcp_stream: Option<Arc<Mutex<TcpStream>>>,
}

pub struct ReadWriteGateway {
    pub gateway: Arc<Box<Gateway>>,
}

impl ReadWriteGateway {
    pub fn read_write_loop(&self, stop_thread: Arc<Mutex<bool>>, serial_sender: &mpsc::Sender<String>) {
        let mut read_gateway = self.gateway.clone();
        let mut write_gateway = self.gateway.clone();
        let stop_thread1 = stop_thread.clone();
        let sender = serial_sender.clone();
        thread::spawn(move || {
            read_gateway.read_loop(stop_thread1, &sender);
        });

    }
}

pub fn create_gateway(connection_type: ConnectionType, port: &String) -> Box<Gateway> {
    println!("connection to {} with type {:?}", port, connection_type);
    match connection_type {
        ConnectionType::Serial => create_serial_gateway(port),
        ConnectionType::TcpClient => {
            Box::new(TcpGateway { tcp_port: port.clone(), tcp_stream: None })
        }
        ConnectionType::TcpServer => {
            let tcp_listener = TcpListener::bind(port).unwrap();
            Box::new(TcpServerGateway { tcp_port: port.clone(), tcp_listener, tcp_stream: None })
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
    fn connect(&mut self) {
        let mut settings: SerialPortSettings = Default::default();
        settings.timeout = Duration::from_millis(10000);
        settings.baud_rate = BaudRate::Baud38400;
        loop {
            self.stream = match serialport::open_with_settings(&self.serial_port, &settings) {
                Ok(serial_port) => serial_port,
                Err(_) => continue,
            };
            break;
        }
    }
}

impl Gateway for TcpServerGateway {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self.tcp_stream {
            Some(ref mut stream) => stream.read(buf),
            None => Err(Error::from(ErrorKind::NotConnected)),
        }
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match self.tcp_stream {
            Some(ref mut stream) => stream.write(buf),
            None => Err(Error::from(ErrorKind::NotConnected)),
        }
    }

    fn clone(&self) -> Box<Gateway> {
        let clone_stream = match self.tcp_stream {
            Some(ref stream) => Some(stream.try_clone().unwrap()),
            None => None,
        };
        Box::new(TcpServerGateway {
            tcp_port: self.tcp_port.clone(),
            tcp_listener: self.tcp_listener.try_clone().unwrap(),
            tcp_stream: clone_stream,
        })
    }
    fn connect(&mut self) {
        println!("Waiting for clients to connect to -- {}", self.tcp_port);
        self.tcp_stream = match self.tcp_listener.accept() {
            Ok((_stream, _socket)) => {
                println!("Connected to client -- {:?}", _socket);
                Some(Arc::new(Mutex::new(_stream)))
            },
            Err(_) => None,
        };
    }
}

impl Gateway for TcpGateway {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self.tcp_stream {
            Some(ref mut stream) => stream.read(buf),
            None => Err(Error::from(ErrorKind::NotConnected)),
        }
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match self.tcp_stream {
            Some(ref mut stream) => stream.write(buf),
            None => Err(Error::from(ErrorKind::NotConnected)),
        }
    }

    fn clone(&self) -> Box<Gateway> {
        let clone_stream = match self.tcp_stream {
            Some(ref stream) => Some(stream.try_clone().unwrap()),
            None => None,
        };
        Box::new(TcpGateway {
            tcp_port: self.tcp_port.clone(),
            tcp_stream: clone_stream,
        })
    }

    fn connect(&mut self) {
        loop {
            println!("Connecting server to -- {}", self.tcp_port);
            let stream = TcpStream::connect(&self.tcp_port);
            self.tcp_stream = match stream {
                Ok(_stream) => Some(_stream),
                Err(_) => {
                    thread::sleep(Duration::from_millis(5000));
                    continue;
                },
            };
            println!("Connected to -- {}", self.tcp_port);
            break;
        }
    }
}