use serialport::prelude::*;

use std::io;
use std::io::prelude::*;
use std::str;

use std::sync::mpsc;

pub fn write(port: &mut Box<SerialPort>, serial_receiver: &mpsc::Receiver<String>) {
    loop {
        let received_value: String = serial_receiver.recv().unwrap();
        port.write(&received_value.as_bytes()).unwrap();
    }
}

pub fn read(port: &mut Box<SerialPort>, serial_sender: &mpsc::Sender<String>) {
    loop {
        let mut line = String::new();
        let mut serial_buf: Vec<u8> = vec![0; 1];

        loop {
            match port.read(serial_buf.as_mut_slice()) {
                Ok(_t) => {
                    let s = match str::from_utf8(&serial_buf) {
                        Ok(v) => v,
                        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                    };
                    line.push_str(&s);
                    if s.contains("\n") {
                        break;
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                Err(e) => eprintln!("{:?}", e),
            }
        }
        println!("{:?}", line);
        serial_sender.send(line).unwrap();
    }
}
