use rumqtt::{MqttClient, MqttOptions, QoS, ReconnectOptions};
use rumqtt::client::Notification;
use std::time::Duration;
use super::Connection;
use std::io::{Result, Error, ErrorKind};
use crossbeam_channel::Receiver;

pub struct MqttConnection {
    broker: String,
    port: u16,
    publish_topic_prefix: String,
    mqtt_client: MqttClient,
    notifications: Receiver<Notification>,
}

impl MqttConnection {
    pub fn new(broker: String, port: u16, publish_topic_prefix: String, id: &str) -> MqttConnection {
        let reconnection_options = ReconnectOptions::Always(10);

        let mqtt_options = MqttOptions::new(id, broker.clone(), port)
                                        .set_keep_alive(10)
                                        .set_request_channel_capacity(3)
                                        .set_reconnect_opts(reconnection_options)
                                        .set_clean_session(false);

        let (mut mqtt_client, notifications) = MqttClient::start(mqtt_options).unwrap();
        let mut subsribe_topic = publish_topic_prefix.clone();
        subsribe_topic.push_str("-out/#");
        mqtt_client.subscribe(subsribe_topic, QoS::AtLeastOnce).unwrap();

        MqttConnection {
            broker, port,publish_topic_prefix,mqtt_client, notifications
        }
    }
}

fn mqtt_message_to_mys_message(message: Notification) -> String {
    if let Notification::Publish(message) = message {
        let topic = message.topic_name;
        let payload = message.payload.as_slice();

        let mut mys_message = String::from(topic);
        mys_message.push_str(String::from_utf8(payload.to_owned()).unwrap().as_str());
        mys_message
    } else {
        String::new()
    }
}

impl Connection for MqttConnection {

    fn read_line(&mut self) -> Result<String> {
        let message = self.notifications.recv();
        if let Ok(msg) = message {
            return Result::Ok(mqtt_message_to_mys_message(msg));
        }
        Result::Err(Error::new(ErrorKind::Other, "Not able to read from MQTT Client"))
    }

    fn write_line(&mut self, line: &str) -> Result<usize> {
        self.mqtt_client.publish("hello/world", QoS::AtLeastOnce, false, line).unwrap();
        Result::Ok(line.len())
    }

    fn clone(&self) -> Box<dyn Connection> {
        Box::new(MqttConnection::new(self.broker.clone(), self.port, self.publish_topic_prefix.clone(), "myrs_controller_writer"))
    }

    fn host(&self) -> &String {
        &self.broker
    }

    fn timeout(&mut self, _: Duration) {
        //TODO: Handle timeout
    }
}