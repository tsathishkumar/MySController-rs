use channel::Sender;
use core::message::set::{SetMessage, SetReqType, Value};
use model::sensor::Sensor;
use serde_json;
use std::sync::{Arc, RwLock};
use webthing::property::EmptyValueForwarder;
use webthing::property::ValueForwarder;
use webthing::{BaseProperty, BaseThing, Thing};

pub struct PropertyValueForwarder {
    sensor: Sensor,
    set_type: SetReqType,
    set_message_sender: Sender<SetMessage>,
}

impl PropertyValueForwarder {
    pub fn build_message(&self, value: serde_json::Value) -> Option<SetMessage> {
        Value::build(self.set_type, value).map(|value| SetMessage {
            node_id: self.sensor.node_id as u8,
            child_sensor_id: self.sensor.child_sensor_id as u8,
            ack: 0,
            value: value,
        })
    }
}

impl ValueForwarder for PropertyValueForwarder {
    fn set_value(&mut self, value: serde_json::Value) -> Result<serde_json::Value, &'static str> {
        info!("Property forwarded {} for sensor {:?}", value, &self.sensor);
        match self.build_message(value.clone()) {
            Some(message) => match self.set_message_sender.send(message) {
                Ok(_) => Ok(value),
                Err(_) => Err("Error in setting property value"),
            },

            None => Err("Internal error while setting property"),
        }
    }
}

pub fn build_thing(
    name: String,
    sensor: Sensor,
    set_message_sender: Sender<SetMessage>,
) -> Option<(Sensor, Arc<RwLock<Box<Thing + 'static>>>)> {
    match sensor.sensor_type.is_supported() {
        true => {
            let mut thing = BaseThing::new(
                name,
                Some(sensor.sensor_type.thing_type()),
                Some(sensor.sensor_type.thing_description()),
            );
            build_properties(&sensor, set_message_sender)
                .into_iter()
                .for_each(|property| thing.add_property(Box::new(property)));
            Some((sensor, Arc::new(RwLock::new(Box::new(thing)))))
        }
        false => {
            warn!(
                "PresentationType {:?} is not handled yet!",
                sensor.sensor_type
            );
            None
        }
    }
}

fn build_properties(sensor: &Sensor, set_message_sender: Sender<SetMessage>) -> Vec<BaseProperty> {
    sensor
        .sensor_type
        .property_types()
        .into_iter()
        .map(|set_type| build_property(sensor.clone(), set_type, set_message_sender.clone()))
        .collect()
}

fn build_property(
    sensor: Sensor,
    set_type: SetReqType,
    set_message_sender: Sender<SetMessage>,
) -> BaseProperty {
    let description = json!({
        "type": set_type.data_type(),
        "description": set_type.description()
    });
    let value_forwarder: Option<Box<ValueForwarder>> = match set_type.is_forwardable() {
        true => Some(Box::new(PropertyValueForwarder {
            sensor,
            set_type,
            set_message_sender,
        })),
        false => Some(Box::new(EmptyValueForwarder)),
    };
    BaseProperty::new(
        set_type.property_name(),
        json!(true),
        value_forwarder,
        Some(description.as_object().unwrap().clone()),
    )
}
