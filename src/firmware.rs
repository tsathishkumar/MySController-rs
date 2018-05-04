use crc16::*;
use ihex::record::Record;
use std::fs::{File};
use std::io::BufReader;
use std::io::prelude::*;
use std::iter::FromIterator;
use std::path::{Path};

const FIRMWARE_BLOCK_SIZE: i32 = 16;

table! {
    firmwares (firmware_type, firmware_version) {
        firmware_type -> Integer,
        firmware_version -> Integer,
        name -> Text,
        blocks -> Integer,
        crc -> Integer,
        data -> Binary,
    }
}

#[derive(Debug, Queryable, Serialize, Deserialize, Insertable)]
#[table_name = "firmwares"]
pub struct Firmware {
    pub firmware_type: i32,
    pub firmware_version: i32,
    pub name: String,
    pub blocks: i32,
    pub crc: i32,
    pub data: Vec<u8>,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct FirmwareKey {
    pub _type: u16,
    pub version: u16,
}

impl Firmware {
    pub fn new(firmware_type: i32, firmware_version: i32, blocks: i32, data: Vec<u8>, name: String) -> Firmware {
        Firmware {
            firmware_type,
            firmware_version,
            name,
            blocks,
            crc: Firmware::compute_crc(&data) as i32,
            data,
        }
    }

    pub fn get_block(&self, block: u16) -> [u8; 16] {
        let start_index: usize = (block * 16) as usize;
        if start_index > self.data.len() {
            let no_binary: [u8; 16] = [0; 16];
            return no_binary;
        }
        let v = Vec::from_iter(
            self.data[start_index..(start_index + 16) as usize]
                .iter()
                .cloned(),
        );
        let mut block = [0u8; 16];
        for (place, element) in block.iter_mut().zip(v.iter()) {
            *place = *element;
        }
        block
    }

    pub fn ihex_to_bin(record: &Record) -> Vec<u8> {
        match record {
            &Record::Data {
                offset: _,
                value: ref _value,
            } => _value.clone(),
            _ => Vec::new(),
        }
    }

    pub fn prepare_fw(_type: i32, version: i32, name: String, path: &Path) -> Firmware {
        let f = File::open(path).unwrap();
        let f = BufReader::new(f);
        let mut data: Vec<u8> = f.lines()
            .flat_map(|line| Firmware::ihex_to_bin(&Record::from_record_string(&line.unwrap()).unwrap()))
            .collect();
        let pads: usize = data.len() % 128; // 128 bytes per page for atmega328
        for _ in 0..(128 - pads) {
            data.push(255);
        }
        let blocks: i32 = data.len() as i32 / FIRMWARE_BLOCK_SIZE;
        Firmware::new(_type, version, blocks, data, name)
    }

    fn compute_crc(data: &[u8]) -> u16 {
        let mut state = State::<MODBUS>::new();
        state.update(data);
        state.get()
    }
}

#[cfg(test)]
mod test {
    use hex;
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn reader_respects_all_newline_formats() {
        let input = String::new() + &":100490008B002097E1F30E940000F9CF0895F894B3";

        assert_eq!(
            String::from("8B002097E1F30E940000F9CF0895F894"),
            hex::encode_upper(Firmware::ihex_to_bin(&Record::from_record_string(&input).unwrap()))
        );
    }

    #[test]
    fn hex_file_to_vector() {
        let fw_binary = Firmware::prepare_fw(10, 2, String::from("Blink"), &PathBuf::from("firmwares/10__2__Blink.ino.hex"));
        assert_eq!(fw_binary.data.len(), 1280);
    }

    #[test]
    fn extract_given_block_from_binary_data() {
        let fw_binary = Firmware::prepare_fw(10, 2, String::from("Blink"), &PathBuf::from("firmwares/10__2__Blink.ino.hex"));
        assert_eq!(fw_binary.get_block(1), [
            12, 148, 110, 0, 12, 148, 110, 0, 12, 148, 110, 0, 12, 148, 110, 0
        ]);
    }

    #[test]
    fn compute_correct_crc() {
        let fw_binary = Firmware::prepare_fw(10, 2, String::from("Blink"), &PathBuf::from("firmwares/10__2__Blink.ino.hex"));
        assert_eq!(Firmware::compute_crc(&fw_binary.data), 0x46D4);
    }
}
