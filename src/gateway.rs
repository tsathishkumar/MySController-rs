use std::io;
use std::str;

use std::sync::mpsc;

pub fn write<W: io::Write>(port: &mut W, serial_receiver: &mpsc::Receiver<String>) {
    loop {
        match serial_receiver.recv() {
            Ok(received_value) => {
                port.write(&received_value.as_bytes()).unwrap();
            }
            Err(error) => eprintln!("{:?}", error),
        }
    }
}

pub fn read<R: io::Read>(port: &mut R, serial_sender: &mpsc::Sender<String>) {
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
