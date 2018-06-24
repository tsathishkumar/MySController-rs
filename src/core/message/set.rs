use super::error::ParseError;
use model::sensor::Sensor;
use num::FromPrimitive;
use serde_json;
use std::fmt;

#[derive(Debug)]
pub struct SetMessage {
    pub node_id: u8,
    pub child_sensor_id: u8,
    pub ack: u8,
    pub value: Value,
}

impl fmt::Display for SetMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let _cmd = 1;
        write!(
            f,
            "{};{};{};{};{}\n",
            self.node_id,
            self.child_sensor_id,
            _cmd,
            0,
            self.value.to_string()
        )
    }
}

impl SetMessage {
    pub fn build(
        node_id: u8,
        child_sensor_id: u8,
        ack: u8,
        sub_type: u8,
        payload: &str,
    ) -> Result<SetMessage, ParseError> {
        let sub_type = SetReqType::from_u8(sub_type).ok_or(ParseError::InvalidSubType)?;
        Ok(SetMessage {
            node_id,
            child_sensor_id,
            ack,
            value: Value {
                set_type: sub_type,
                value: String::from(payload),
            },
        })
    }

    pub fn for_sensor(&self, sensor: &Sensor) -> bool {
        self.node_id == sensor.node_id as u8 && self.child_sensor_id == sensor.child_sensor_id as u8
    }
}

#[derive(Debug)]
pub struct Value {
    pub set_type: SetReqType,
    pub value: String,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{};{}", self.set_type as u8, &self.value)
    }
}

impl Value {
    pub fn to_json(&self) -> Option<serde_json::Value> {
        match self.set_type.data_type() {
            "boolean" => match self.value.as_str() {
                "1" => Some(json!(true)),
                "0" => Some(json!(false)),
                _ => None,
            },
            "number" => match self.value.parse::<f64>() {
                Ok(number) => Some(json!(number)),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn build(set_type: SetReqType, value: serde_json::Value) -> Option<Value> {
        let value = match set_type.data_type() {
            "boolean" => match value {
                serde_json::Value::Bool(true) => Some("1".to_owned()),
                serde_json::Value::Bool(false) => Some("0".to_owned()),
                _ => Some("".to_owned()),
            },
            "number" => match value {
                serde_json::Value::Number(number) => Some(number.to_string()),
                _ => Some("".to_owned()),
            },
            _ => None,
        };
        value.map(|value| Value { set_type, value })
    }
}

enum_from_primitive! {
    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum SetReqType {
        Temp = 0,
        Hum = 1,
        Status = 2,
        Percentage = 3,
        Pressure = 4,
        Forecast = 5,
        Rain = 6,
        Rainrate = 7,
        Wind = 8,
        Gust = 9,
        Direction = 10,
        Uv = 11,
        Weight = 12,
        Distance = 13,
        Impedance = 14,
        Armed = 15,
        Tripped = 16,
        Watt = 17,
        Kwh = 18,
        SceneOn = 19,
        SceneOff = 20,
        HvacFlowState = 21,
        HvacSpeed = 22,
        LightLevel = 23,
        Var1 = 24,
        Var2 = 25,
        Var3 = 26,
        Var4 = 27,
        Var5 = 28,
        Up = 29,
        Down = 30,
        Stop = 31,
        IRSend = 32,
        IRReceive = 33,
        Flow = 34,
        Volume = 35,
        LockStatus = 36,
        Level = 37,
        Voltage = 38,
        Current = 39,
        Rgb = 40,
        Rgbw = 41,
        Id = 42,
        UnitPrefix = 43,
        HvacSetpointCool = 44,
        HvacSetpointHeat = 45,
        HvacFlowMode = 46,
        Text = 47,
        Custom = 48,
        Position = 49,
        IRRecord = 50,
        Ph = 51,
        Orp = 52,
        Ec = 53,
        Var = 54,
        Va = 55,
        PowerFactor = 56,
    }
}

impl SetReqType {
    pub fn is_supported(&self) -> bool {
        !(self.property_name().is_empty() || self.data_type().is_empty()
            || self.description().is_empty())
    }

    pub fn is_forwardable(&self) -> bool {
        match *self {
            SetReqType::Temp => false,
            SetReqType::Percentage | SetReqType::Status => true,
            _ => false,
        }
    }

    pub fn property_name(&self) -> String {
        match *self {
            SetReqType::Temp => "level",
            SetReqType::Status => "on",
            SetReqType::Percentage => "level",
            _ => "",
        }.to_string()
    }

    pub fn data_type(&self) -> &'static str {
        match *self {
            SetReqType::Status => "boolean",
            SetReqType::Temp => "number",
            SetReqType::Percentage => "number",
            _ => "",
        }
    }

    pub fn description(&self) -> String {
        match *self {
            SetReqType::Temp => "Temperature".to_owned(),
            SetReqType::Status => "Whether the thing is on".to_owned(),
            SetReqType::Percentage => "The level of the thing from 0-100".to_owned(),
            _ => "".to_owned(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn enum_primitive() {
        assert_eq!(0, SetReqType::Temp as u8);
    }

    #[test]
    fn supported_sub_type() {
        assert!(SetReqType::Temp.is_supported());
        assert!(SetReqType::Status.is_supported());
        assert!(SetReqType::Percentage.is_supported());
    }

    #[test]
    fn set_message_display_method() {
        assert_eq!(
            "1;2;1;0;2;1\n",
            SetMessage {
                node_id: 1,
                child_sensor_id: 2,
                ack: 0,
                value: Value {
                    set_type: SetReqType::Status,
                    value: "1".to_owned()
                }
            }.to_string()
        )
    }
}
