pub mod error;
pub mod presentation;
pub mod set;
pub mod stream;

use self::error::ParseError;
use num::FromPrimitive;
use std::fmt;

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

#[derive(Debug)]
pub enum CommandMessage {
    Presentation(presentation::PresentationMessage),
    Set(set::SetMessage),
    // Req(ReqMessage),
    // Internal(InternalMessage),
    Other(String),
    Stream(stream::StreamMessage),
}

//"node-id ; child-sensor-id ; command ; ack ; type ; payload \n"
impl CommandMessage {
    pub fn new(command_message: &String) -> Result<CommandMessage, ParseError> {
        let message_parts = command_message.trim().split(";").collect::<Vec<&str>>();

        if message_parts.len() != 6 {
            return Err(ParseError::InvalidCommandMessage);
        }

        let node_id = message_parts[0]
            .parse::<u8>()
            .map_err(|_| ParseError::InvalidNodeId)?;
        let child_sensor_id = message_parts[1]
            .parse::<u8>()
            .map_err(|_| ParseError::InvalidChildSensorId)?;
        let command = message_parts[2]
            .parse::<u8>()
            .map_err(|_| ParseError::InvalidCommand)
            .and_then(|result| CommandType::from_u8(result).ok_or(ParseError::InvalidCommand))?;
        let ack = message_parts[3]
            .parse::<u8>()
            .map_err(|_| ParseError::InvalidACK)?;
        let sub_type = message_parts[4]
            .parse::<u8>()
            .map_err(|_| ParseError::InvalidSubType)?;
        let payload = message_parts[5];

        Ok(match command {
            CommandType::STREAM => CommandMessage::Stream(stream::StreamMessage::build(
                node_id,
                child_sensor_id,
                sub_type,
                ack,
                payload,
            )?),
            CommandType::PRESENTATION => {
                CommandMessage::Presentation(presentation::PresentationMessage::build(
                    node_id,
                    child_sensor_id,
                    ack,
                    sub_type,
                    payload,
                )?)
            }
            CommandType::SET => CommandMessage::Set(set::SetMessage::build(
                node_id,
                child_sensor_id,
                ack,
                sub_type,
                payload,
            )?),
            _ => CommandMessage::Other(command_message.to_owned()),
        })
    }
}

