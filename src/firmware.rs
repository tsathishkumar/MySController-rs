use crc16::*;
use ihex::record::Record;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::iter::FromIterator;

const FIRMWARE_BLOCK_SIZE: u16 = 16;

#[derive(Debug)]
pub struct FirmwareRepo {
    pub firmware_map: HashMap<FirmwareKey, Firmware>,
}

impl FirmwareRepo {
    pub fn new() -> FirmwareRepo {
        FirmwareRepo { firmware_map: FirmwareRepo::populate_firmwares() }
    }

    pub fn get_firmware(&self, _type: u16, version: u16) -> Result<&Firmware, String> {
        let firmware_key = FirmwareKey { _type, version };
        match self.firmware_map.get(&firmware_key) {
            Some(firmware) => Ok(firmware),
            None => Err(format!("Firmware not found with type {}, version {}", _type, version)),
        }
    }

    fn populate_firmwares() -> HashMap<FirmwareKey, Firmware> {
        let mut firmware_map = HashMap::new();
        let firmwares_directory = "firmwares/";
        let paths = fs::read_dir(firmwares_directory)
            .expect("Place the firmwares under directory named 'firmwares' in the server root directory");
        for path in paths {
            let _path = path.unwrap();
            let file_name = _path.file_name().into_string().unwrap();
            println!("Loading firmware: {:?}", file_name);
            let file_name_parts = file_name.trim().split("__").collect::<Vec<&str>>();
            if file_name_parts.len() != 3 {
                panic!("Invalid filename for firmware. It should follow the convention type__version__firmwarename.hex Example: `10__2__blink.ino.hex`.");
            }
            let firmware_type = file_name_parts[0].parse::<u16>().expect("Firmware type should be a a number");
            let version = file_name_parts[1].parse::<u16>().expect("Firmware type should be a a number");
            let firmware = Firmware::prepare_fw(firmware_type, version, format!("{}{}", firmwares_directory, file_name));
            firmware_map.insert(FirmwareKey { _type: firmware_type, version }, firmware);
        }
        firmware_map
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct FirmwareKey {
    pub _type: u16,
    pub version: u16,
}

#[derive(Debug)]
pub struct Firmware {
    pub _type: u16,
    pub version: u16,
    pub blocks: u16,
    pub crc: u16,
    pub bin_data: Vec<u8>,
    pub name: String,
}

impl Firmware {
    pub fn new(_type: u16, version: u16, blocks: u16, bin_data: Vec<u8>, name: String) -> Firmware {
        Firmware {
            _type,
            version,
            blocks,
            crc: Firmware::compute_crc(&bin_data),
            bin_data,
            name,
        }
    }

    pub fn get_block(&self, block: u16) -> [u8; 16] {
        let start_index: usize = (block * 16) as usize;
        if start_index > self.bin_data.len() {
            let no_binary: [u8; 16] = [0; 16];
            return no_binary;
        }
        let v = Vec::from_iter(
            self.bin_data[start_index..(start_index + 16) as usize]
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

    pub fn prepare_fw(_type: u16, version: u16, file_name: String) -> Firmware {
        let f = File::open(file_name.clone()).unwrap();
        let f = BufReader::new(f);
        let mut bin_data: Vec<u8> = f.lines()
            .flat_map(|line| Firmware::ihex_to_bin(&Record::from_record_string(&line.unwrap()).unwrap()))
            .collect();
        let pads: usize = bin_data.len() % 128; // 128 bytes per page for atmega328
        for _ in 0..(128 - pads) {
            bin_data.push(255);
        }
        let blocks: u16 = bin_data.len() as u16 / FIRMWARE_BLOCK_SIZE;
        Firmware::new(_type, version, blocks, bin_data, file_name.clone())
    }

    fn compute_crc(bin_data: &[u8]) -> u16 {
        let mut state = State::<MODBUS>::new();
        state.update(bin_data);
        state.get()
    }
}


#[cfg(test)]
mod test {

    use super::*;
    use hex;

    #[test]
    fn populate_all_firmwares_available() {
        let repo = FirmwareRepo::new();
        assert_eq!(repo.firmware_map.len(), 1);
    }

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
        let fw_binary = Firmware::prepare_fw(10, 2, String::from("firmwares/10__2__Blink.ino.hex"));
        assert_eq!(fw_binary.bin_data.len(), 1280);
    }

    #[test]
    fn extract_given_block_from_binary_data() {
        let fw_binary = Firmware::prepare_fw(10, 2, String::from("firmwares/10__2__Blink.ino.hex"));
        assert_eq!(fw_binary.get_block(1), [
            12, 148, 110, 0, 12, 148, 110, 0, 12, 148, 110, 0, 12, 148, 110, 0
        ]);
    }

    #[test]
    fn compute_correct_crc() {
        let fw_binary = Firmware::prepare_fw(10, 2, String::from("firmwares/10__2__Blink.ino.hex"));
        assert_eq!(Firmware::compute_crc(&fw_binary.bin_data), 0x46D4);
    }
}
