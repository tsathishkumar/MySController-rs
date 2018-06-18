#[derive(Debug)]
pub enum ParseError {
    InvalidCommandMessage,
    InvalidNodeId,
    InvalidChildSensorId,
    InvalidCommand,
    InvalidACK,
    InvalidSubType,
    InvalidPayload,
}
