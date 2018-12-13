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
    pub fn is_supported(&self) -> bool {
        !(self.thing_type().is_empty() || self.thing_description().is_empty()
            || !(self.property_types()
                .into_iter()
                .map(|p| p.is_supported())
                .fold(true, |f, s| s && f)))
    }

    pub fn thing_type(&self) -> String {
        match *self {
            PresentationType::Door => "onOffSwitch".to_owned(),
            PresentationType::Motion => "binarySensor".to_owned(),
            PresentationType::Smoke => "binarySensor".to_owned(),
            PresentationType::Binary => "onOffLight".to_owned(),
            PresentationType::Dimmer => "dimmableLight".to_owned(),
            PresentationType::Temp => "multiLevelSensor".to_owned(),
            PresentationType::Hum => "multiLevelSensor".to_owned(),
            PresentationType::Lock => "onOffSwitch".to_owned(),
            PresentationType::AirQuality => "multiLevelSensor".to_owned(),
            PresentationType::Moisture => "multiLevelSensor".to_owned(),
            PresentationType::Baro => "multiLevelSensor".to_owned(),
            PresentationType::Dust => "multiLevelSensor".to_owned(),
            PresentationType::Info => "string".to_owned(); // IS THIS CORRECT?
            _ => "".to_owned(),
        }
    }

    pub fn thing_description(&self) -> String {
        match *self {
            PresentationType::Door => "Door lock".to_owned(),
            PresentationType::Motion => "Motion sensor".to_owned(),
            PresentationType::Smoke => "Smoke sensor".to_owned(),
            PresentationType::Binary => "Binary switch".to_owned(),
            PresentationType::Dimmer => "Dimmable device".to_owned(),
            PresentationType::Temp => "Temperature sensor".to_owned(),
            PresentationType::Hum => "Humidity sensor".to_owned(),
            PresentationType::Lock => "Lock device".to_owned(),
            PresentationType::AirQuality => "Air Quality sensor".to_owned(),
            PresentationType::Moisture => "Moisture sensor".to_owned(),
            PresentationType::Baro => "Barometer sensor".to_owned(),
            PresentationType::Dust => "Dust sensor".to_owned(),
            PresentationType::Info => "Text".to_owned(),
            _ => "".to_owned(),
        }
    }

    pub fn property_types(&self) -> Vec<SetReqType> {
        match *self {
            PresentationType::Door => vec![SetReqType::Armed],
            PresentationType::Motion => vec![SetReqType::Tripped],
            PresentationType::Smoke => vec![SetReqType::Tripped],
            PresentationType::Binary => vec![SetReqType::Status],
            PresentationType::Dimmer => vec![SetReqType::Status, SetReqType::Percentage],
            PresentationType::Temp => vec![SetReqType::Temp, SetReqType::Status],
            PresentationType::Hum => vec![SetReqType::Hum],
            PresentationType::Lock => vec![SetReqType::LockStatus],
            PresentationType::AirQuality => vec![SetReqType::Level, SetReqType::UnitPrefix],
            PresentationType::Moisture => vec![SetReqType::Level],
            PresentationType::Baro => vec![SetReqType::Pressure, SetReqType::Forecast],
            PresentationType::Dust => vec![SetReqType::Level,SetReqType::UnitPrefix],
            PresentationType::Info => vec![SetReqType::String],
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _cmd = 0;
        let _sub_type = (self.sub_type) as u8;
        write!(
            f,
            "{:?};{};{:?};{};{:?};{}\n",
            self.node_id, self.child_sensor_id, _cmd, self.ack, _sub_type, &self.payload
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn supported_sensor_types() {
        assert!(PresentationType::Door.is_supported());
        assert!(PresentationType::Motion.is_supported());
        assert!(PresentationType::Smoke.is_supported());
        assert!(PresentationType::Binary.is_supported());
        assert!(PresentationType::Dimmer.is_supported());
        assert!(PresentationType::Temp.is_supported());
        assert!(PresentationType::Hum.is_supported());
        assert!(PresentationType::Lock.is_supported());
        assert!(PresentationType::AirQuality.is_supported());
        assert!(PresentationType::Moisture.is_supported());
        assert!(PresentationType::Baro.is_supported());
        assert!(PresentationType::Dust.is_supported());
        assert!(PresentationType::Info.is_supported());
    }
}
