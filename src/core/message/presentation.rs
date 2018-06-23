use super::error::ParseError;
use super::set::SetReqType;
use num::FromPrimitive;
use std::fmt;

enum_from_primitive! {
    #[derive(DbEnum, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
    pub enum PresentationType {
        Door=0,
        Motion=1,
        Smoke=2,
        Binary=3,
        Dimmer=4,
        Cover=5,
        Temp=6,
        Hum=7,
        Baro=8,
        Wind=9,
        Rain=10,
        Uv=11,
        Weight=12,
        Power=13,
        Heater=14,
        Distance=15,
        LightLevel=16,
        ArduinoNode=17,
        ArduinoRepeaterNode=18,
        Lock=19,
        Ir=20,
        Water=21,
        AirQuality=22,
        Custom=23,
        Dust=24,
        SceneController=25,
        RgbLight=26,
        RgbwLight=27,
        ColorSensor=28,
        Hvac=29,
        Multimeter=30,
        Sprinkler=31,
        WaterLeak=32,
        Sound=33,
        Vibration=34,
        Moisture=35,
        Info=36,
        Gas=37,
        Gps=38,
        WaterQuality=39,
    }
}

impl PresentationType {
    pub fn thing_type(&self) -> String {
        match *self {
            PresentationType::Binary => "onOffLight".to_owned(),
            _ => "".to_owned(),
        }
    }

    pub fn thing_description(&self) -> String {
        match *self {
            PresentationType::Binary => "A web connected lamp".to_owned(),
            _ => "".to_owned(),
        }
    }

    pub fn set_types(&self) -> Vec<SetReqType> {
        match *self {
            PresentationType::Binary => vec![SetReqType::Status],
            _ => Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PresentationMessage {
    pub node_id: u8,
    pub child_sensor_id: u8,
    pub ack: u8,
    pub sub_type: PresentationType,
    pub payload: String,
}

impl PresentationMessage {
    pub fn build(
        node_id: u8,
        child_sensor_id: u8,
        ack: u8,
        sub_type: u8,
        payload: &str,
    ) -> Result<PresentationMessage, ParseError> {
        let sub_type = PresentationType::from_u8(sub_type).ok_or(ParseError::InvalidSubType)?;
        Ok(PresentationMessage {
            node_id,
            child_sensor_id,
            ack,
            sub_type,
            payload: String::from(payload),
        })
    }
}

impl fmt::Display for PresentationMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let _cmd = 0;
        let _sub_type = (self.sub_type) as u8;
        write!(
            f,
            "{:?};{};{:?};{};{:?};{}\n",
            self.node_id, self.child_sensor_id, _cmd, self.ack, _sub_type, &self.payload
        )
    }
}
