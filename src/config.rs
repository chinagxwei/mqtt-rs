use crate::protocol::{MqttProtocolLevel, MQTT_PROTOCOL_NAME, MqttWillFlag, MqttQos, MqttRetain};
use crate::hex::Property;

#[derive(Debug)]
pub struct Will {
    will_flag: MqttWillFlag,
    will_qos: MqttQos,
    will_retain: MqttRetain,
    will_topic: Option<String>,
    will_message: Option<String>,
}

impl Will {
    pub fn will_flag(&self) -> MqttWillFlag {
        self.will_flag
    }
    pub fn will_qos(&self) -> MqttQos {
        self.will_qos
    }
    pub fn will_retain(&self) -> MqttRetain {
        self.will_retain
    }
    pub fn will_topic(&self) -> Option<String> {
        self.will_topic.to_owned()
    }
    pub fn will_message(&self) -> Option<String> {
        self.will_message.to_owned()
    }
}

#[derive(Debug)]
pub struct Config {
    client_id: String,
    username: Option<String>,
    password: Option<String>,
    keep_alive: u32,
    protocol_name: String,
    protocol_level: MqttProtocolLevel,
    delay: u32,
    max_attempts: i32,
    will: Will,
    properties: Option<Property>,
}

impl Config {
    pub fn client_id(&self) -> String {
        self.client_id.to_owned()
    }
    pub fn username(&self) -> Option<String> {
        self.username.to_owned()
    }
    pub fn password(&self) -> Option<String> {
        self.password.to_owned()
    }
    pub fn keep_alive(&self) -> u32 {
        self.keep_alive
    }
    pub fn protocol_name(&self) -> String {
        self.protocol_name.to_owned()
    }
    pub fn protocol_level(&self) -> MqttProtocolLevel {
        self.protocol_level
    }
    pub fn delay(&self) -> u32 {
        self.delay
    }
    pub fn max_attempts(&self) -> i32 {
        self.max_attempts
    }
    pub fn will(&self) -> &Will {
        &self.will
    }
}

#[derive(Debug)]
pub struct ConfigBuilder {
    client_id: Option<String>,
    username: Option<String>,
    password: Option<String>,
    keep_alive: Option<u32>,
    protocol_name: Option<String>,
    protocol_level: Option<MqttProtocolLevel>,
    delay: Option<u32>,
    max_attempts: Option<i32>,
    will: Option<Will>,
}

impl ConfigBuilder {
    pub fn new() -> ConfigBuilder {
        ConfigBuilder {
            client_id: None,
            username: None,
            password: None,
            keep_alive: None,
            protocol_name: None,
            protocol_level: None,
            delay: None,
            max_attempts: None,
            will: None,
        }
    }

    pub fn set_client_id<S: Into<String>>(mut self, client_id: S) -> ConfigBuilder {
        self.client_id = Option::from(client_id.into());
        self
    }

    pub fn set_username<S: Into<String>>(mut self, username: S) -> ConfigBuilder {
        self.username = Option::from(username.into());
        self
    }
    pub fn set_password<S: Into<String>>(mut self, password: S) -> ConfigBuilder {
        self.password = Option::from(password.into());
        self
    }
    pub fn set_keep_alive(mut self, keep_alive: u32) -> ConfigBuilder {
        self.keep_alive = Option::from(keep_alive);
        self
    }

    pub fn set_protocol_name<S: Into<String>>(mut self, protocol_name: S) -> ConfigBuilder {
        self.protocol_name = Option::from(protocol_name.into());
        self
    }
    pub fn set_protocol_level(mut self, protocol_level: MqttProtocolLevel) -> ConfigBuilder {
        self.protocol_level = Option::from(protocol_level);
        self
    }
    pub fn set_delay(mut self, delay: u32) -> ConfigBuilder {
        self.delay = Option::from(delay);
        self
    }
    pub fn set_max_attempts(mut self, max_attempts: i32) -> ConfigBuilder {
        self.max_attempts = Option::from(max_attempts);
        self
    }

    pub fn set_will(mut self, will: Will) -> ConfigBuilder {
        self.will = Option::from(will);
        self
    }

    fn check(&self) -> bool {
        self.client_id.is_some() &&
            self.keep_alive.is_some() &&
            self.protocol_name.is_some() &&
            self.protocol_level.is_some() &&
            self.delay.is_some() &&
            self.max_attempts.is_some()
    }

    pub fn build(mut self) -> Option<Config> {
        if !self.check() {
            return None;
        }

        Some(
            Config {
                client_id: self.client_id.take().unwrap(),
                username: self.username.take(),
                password: self.username.take(),
                keep_alive: self.keep_alive.take().unwrap(),
                protocol_name: self.protocol_name.take().unwrap(),
                protocol_level: self.protocol_level.take().unwrap(),
                delay: self.delay.take().unwrap(),
                max_attempts: self.max_attempts.take().unwrap(),
                will: self.will.take().unwrap(),
                properties: None
            }
        )
    }
}


impl Default for ConfigBuilder {
    fn default() -> Self {
        ConfigBuilder {
            client_id: Some(String::from("rs-mqtt-test")),
            username: None,
            password: None,
            keep_alive: Some(60),
            protocol_name: Some(MQTT_PROTOCOL_NAME.to_owned()),
            protocol_level: Some(MqttProtocolLevel::Level3_1_1),
            delay: Some(3000),
            max_attempts: Some(-1),
            will: Option::from(Will {
                will_flag: MqttWillFlag::Disable,
                will_qos: MqttQos::Qos0,
                will_retain: MqttRetain::Disable,
                will_topic: None,
                will_message: None,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        println!("{:?}", ConfigBuilder::default());
    }
}
