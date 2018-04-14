use enum_primitive;
use firmware::Firmware;
use hex;
use num::FromPrimitive;
use std::mem;

const MAX_MESSAGE_LENGTH: usize = 32;

enum_from_primitive! {
    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum CommandType {
        PRESENTATION = 0,
        SET = 1,
        REQ = 2,
        INTERNAL = 3,
        STREAM = 4,
    }
}

enum_from_primitive! {
    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum CommandSubType {
        StFirmwareConfigRequest  = 0,
        StFirmwareConfigResponse = 1,
        StFirmwareRequest = 2,  // Request FW block
        StFirmwareResponse = 3, // Response FW block
        Other = 9
    }
}

impl CommandType {
    pub fn _u8(value: u8) -> enum_primitive::Option<CommandType> {
        CommandType::from_u8(value)
    }
}

impl CommandSubType {
    pub fn _u8(value: u8) -> enum_primitive::Option<CommandSubType> {
        CommandSubType::from_u8(value)
    }
}

//"node-id ; child-sensor-id ; command ; ack ; type ; payload \n"
#[derive(Clone, Copy, Debug)]
pub struct CommandMessage {
    node_id: u8,
    child_sensor_id: u8,
    pub command: CommandType,
    ack: u8,
    pub sub_type: CommandSubType,
    pub payload: MessagePayloadType,
}

