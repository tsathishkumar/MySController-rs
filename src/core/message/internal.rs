use super::error::ParseError;
use hex;
use num::FromPrimitive;
use std::fmt;
use std::mem;

const MAX_MESSAGE_LENGTH: usize = 32;

enum_from_primitive! {
    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum InternalType {
        BatteryLevel         = 0,  // Use this to report the battery level (in percent 0-100).
        Time                 = 1,  // Sensors can request the current time from the Controller using this message. The time will be reported as the seconds since 1970
        Version              = 2,  // Used to request gateway version from controller.
        IdRequest            = 3,  // Use this to request a unique node id from the controller.
        IdResponse           = 4,  // Id response back to node. Payload contains node id.
        InclusionMode        = 5,  // Start/stop inclusion mode of the Controller (1=start, 0=stop).
        Config               = 6,  // Config request from node. Reply with (M)etric or (I)mperal back to sensor.
        FindParent           = 7,  // When a sensor starts up, it broadcast a search request to all neighbor nodes. They reply with a I_FIND_PARENT_RESPONSE.
        FindParentResponse   = 8,  // Reply message type to I_FIND_PARENT request.
        LogMessage           = 9,  // Sent by the gateway to the Controller to trace-log a message
        Children             = 10, // A message that can be used to transfer child sensors (from EEPROM routing table) of a repeating node.
        SketchName           = 11, // Optional sketch name that can be used to identify sensor in the Controller GUI
        SketchVersion        = 12, // Optional sketch version that can be reported to keep track of the version of sensor in the Controller GUI.
        Reboot               = 13, // Used by OTA firmware updates. Request for node to reboot.
        GatewayReady         = 14, // Send by gateway to controller when startup is complete.
        SigningPresentation  = 15, // Provides signing related preferences (first byte is preference version).
        NonceRequest         = 16, // Used between sensors when requesting nonce.
        NonceResponse        = 17, // Used between sensors for nonce response.
        HeartbeatRequest     = 18, // Heartbeat request
        Presentation         = 19, // Presentation message
        DiscoverRequest      = 20, // Discover request
        DiscoverResponse     = 21, // Discover response
        HeartbeatResponse    = 22, // Heartbeat response
        Locked               = 23, // Node is locked (reason in string-payload)
        Ping                 = 24, // Ping sent to node, payload incremental hop counter
        Pong                 = 25, // In return to ping, sent back to sender, payload incremental hop counter
        RegistrationRequest  = 26, // Register request to GW
        RegistrationResponse = 27, // Register response from GW
        Debug                = 28  // Debug message
    }
}

#[derive(Clone, Debug)]
pub struct InternalMessage {
    pub node_id: u8,
    pub child_sensor_id: u8,
    pub ack: u8,
    pub sub_type: InternalType,
    pub payload: String,
}

impl InternalMessage {
    pub fn build(
        node_id: u8,
        child_sensor_id: u8,
        sub_type: u8,
        ack: u8,
        payload: &str,
    ) -> Result<InternalMessage, ParseError> {
        let sub_type = InternalType::from_u8(sub_type).ok_or(ParseError::InvalidSubType)?;
        Ok(InternalMessage {
            node_id: node_id,
            child_sensor_id: child_sensor_id,
            ack: ack,
            sub_type: sub_type,
            payload: String::from(payload),
        })
    }
}

impl fmt::Display for InternalMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _cmd = 3;
        let _sub_type = (self.sub_type) as u8;

        write!(
            f,
            "{:?};{};{:?};{};{:?};{}\n",
            self.node_id, self.child_sensor_id, _cmd, self.ack, _sub_type, &self.payload
        )
    }
}
