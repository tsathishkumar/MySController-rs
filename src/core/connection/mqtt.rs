use rumqtt::{MqttClient, MqttOptions, QoS, ReconnectOptions};
use rumqtt::client::Notification;
use std::time::Duration;
use super::Connection;
use std::io::{Result, Error, ErrorKind};
use crossbeam_channel::Receiver;

struct MySMessage;

impl MySMessage {
    fn messge(notification: Notification) -> String {
        if let Notification::Publish(message) = notification {
            let topic = message.topic_name;
            let payload = std::str::from_utf8(message.payload.as_slice()).unwrap();

            let mut message_parts = topic.trim().split('/').collect::<Vec<&str>>();
            message_parts.remove(0);
            message_parts.push(payload);
            message_parts.join(";")
        } else {
            String::new()
        }
    }

    fn topic_and_payload(publish_topic_prefix: String, line: String) -> (String, String) {
        let mut message_parts = line.trim().split(';').collect::<Vec<&str>>();

        if message_parts.len() != 6 {
            return (String::new(), String::new());
        }
        let payload = message_parts.pop().unwrap().to_owned();
        let topic = [publish_topic_prefix.as_str(),"-in"].join("");
        message_parts.insert(0, topic.as_str());
        (message_parts.join("/"), payload)
    }
}

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

impl Connection for MqttConnection {

    fn read_line(&mut self) -> Result<String> {
        let message = self.notifications.recv();
        if let Ok(msg) = message {
            return Result::Ok(MySMessage::messge(msg));
        }
        Result::Err(Error::new(ErrorKind::Other, "Not able to read from MQTT Client"))
    }

    fn write_line(&mut self, line: &str) -> Result<usize> {
        let (topic, message) = MySMessage::topic_and_payload(self.publish_topic_prefix.clone(), line.to_owned());
        self.mqtt_client.publish(topic, QoS::AtLeastOnce, false, message).unwrap();
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

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn should_get_topic_and_payload_in_mqtt_format() {
        let message_string = "1;255;4;0;0;0A0001005000D4460102\n".to_owned();
        let (topic, payload) = MySMessage::topic_and_payload("prefix".to_owned(), message_string);
        assert_eq!(topic, "prefix-in/1/255/4/0/0");
        assert_eq!(payload, "0A0001005000D4460102");
    }

    #[test]
    fn should_get_mysensors_message_from_mqtt_notification() {
        let notification = Notification::Publish(rumqtt::Publish{dup: true,
            qos: QoS::AtLeastOnce,
            retain: false,
            topic_name: "prefix-out/1/255/4/0/0".to_owned(),
            pkid: None,
            payload: Arc::new(b"0A0001005000D4460102".to_vec())}
        );
        let message = MySMessage::messge(notification);
        assert_eq!(message, "1;255;4;0;0;0A0001005000D4460102");
    }
}