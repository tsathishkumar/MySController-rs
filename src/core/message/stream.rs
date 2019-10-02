use super::error::ParseError;
use hex;
use crate::model::firmware::Firmware;
use num::FromPrimitive;
use std::fmt;
use std::mem;

const MAX_MESSAGE_LENGTH: usize = 32;

enum_from_primitive! {
    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum StreamType {
        StFirmwareConfigRequest  = 0,
        StFirmwareConfigResponse = 1,
        StFirmwareRequest = 2,
        StFirmwareResponse = 3
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StreamMessage {
    pub node_id: u8,
    pub child_sensor_id: u8,
    pub ack: u8,
    pub sub_type: StreamType,
    pub payload: StreamPayload,
}

impl StreamMessage {
    pub fn build(
        node_id: u8,
        child_sensor_id: u8,
        sub_type: u8,
        ack: u8,
        payload: &str,
    ) -> Result<StreamMessage, ParseError> {
        let array_val = hex::decode(payload)
            .map_err(|_| ParseError::InvalidPayload)
            .and_then(|vector| Ok(vector_as_u8_32_array(vector)))?;
        let sub_type = StreamType::from_u8(sub_type).ok_or(ParseError::InvalidSubType)?;
        let payload = match sub_type {
            StreamType::StFirmwareConfigRequest => StreamPayload::FwConfigRequest(unsafe {
                FirmwarePayload::new(array_val).fw_config_request
            }),
            StreamType::StFirmwareConfigResponse => StreamPayload::FwConfigResponse(unsafe {
                FirmwarePayload::new(array_val).fw_config_response
            }),
            StreamType::StFirmwareRequest => {
                StreamPayload::FwRequest(unsafe { FirmwarePayload::new(array_val).fw_request })
            }
            StreamType::StFirmwareResponse => {
                StreamPayload::FwResponse(unsafe { FirmwarePayload::new(array_val).fw_response })
            }
        };
        Ok(StreamMessage {
            node_id,
            child_sensor_id,
            ack,
            sub_type,
            payload,
        })
    }

    pub fn response(&mut self, firmware: &Firmware) {
        self.sub_type = match self.sub_type {
            StreamType::StFirmwareConfigRequest => StreamType::StFirmwareConfigResponse,
            StreamType::StFirmwareRequest => StreamType::StFirmwareResponse,
            _ => self.sub_type,
        };

        self.payload = match self.payload {
            StreamPayload::FwConfigRequest(_request) => {
                StreamPayload::FwConfigResponse(FwConfigResponseMessage {
                    firmware_type: firmware.firmware_type as u16,
                    firmware_version: firmware.firmware_version as u16,
                    blocks: firmware.blocks as u16,
                    crc: firmware.crc as u16,
                })
            }
            StreamPayload::FwRequest(request) => StreamPayload::FwResponse(FwResponseMessage {
                firmware_type: firmware.firmware_type as u16,
                firmware_version: firmware.firmware_version as u16,
                blocks: request.blocks,
                data: firmware.get_block(request.blocks),
            }),
            _ => self.payload,
        };
    }
}

impl fmt::Display for StreamMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _cmd = 4;
        let _sub_type = (self.sub_type) as u8;
        let payload = match self.payload {
            StreamPayload::FwConfigResponse(stream_payload) => {
                hex::encode_upper(&unsafe { mem::transmute::<_, [u8; 8]>(stream_payload) })
            }
            StreamPayload::FwResponse(stream_payload) => {
                hex::encode_upper(&unsafe { mem::transmute::<_, [u8; 22]>(stream_payload) })
            }
            StreamPayload::FwConfigRequest(stream_payload) => {
                hex::encode_upper(&unsafe { mem::transmute::<_, [u8; 10]>(stream_payload) })
            }
            StreamPayload::FwRequest(stream_payload) => {
                hex::encode_upper(&unsafe { mem::transmute::<_, [u8; 6]>(stream_payload) })
            }
        };
        writeln!(
            f,
            "{:?};{};{:?};{};{:?};{}",
            self.node_id, self.child_sensor_id, _cmd, self.ack, _sub_type, &payload
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StreamPayload {
    FwConfigRequest(FwConfigRequestMessage),
    FwRequest(FwRequestMessage),
    FwConfigResponse(FwConfigResponseMessage),
    FwResponse(FwResponseMessage),
}

pub union FirmwarePayload {
    fw_config_request: FwConfigRequestMessage,
    fw_config_response: FwConfigResponseMessage,
    fw_request: FwRequestMessage,
    fw_response: FwResponseMessage,
    data: [u8; MAX_MESSAGE_LENGTH],
}

impl FirmwarePayload {
    pub fn new(data: [u8; MAX_MESSAGE_LENGTH]) -> FirmwarePayload {
        FirmwarePayload { data }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FwConfigRequestMessage {
    pub firmware_type: u16,
    pub firmware_version: u16,
    pub blocks: u16,
    pub crc: u16,
    pub bl_version: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct FwConfigResponseMessage {
    pub firmware_type: u16,
    pub firmware_version: u16,
    pub blocks: u16,
    pub crc: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct FwRequestMessage {
    pub firmware_type: u16,
    pub firmware_version: u16,
    pub blocks: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct FwResponseMessage {
    pub firmware_type: u16,
    pub firmware_version: u16,
    pub blocks: u16,
    pub data: [u8; 16],
}

fn vector_as_u8_32_array(vector: Vec<u8>) -> [u8; MAX_MESSAGE_LENGTH] {
    let mut arr = [0u8; MAX_MESSAGE_LENGTH];
    for (place, element) in arr.iter_mut().zip(vector.iter()) {
        *place = *element;
    }
    arr
}