impl fmt::Display for CommandMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            CommandMessage::Stream(ref message) => write!(f, "{}", message.to_string()),
            CommandMessage::Presentation(ref message) => write!(f, "{}", message.to_string()),
            CommandMessage::Set(ref message) => write!(f, "{}", message.to_string()),
            CommandMessage::Other(ref message) => write!(f, "{}", message),
        }
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::firmware::Firmware;
    use std::path::PathBuf;

    #[test]
    fn parse_correct_command_fw_config_request() {
        let message_string = "1;255;4;0;0;0A0001005000D4460102\n";
        if let Ok(CommandMessage::Stream(message)) =
            CommandMessage::new(&String::from(message_string))
        {
            assert_eq!(
                message.sub_type,
                stream::StreamType::StFirmwareConfigRequest
            );
            let stream_payload = match message.payload {
                stream::StreamPayload::FwConfigRequest(stream_payload) => Some(stream_payload),
                _ => None,
            }.unwrap();
            assert_eq!(stream_payload.firmware_type, 10);
            assert_eq!(stream_payload.firmware_version, 1);
            assert_eq!(stream_payload.blocks, 80);
            assert_eq!(stream_payload.crc, 18132);
            assert_eq!(stream_payload.bl_version, 513);
        } else {
            assert!(false, "Didn't parse to Stream message");
        }
    }

    #[test]
    fn parse_correct_command_fw_config_response() {
        let message_string = "1;255;4;0;1;0A0002005000D446\n";
        if let Ok(CommandMessage::Stream(message)) =
            CommandMessage::new(&String::from(message_string))
        {
            assert_eq!(
                message.sub_type,
                stream::StreamType::StFirmwareConfigResponse
            );
            let stream_payload = match message.payload {
                stream::StreamPayload::FwConfigResponse(stream_payload) => Some(stream_payload),
                _ => None,
            }.unwrap();
            assert_eq!(stream_payload.firmware_type, 10);
            assert_eq!(stream_payload.firmware_version, 2);
            assert_eq!(stream_payload.blocks, 80);
            assert_eq!(stream_payload.crc, 18132);
        } else {
            assert!(false, "Didn't parse to Stream message");
        }
    }

    #[test]
    fn parse_correct_command_fw_request() {
        let message_string = "1;255;4;0;2;0A0002004F00\n ";
        if let Ok(CommandMessage::Stream(message)) =
            CommandMessage::new(&String::from(message_string))
        {
            assert_eq!(message.sub_type, stream::StreamType::StFirmwareRequest);

            let stream_payload = match message.payload {
                stream::StreamPayload::FwRequest(stream_payload) => Some(stream_payload),
                _ => None,
            }.unwrap();

            assert_eq!(stream_payload.firmware_type, 10);
            assert_eq!(stream_payload.firmware_version, 2);
            assert_eq!(stream_payload.blocks, 79);
        } else {
            assert!(false, "Didn't parse to Stream message");
        }
    }

    #[test]
    fn parse_correct_command_fw_response() {
        let message_string = "1;255;4;0;3;0A0001004F00FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF\n ";
        if let Ok(CommandMessage::Stream(message)) =
            CommandMessage::new(&String::from(message_string))
        {
            assert_eq!(message.sub_type, stream::StreamType::StFirmwareResponse);
            let stream_payload = match message.payload {
                stream::StreamPayload::FwResponse(stream_payload) => Some(stream_payload),
                _ => None,
            }.unwrap();

            assert_eq!(stream_payload.firmware_type, 10);
            assert_eq!(stream_payload.firmware_version, 1);
            assert_eq!(stream_payload.blocks, 79);
        } else {
            assert!(false, "Didn't parse to Stream message");
        }
    }

    #[test]
    fn format_fw_config_request() {
        let message_string = "1;255;4;0;0;0A0001005000D4460102\n";
        let command_message = CommandMessage::new(&String::from(message_string)).unwrap();
        assert_eq!(command_message.to_string(), String::from(message_string));
    }

    #[test]
    fn format_fw_config_response() {
        let message_string = "1;255;4;0;1;0A0002005000D446\n";
        let command_message = CommandMessage::new(&String::from(message_string)).unwrap();
        assert_eq!(command_message.to_string(), String::from(message_string));
    }

    #[test]
    fn format_fw_resquest() {
        let message_string = "1;255;4;0;3;0A0002004F0000000000000000000000000000000000\n";
        let command_message = CommandMessage::new(&String::from(message_string)).unwrap();
        assert_eq!(command_message.to_string(), String::from(message_string));
    }

    #[test]
    fn format_fw_response() {
        let message_string = "1;255;4;0;3;0A0001004F00FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF\n";
        let command_message = CommandMessage::new(&String::from(message_string)).unwrap();
        assert_eq!(command_message.to_string(), String::from(message_string));
    }

    #[test]
    fn convert_fw_config_request_to_response() {
        let message_string = "1;255;4;0;0;0A0001005000D4460102\n";
        if let Ok(CommandMessage::Stream(mut message)) =
            CommandMessage::new(&String::from(message_string))
        {
            message.to_response(&Firmware {
                firmware_type: 10,
                firmware_version: 2,
                blocks: 79,
                crc: 1000,
                data: vec![],
                name: String::from("Blink.hex"),
            });
            assert_eq!(
                message.to_string(),
                String::from("1;255;4;0;1;0A0002004F00E803\n")
            );
        } else {
            assert!(false, "Didn't parse to Stream message");
        }
    }

    #[test]
    fn convert_fw_request_to_response() {
        let message_string = "1;255;4;0;2;0A0002000700\n";
        if let Ok(CommandMessage::Stream(mut message)) =
            CommandMessage::new(&String::from(message_string))
        {
            message.to_response(&Firmware::prepare_fw(
                10,
                2,
                String::from("Blink"),
                &PathBuf::from("firmwares/10__2__Blink.ino.hex"),
            ).unwrap());
            assert_eq!(
                message.to_string(),
                String::from("1;255;4;0;3;0A000200070000030407000000000000000001020408\n")
            );
        } else {
            assert!(false, "Didn't parse to Stream message");
        }
    }
}
