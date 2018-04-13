use message::{CommandMessage, CommandSubType};

use ihex::record::Record;

use crc16::*;

use std::sync::mpsc;

use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::iter::FromIterator;

const FIRMWARE_BLOCK_SIZE: u16 = 16;

#[derive(Debug)]
pub struct Firmware {
  pub _type: u16,
  pub version: u16,
  pub blocks: u16,
  pub crc: u16,
  pub bin_data: Vec<u8>,
}

impl Firmware {
  pub fn new(_type: u16, version: u16, blocks: u16, crc: u16, bin_data: Vec<u8>) -> Firmware {
    Firmware {
      _type: _type,
      version: version,
      blocks: blocks,
      crc: crc,
      bin_data: bin_data,
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
}

pub fn process_ota(
  ota_receiver: &mpsc::Receiver<CommandMessage>,
  serial_sender: &mpsc::Sender<String>,
) {
  let only_firmware = prepare_fw();
  loop {
    match ota_receiver.recv() {
      Ok(command_message_request) => match command_message_request.sub_type {
        CommandSubType::StFirmwareConfigRequest => send_response(
          serial_sender,
          command_message_request.clone(),
          &only_firmware,
        ),
        CommandSubType::StFirmwareRequest => send_response(
          serial_sender,
          command_message_request.clone(),
          &only_firmware,
        ),
        _ => (),
      },
      _ => (),
    }
  }
}

fn send_response(
  serial_sender: &mpsc::Sender<String>,
  mut command_message: CommandMessage,
  _firmware: &Firmware,
) {
  command_message.to_response(_firmware);
  let response = command_message.serialize();
  println!("sending: {:?}", response);
  serial_sender.send(response).unwrap();
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

pub fn prepare_fw() -> Firmware {
  let f = File::open("Blink.ino.hex").unwrap();
  let f = BufReader::new(f);
  let mut result_bin: Vec<u8> = f.lines()
    .flat_map(|line| ihex_to_bin(&Record::from_record_string(&line.unwrap()).unwrap()))
    .collect();

  let mut state = State::<MODBUS>::new();
  state.update(&result_bin);
  // let crc = state.get();
  let pads: usize = result_bin.len() % 128; // 128 bytes per page for atmega328
  for _ in 0..(128 - pads) {
    result_bin.push(255);
  }
  let blocks: u16 = result_bin.len() as u16 / FIRMWARE_BLOCK_SIZE;
  Firmware {
    _type: 10,
    version: 2,
    blocks: blocks,
    crc: 0x46D4,
    bin_data: result_bin,
  }
}

#[cfg(test)]
mod test {
  use super::*;

  use hex;

  #[test]
  fn reader_respects_all_newline_formats() {
    let input = String::new() + &":100490008B002097E1F30E940000F9CF0895F894B3";

    assert_eq!(
      String::from("8B002097E1F30E940000F9CF0895F894"),
      hex::encode_upper(ihex_to_bin(&Record::from_record_string(&input).unwrap()))
    );
  }

  #[test]
  fn hex_file_to_vector() {
    let fw_binary = prepare_fw();
    assert!(fw_binary.bin_data.len() == 1280);
  }

  #[test]
  fn extract_given_block_from_binary_data() {
    let fw_binary = prepare_fw();
    assert!(
      fw_binary.get_block(1)
        == [
          12, 148, 110, 0, 12, 148, 110, 0, 12, 148, 110, 0, 12, 148, 110, 0
        ]
    );
  }
}
