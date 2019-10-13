use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::Read;
use std::io::Result;
use std::io::Write;
use std::time::Duration;
use super::{Connection, StreamConnection};

pub struct TcpConnection {
    tcp_port: String,
    tcp_stream: TcpStream,
}

impl StreamConnection for TcpConnection {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.tcp_stream.read(buf)
    }

    fn port(&self) -> &String {
        &self.tcp_port
    }

    fn timeout(&mut self, duration: Duration) {
        match self.tcp_stream.set_read_timeout(Some(duration)) {
            Ok(_) => (),
            Err(_) => error!(
                "Error while setting timeout for TCP connection {:?}",
                &self.tcp_port
            ),
        }
    }

    fn write_line(&mut self, line: &str) -> Result<usize> {
        self.tcp_stream.write(&String::from(line).as_bytes())
    }

    fn clone(&self) -> Box<dyn Connection> {
        Box::new(TcpConnection {
            tcp_port: self.tcp_port.clone(),
            tcp_stream: self.tcp_stream.try_clone().unwrap(),
        })
    }
}

impl TcpConnection {
    pub fn new(port: String, timeout_enabled: bool) -> TcpConnection {
        let stream: TcpStream;
        info!("Waiting for server connection -- {} ...", port);
        loop {
            stream = match TcpStream::connect(port.clone()) {
                Ok(stream) => stream,
                Err(_) => {
                    thread::sleep(Duration::from_secs(10));
                    continue;
                }
            };
            info!("Connected to -- {}", port);
            break;
        }
        let mut connection = TcpConnection {
            tcp_port: port.to_string(),
            tcp_stream: stream,
        };
        if timeout_enabled {
            Connection::timeout(&mut connection, Duration::from_secs(40));
        }
        connection
    }

    pub fn new_server(port: String, timeout_enabled: bool) -> TcpConnection{
        let stream = TcpListener::bind(port.clone()).unwrap();
            info!("Server listening on -- {}", port.as_str());
            let (stream, _socket) = stream.accept().unwrap();
            info!("Accepted connection from {:?}", _socket);
            let mut connection = TcpConnection {
                tcp_port: port.clone(),
                tcp_stream: stream,
            };
            if timeout_enabled {
                Connection::timeout(&mut connection, Duration::from_secs(40));
            }
            connection
    }
}