pub mod adapter;

use channel::Sender;
use core::message::set::SetMessage;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use model::node::Node;
use model::sensor::Sensor;
use serde_json;
use std::sync::{Arc, RwLock, Weak};
use std::thread;

use webthing::server::ActionGenerator;
use webthing::{Action, Thing, ThingsType, WebThingServer};

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
            match (&node_list)
                .into_iter()
                .find(|node| node.node_id == sensor.node_id)
                .map(|node| node.node_name.clone())
            {
                Some(node_name) => {
                    let thing = adapter::build_thing(
                        format!("{} - {}", node_name, sensor.child_sensor_id).to_owned(),
                        sensor,
                        set_message_sender.clone(),
                    );
                    match thing {
                        Some(thing) => things.push(thing.clone()),
                        None => (),
                    }
                }
                None => (),
            }
        }
        if !things.is_empty() {
            let server = WebThingServer::new(
                ThingsType::Multiple(things, "LightAndTempDevice".to_owned()),
                Some(8888),
                None,
                Box::new(Generator),
            );
            server.start();
        }
    });
}
