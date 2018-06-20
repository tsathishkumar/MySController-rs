use channel::Sender;
use core::message::presentation::PresentationType;
use core::message::set::SetMessage;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use model::node::Node;
use model::sensor::Sensor;
use serde_json;
use std::sync::{Arc, RwLock, Weak};
use std::thread;
use webthing::property::ValueForwarder;
use webthing::server::ActionGenerator;
use webthing::{Action, BaseProperty, BaseThing, Thing, ThingsType, WebThingServer};

struct Generator;

impl ActionGenerator for Generator {
    fn generate(
        &self,
        _thing: Weak<RwLock<Box<Thing>>>,
        name: String,
        input: Option<&serde_json::Value>,
    ) -> Option<Box<Action>> {
        let _input = match input {
            Some(v) => match v.as_object() {
                Some(o) => Some(o.clone()),
                None => None,
            },
            None => None,
        };

        let name: &str = &name;
        match name {
            _ => None,
        }
    }
}

struct OnValueForwarder {
    sensor: Sensor,
    set_message_sender: Sender<SetMessage>,
}

impl ValueForwarder for OnValueForwarder {
    fn set_value(&mut self, value: serde_json::Value) -> Result<serde_json::Value, &'static str> {
        info!("On-State is now {} for sensor {:?}", value, &self.sensor);
        match value {
            serde_json::Value::Bool(status) => {
                let status_message = self.sensor.to_set_status_message(status);
                match self.set_message_sender.send(status_message) {
                    Ok(_) => (),
                    Err(e) => error!("Error while sending to set message handler {:?}", e),
                }
            }
            _ => (),
        }
        Ok(value)
    }
}

/// An on off light that logs received commands to stdout.
fn make_light(
    name: String,
    sensor: Sensor,
    set_message_sender: Sender<SetMessage>,
) -> Arc<RwLock<Box<Thing + 'static>>> {
    let mut thing = BaseThing::new(
        name,
        Some("onOffLight".to_owned()),
        Some("A web connected lamp".to_owned()),
    );

    let on_description = json!({
        "type": "boolean",
        "description": "Whether the lamp is turned on"
    });
    let on_description = on_description.as_object().unwrap().clone();
    thing.add_property(Box::new(BaseProperty::new(
        "on".to_owned(),
        json!(true),
        Some(Box::new(OnValueForwarder {
            sensor,
            set_message_sender,
        })),
        Some(on_description),
    )));

    Arc::new(RwLock::new(Box::new(thing)))
}

pub fn start_server(
    pool: Pool<ConnectionManager<SqliteConnection>>,
    set_message_sender: Sender<SetMessage>,
) {
    let mut sensor_list: Vec<Sensor> = vec![];
    let mut node_list: Vec<Node> = vec![];

    {
        match pool.get() {
            Ok(conn) => {
                use model::node::nodes::dsl::*;
                use model::sensor::sensors::dsl::*;

                match nodes.load::<Node>(&conn) {
                    Ok(existing_nodes) => node_list = existing_nodes,
                    Err(e) => error!("Error while trying to get nodes {:?}", e),
                };
                match sensors.load::<Sensor>(&conn) {
                    Ok(existing_sensors) => sensor_list = existing_sensors,
                    Err(e) => error!("Error while trying to get sensors {:?}", e),
                };
            }
            Err(e) => error!("Error while trying to get db connection {:?}", e),
        }
    }

    thread::spawn(move || {
        let mut things: Vec<Arc<RwLock<Box<Thing + 'static>>>> = Vec::new();

        for sensor in sensor_list {
            // Create a thing that represents a dimmable light
            let node_name = (&node_list)
                .into_iter()
                .find(|node| node.node_id == sensor.node_id)
                .map(|node| node.node_name.clone())
                .unwrap();
            match sensor.sensor_type {
                PresentationType::Binary => {
                    let light = make_light(
                        format!("{} - {}", node_name, sensor.child_sensor_id).to_owned(),
                        sensor,
                        set_message_sender.clone(),
                    );
                    things.push(light.clone());
                }
                _other => warn!("PresentationType {:?} is not handled yet!", _other),
            }
        }
        let server = WebThingServer::new(
            ThingsType::Multiple(things, "LightAndTempDevice".to_owned()),
            Some(8888),
            None,
            Box::new(Generator),
        );
        server.start();
        //server.stop();
    });
}
