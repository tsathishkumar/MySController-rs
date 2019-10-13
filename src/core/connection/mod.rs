pub mod tcp;
pub mod serial;
pub mod mqtt;

use std::io;
use std::io::{Result, Error, ErrorKind};

use std::str;
use std::thread;
use std::time::Duration;

use crate::channel;
use crate::channel::{Receiver, Sender};

#[derive(Debug, Clone)]
pub enum ConnectionType {
    Serial{ port: String, baud_rate: u32},
    TcpServer{port: String, timeout_enabled: bool},
    TcpClient{port: String, timeout_enabled: bool},
    MQTT{broker: String, port: u16, publish_topic_prefix: String},
}

impl ConnectionType {

}

pub trait Connection: Send {
    fn timeout(&mut self, duration: Duration);
    fn read_line(&mut self) -> Result<String>;
    fn write_line(&mut self, line: &str) -> Result<usize>;
    fn clone(&self) -> Box<dyn Connection>;
    fn host(&self) -> &String;

    fn write_loop(
        &mut self,
        receiver: Receiver<String>,
        stop_receiver: Receiver<String>,
    ) -> Receiver<String> {
        loop {
            if stop_receiver.recv_timeout(Duration::from_millis(10)).is_ok() {
                break;
            }
            match receiver.recv_timeout(Duration::from_secs(5)) {
                Ok(received_value) => match self.write_line(received_value.as_str()) {
                    Ok(_) => info!("{} << {:?}", self.host(), received_value),
                    Err(e) => {
                        error!("Error while writing -- {:?}", e);
                        break;
                    }
                },
                Err(channel::RecvTimeoutError::Timeout) => (),
                Err(_error) => error!("Error while receiving -- {:?}", _error),
            }
        }
        (receiver)
    }

    fn read_loop(&mut self, message_sender: Sender<String>) -> Sender<String> {
        self.timeout(Duration::from_secs(30));

        while let Ok(line) = self.read_line() {
            info!("{} >> {:?}", self.host(), line);
            match message_sender.send(line) {
                Ok(_) => (),
                Err(_) => break,
            }
        }
        (message_sender)
    }

    fn health_check(&mut self, stop_check_receiver: Receiver<String>) {
        loop {
            match self.write_line("0;255;3;0;2;") {
                Ok(_) => info!("{} << 0;255;3;0;2;", self.host()),
                Err(e) => {
                    error!("Error while writing -- {:?}", e);
                    break;
                }
            }
            if stop_check_receiver.recv_timeout(Duration::from_secs(10)).is_ok() {
                break;
            }
            thread::sleep(Duration::from_secs(30))
        }
    }
}

pub trait StreamConnection: Connection {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn port(&self) -> &String;
    fn timeout(&mut self, duration: Duration);
    fn write_line(&mut self, line: &str) -> Result<usize>;
    fn clone(&self) -> Box<dyn Connection>;
}

impl<T> Connection for T where T: StreamConnection {
    fn timeout(&mut self, duration: Duration) {
        StreamConnection::timeout(self, duration)
    }

    fn write_line(&mut self, line: &str) -> Result<usize> {
        StreamConnection::write_line(self, line)
    }

    fn clone(&self) -> Box<dyn Connection> {
        StreamConnection::clone(self)
    }

    fn host(&self) -> &String {
        self.port()
    }

    fn read_line(&mut self) -> Result<String> {

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
                        return Err(Error::new(ErrorKind::ConnectionAborted, "Error while reading -- reached EOF"));
                    }
                    line.push_str(&s);
                    if s.contains('\n') {
                        break;
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                Err(e) => {
                    error!("Error while reading -- {:?}", e);
                    return Result::Err(Error::new(ErrorKind::Other, e));
                }
            }
        }
        Ok(line)
    }
}

pub fn stream_read_write(
    stream_info: ConnectionType,
    mut sender: Sender<String>,
    mut receiver: Receiver<String>,
) {
    loop {
        let (cancel_token_sender, cancel_token_receiver) = channel::unbounded();
        let (stop_check_sender, stop_check_receiver) = channel::unbounded();
        let simple_consumer = thread::spawn(move || consume(receiver, cancel_token_receiver));
        let mut read_connection = create_connection(stream_info.clone());
        
        cancel_token_sender.send(String::from("stop")).unwrap();
        receiver = simple_consumer.join().unwrap();

        let (cancel_token_sender, cancel_token_receiver) = channel::unbounded();
        let mut write_connection = read_connection.clone();
        let mut health_check_connection = read_connection.clone();
        thread::spawn(move || health_check_connection.health_check(stop_check_receiver));
        let reader = thread::spawn(move || read_connection.read_loop(sender));
        let writer =
            thread::spawn(move || write_connection.write_loop(receiver, cancel_token_receiver));
        sender = reader.join().unwrap();
        let stop_token = String::from("reader stopped");
        stop_check_sender.send(stop_token.clone()).unwrap();
        cancel_token_sender.send(stop_token).unwrap();
        receiver = writer.join().unwrap();
    }
}

fn consume(
    receiver: Receiver<String>,
    cancel_token_receiver: Receiver<String>,
) -> Receiver<String> {
    loop {
        if cancel_token_receiver.recv_timeout(Duration::from_millis(500)).is_ok() {
            break;
        }
        match receiver.recv_timeout(Duration::from_millis(500)) {
            _ => continue,
        }
    }
    receiver
}

pub fn create_connection(
    connection_type: ConnectionType) -> Box<dyn Connection> {
    match connection_type {
        ConnectionType::Serial{port, baud_rate} => Box::new(serial::SerialConnection::new(port.as_str(), baud_rate)),
        ConnectionType::TcpClient{port, timeout_enabled} => 
            Box::new(tcp::TcpConnection::new(port, timeout_enabled)),
        ConnectionType::TcpServer{port, timeout_enabled} => 
            Box::new(tcp::TcpConnection::new_server(port, timeout_enabled)),
        ConnectionType::MQTT{broker, port, publish_topic_prefix} => Box::new(mqtt::MqttConnection::new(broker, port, publish_topic_prefix, "myscontroller-read"))
    }
}

