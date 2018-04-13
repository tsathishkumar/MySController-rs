use message::{CommandMessage, CommandSubType, MessagePayloadType};

use ihex::record::Record;

use hex;

use crc16::*;

use std::sync::mpsc;

use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader};

const FIRMWARE_BLOCK_SIZE: usize = 16;

#[derive(Debug)]
pub struct Firmware {
  _type: u8,
  version: u8,
  blocks: usize,
  crc: u16,
  bin_data: Vec<u8>,
}

pub fn process_ota(ota_receiver: &mpsc::Receiver<CommandMessage>) {
  let only_firmware = prepare_fw();
  loop {
    let command_message_request = ota_receiver.recv().unwrap();
    match command_message_request.sub_type {
      CommandSubType::StFirmwareConfigRequest => {
        send_fw_config_response(command_message_request.clone(), &only_firmware)
      }
      CommandSubType::StFirmwareRequest => send_fw_response(command_message_request.clone()),
      _ => (),
    }
  }
}

fn send_fw_config_response(mut command_message: CommandMessage, _firmware: &Firmware) {
  match command_message.to_response() {
    Ok(_message) => match command_message.payload {
      MessagePayloadType::FwConfigRequest(payload) => println!("FW Config request payload {:?}", &payload),
      MessagePayloadType::FwRequest(payload) => println!("FW request payload {:?}", &payload),
      _ => (),
    },
    _ => (),
  }
}

fn send_fw_response(_command_message: CommandMessage) {}

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
  let crc = state.get();
  println!("{}", crc);
  let pads: usize = result_bin.len() % 128; // 128 bytes per page for atmega328
  for _ in 0..(128 - pads) {
    result_bin.push(255);
  }
  let blocks: usize = result_bin.len() / FIRMWARE_BLOCK_SIZE;
  Firmware {
    _type: 10,
    version: 2,
    blocks: blocks,
    crc: 0xD446,
    bin_data: result_bin,
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_reader_respects_all_newline_formats() {
    let input = String::new() + &":100490008B002097E1F30E940000F9CF0895F894B3";

    assert_eq!(String::from("8B002097E1F30E940000F9CF0895F894"), hex::encode_upper(ihex_to_bin(&Record::from_record_string(&input).unwrap())));
  }

  #[test]
  fn test_hex_file_to_vector() {
    let fw_binary = prepare_fw();
    println!("{:?}", fw_binary);
    assert!(fw_binary.bin_data.len() == 1280);
  }

  // #[test]
  // fn test_crc() {
  //   // use provided or custom polynomial
  //   let mut digest = crc16::Digest::new_with_initial(0x18005, 0xFFFF);
  //   digest.write(&prepare_fw());
  //   assert_eq!(digest.sum16(), 0xD446);
  // }

  // #[test]
  // fn test_crc16() {
  //   let mut state = State::<MODBUS>::new();
  //   state.update(&prepare_fw());
  //   let crc = state.get();
  //   assert_eq!(crc, 0xD446);
  // }
}