#[derive(Debug, Clone, Copy)]
pub enum MessagePayloadType {
    FwConfigRequest(FwConfigRequestMessage),
    FwRequest(FwRequestMessage),
    FwConfigResponse(FwConfigResponseMessage),
    FwResponse(FwResponseMessage),
    Other([u8; 32]),
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
        FirmwarePayload { data: data }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FwConfigRequestMessage {
    _type: u16,
    version: u16,
    blocks: u16,
    crc: u16,
    bl_version: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct FwConfigResponseMessage {
    _type: u16,
    version: u16,
    blocks: u16,
    crc: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct FwRequestMessage {
    _type: u16,
    version: u16,
    blocks: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct FwResponseMessage {
    _type: u16,
    version: u16,
    blocks: u16,
    data: [u8; 16],
}

impl CommandMessage {
    pub fn new(command_message: &String) -> Result<CommandMessage, String> {
        let message_parts = command_message.trim().split(";").collect::<Vec<&str>>();
        if message_parts.len() != 6 {
            return Err(
                "Invalid Command Message, should have 6 components separated by ';'".to_string(),
            );
        }

        let command = match message_parts[2].parse::<u8>() {
            Ok(result) => CommandType::from_u8(result).unwrap(),
            _ => return Err("Error parsing string to command".to_string()),
        };
        let sub_type = match command {
            CommandType::STREAM => match message_parts[4].parse::<u8>() {
                Ok(result) => CommandSubType::from_u8(result).unwrap(),
                _ => return Err("Error parsing string to sub_type".to_string()),
            },
            _ => CommandSubType::Other,
        };
        let array_val = match command {
            CommandType::STREAM => {
                let command_vector = match hex::decode(message_parts[5]) {
                    Ok(result) => result,
                    _ => return Err("Error while decoding hex".to_string()),
                };
                vector_as_u8_32_array(command_vector)
            }
            _ => [0; 32],
        };
        Result::Ok(CommandMessage {
            node_id: match message_parts[0].parse::<u8>() {
                Ok(result) => result,
                _ => return Err("Error parsing string to node_id".to_string()),
            },
            child_sensor_id: match message_parts[1].parse::<u8>() {
                Ok(result) => result,
                _ => return Err("Error parsing string to child_sensor_id".to_string()),
            },
            command: command.clone(),
            ack: match message_parts[3].parse::<u8>() {
                Ok(result) => result,
                _ => return Err("Error parsing string to ack".to_string()),
            },
            sub_type: sub_type.clone(),
            payload: match command {
                CommandType::STREAM => match sub_type {
                    CommandSubType::StFirmwareConfigRequest => MessagePayloadType::FwConfigRequest(
                        unsafe { FirmwarePayload::new(array_val).fw_config_request },
                    ),
                    CommandSubType::StFirmwareConfigResponse => {
                        MessagePayloadType::FwConfigResponse(unsafe {
                            FirmwarePayload::new(array_val).fw_config_response
                        })
                    }
                    CommandSubType::StFirmwareRequest => MessagePayloadType::FwRequest(unsafe {
                        FirmwarePayload::new(array_val).fw_request
                    }),
                    CommandSubType::StFirmwareResponse => MessagePayloadType::FwResponse(unsafe {
                        FirmwarePayload::new(array_val).fw_response
                    }),
                    _ => MessagePayloadType::Other(array_val),
                },
                _ => MessagePayloadType::Other(array_val),
            },
        })
    }

    pub fn fw_type_version(&self) -> Option<(u16, u16)> {
        match self.payload {
            MessagePayloadType::FwConfigRequest(_request) => Some((_request._type, _request.version)),
            MessagePayloadType::FwRequest(request) => Some((request._type, request.version)),
            _ => None,
        }
    }

    pub fn to_response(&mut self, firmware: &Firmware) {
        self.sub_type = match self.sub_type {
            CommandSubType::StFirmwareConfigRequest => CommandSubType::StFirmwareConfigResponse,
            CommandSubType::StFirmwareRequest => CommandSubType::StFirmwareResponse,
            _ => self.sub_type,
        };

        self.payload = match self.payload {
            MessagePayloadType::FwConfigRequest(_request) => {
                MessagePayloadType::FwConfigResponse(FwConfigResponseMessage {
                    _type: firmware._type,
                    version: firmware.version,
                    blocks: firmware.blocks,
                    crc: firmware.crc,
                })
            }
            MessagePayloadType::FwRequest(request) => {
                MessagePayloadType::FwResponse(FwResponseMessage {
                    _type: firmware._type,
                    version: firmware.version,
                    blocks: request.blocks,
                    data: firmware.get_block(request.blocks),
                })
            }
            _ => self.payload,
        };
    }
}

impl CommandMessage {
    pub fn serialize(self) -> String {
        let _cmd = (self.command) as u8;
        let _sub_type = (self.sub_type) as u8;
        let payload = match self.payload {
            MessagePayloadType::FwConfigResponse(stream_payload) => {
                hex::encode_upper(&unsafe { mem::transmute::<_, [u8; 8]>(stream_payload) })
            }
            MessagePayloadType::FwResponse(stream_payload) => {
                hex::encode_upper(&unsafe { mem::transmute::<_, [u8; 22]>(stream_payload) })
            }
            MessagePayloadType::FwConfigRequest(stream_payload) => {
                hex::encode_upper(&unsafe { mem::transmute::<_, [u8; 10]>(stream_payload) })
            }
            MessagePayloadType::FwRequest(stream_payload) => {
                hex::encode_upper(&unsafe { mem::transmute::<_, [u8; 6]>(stream_payload) })
            }
            MessagePayloadType::Other(payload_string) => hex::encode(payload_string),
        };
        format!(
            "{};{};{:?};{};{:?};{}\n",
            self.node_id, self.child_sensor_id, _cmd, self.ack, _sub_type, &payload
        )
    }
}

pub fn command_type(message_string: &String) -> Option<CommandType> {
    let message_parts = message_string.split(";").collect::<Vec<&str>>();
    if message_parts.len() == 6 {
        //"node-id ; child-sensor-id ; command ; ack ; type ; payload \n"
        let command_type = message_parts[2].parse::<u8>().unwrap();
        match command_type {
            0 => Some(CommandType::PRESENTATION),
            1 => Some(CommandType::SET),
            2 => Some(CommandType::REQ),
            3 => Some(CommandType::INTERNAL),
            4 => Some(CommandType::STREAM),
            _ => None,
        }
    } else {
        None
    }
}

fn vector_as_u8_32_array(vector: Vec<u8>) -> [u8; MAX_MESSAGE_LENGTH] {
    let mut arr = [0u8; MAX_MESSAGE_LENGTH];
    for (place, element) in arr.iter_mut().zip(vector.iter()) {
        *place = *element;
    }
    arr
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn parse_correct_command_fw_config_request() {
        let message_string = "1;255;4;0;0;0A0001005000D4460102\n";
        let command_message = CommandMessage::new(&String::from(message_string)).unwrap();
        assert_eq!(
            command_message.sub_type,
            CommandSubType::StFirmwareConfigRequest
        );
        let stream_payload = match command_message.payload {
            MessagePayloadType::FwConfigRequest(stream_payload) => Some(stream_payload),
            _ => None,
        }.unwrap();
        assert_eq!(stream_payload._type, 10);
        assert_eq!(stream_payload.version, 1);
        assert_eq!(stream_payload.blocks, 80);
        assert_eq!(stream_payload.crc, 18132);
        assert_eq!(stream_payload.bl_version, 513);
    }

    #[test]
    fn parse_correct_command_fw_config_response() {
        let message_string = "1;255;4;0;1;0A0002005000D446\n";
        let command_message = CommandMessage::new(&String::from(message_string)).unwrap();
        assert_eq!(
            command_message.sub_type,
            CommandSubType::StFirmwareConfigResponse
        );
        let stream_payload = match command_message.payload {
            MessagePayloadType::FwConfigResponse(stream_payload) => Some(stream_payload),
            _ => None,
        }.unwrap();
        assert_eq!(stream_payload._type, 10);
        assert_eq!(stream_payload.version, 2);
        assert_eq!(stream_payload.blocks, 80);
        assert_eq!(stream_payload.crc, 18132);
    }

    #[test]
    fn parse_correct_command_fw_request() {
        let message_string = "1;255;4;0;2;0A0002004F00\n ";
        let command_message = CommandMessage::new(&String::from(message_string)).unwrap();
        assert_eq!(command_message.sub_type, CommandSubType::StFirmwareRequest);

        let stream_payload = match command_message.payload {
            MessagePayloadType::FwRequest(stream_payload) => Some(stream_payload),
            _ => None,
        }.unwrap();

        assert_eq!(stream_payload._type, 10);
        assert_eq!(stream_payload.version, 2);
        assert_eq!(stream_payload.blocks, 79);
    }

    #[test]
    fn parse_correct_command_fw_response() {
        let message_string = "1;255;4;0;3;0A0001004F00FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF\n ";
        let command_message = CommandMessage::new(&String::from(message_string)).unwrap();
        assert_eq!(command_message.sub_type, CommandSubType::StFirmwareResponse);
        let stream_payload = match command_message.payload {
            MessagePayloadType::FwResponse(stream_payload) => Some(stream_payload),
            _ => None,
        }.unwrap();

        assert_eq!(stream_payload._type, 10);
        assert_eq!(stream_payload.version, 1);
        assert_eq!(stream_payload.blocks, 79);
    }

    #[test]
    fn format_fw_config_request() {
        let message_string = "1;255;4;0;0;0A0001005000D4460102\n";
        let command_message = CommandMessage::new(&String::from(message_string)).unwrap();
        assert_eq!(command_message.serialize(), String::from(message_string));
    }

    #[test]
    fn format_fw_config_response() {
        let message_string = "1;255;4;0;1;0A0002005000D446\n";
        let command_message = CommandMessage::new(&String::from(message_string)).unwrap();
        assert_eq!(command_message.serialize(), String::from(message_string));
    }

    #[test]
    fn format_fw_resquest() {
        let message_string = "1;255;4;0;3;0A0002004F0000000000000000000000000000000000\n";
        let command_message = CommandMessage::new(&String::from(message_string)).unwrap();
        assert_eq!(command_message.serialize(), String::from(message_string));
    }

    #[test]
    fn format_fw_response() {
        let message_string = "1;255;4;0;3;0A0001004F00FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF\n";
        let command_message = CommandMessage::new(&String::from(message_string)).unwrap();
        assert_eq!(command_message.serialize(), String::from(message_string));
    }

    #[test]
    fn convert_fw_config_request_to_response() {
        let message_string = "1;255;4;0;0;0A0001005000D4460102\n";
        let mut command_message = CommandMessage::new(&String::from(message_string)).unwrap();
        command_message.to_response(&Firmware{_type: 10, version: 2, blocks: 79, crc: 1000, bin_data: vec![], name: String::from("Blink.hex")});
        assert_eq!(
            command_message.serialize(),
            String::from("1;255;4;0;1;0A0002004F00E803\n")
        );
    }

    #[test]
    fn convert_fw_request_to_response() {
        let message_string = "1;255;4;0;2;0A0002000700\n";
        let mut command_message = CommandMessage::new(&String::from(message_string)).unwrap();
        command_message.to_response(&Firmware::prepare_fw(10,2, String::from("firmwares/10__2__Blink.ino.hex")));
        assert_eq!(
            command_message.serialize(),
            String::from("1;255;4;0;3;0A000200070000030407000000000000000001020408\n")
        );
    }
}
