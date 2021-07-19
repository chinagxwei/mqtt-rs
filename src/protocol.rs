use num_enum::TryFromPrimitive;

pub const MQISDP_PROTOCOL_NAME: &'static str = "MQIsdp";

pub const MQTT_PROTOCOL_NAME: &'static str = "MQTT";

#[derive(Debug, Copy, Clone, TryFromPrimitive, Ord, PartialOrd, Eq, PartialEq)]
#[repr(u8)]
pub enum MqttProtocolLevel {
    Level3_1 = 3,
    Level3_1_1 = 4,
    Level5 = 5,
}

impl MqttProtocolLevel{
    pub fn is_level_3_1(&self) -> bool {
        matches!(self, MqttProtocolLevel::Level3_1)
    }

    pub fn is_level_3_1_1(&self) -> bool {
        matches!(self, MqttProtocolLevel::Level3_1_1)
    }

    pub fn is_level_5(&self) -> bool {
        matches!(self, MqttProtocolLevel::Level5)
    }
}

#[derive(Debug, Copy, Clone, TryFromPrimitive, Ord, PartialOrd, Eq, PartialEq)]
#[repr(u8)]
pub enum MqttQos {
    Qos0 = 0,
    Qos1 = 1,
    Qos2 = 2,
}

#[derive(Debug, Copy, Clone, TryFromPrimitive, Ord, PartialOrd, Eq, PartialEq)]
#[repr(u8)]
pub enum MqttWillFlag {
    Disable = 0,
    Enable = 1,
}

#[derive(Debug, Copy, Clone, TryFromPrimitive, Ord, PartialOrd, Eq, PartialEq)]
#[repr(u8)]
pub enum MqttRetain {
    Disable = 0,
    Enable = 1,
}

#[derive(Debug, Copy, Clone, TryFromPrimitive, Ord, PartialOrd, Eq, PartialEq)]
#[repr(u8)]
pub enum MqttDup {
    Disable = 0,
    Enable = 1,
}

#[derive(Debug, Copy, Clone, TryFromPrimitive, Ord, PartialOrd, Eq, PartialEq)]
#[repr(u8)]
pub enum MqttCleanSession {
    Disable = 0,
    Enable = 1,
}

#[derive(Debug, Copy, Clone, TryFromPrimitive, Ord, PartialOrd, Eq, PartialEq)]
#[repr(u8)]
pub enum MqttWillTopic {
    Disable = 0,
    Enable = 1,
}

#[derive(Debug, Copy, Clone, TryFromPrimitive, Ord, PartialOrd, Eq, PartialEq)]
#[repr(u8)]
pub enum MqttWillMessage {
    Disable = 0,
    Enable = 1,
}

#[derive(Debug, Copy, Clone, TryFromPrimitive, Ord, PartialOrd, Eq, PartialEq)]
#[repr(u8)]
pub enum MqttUsernameFlag {
    Disable = 0,
    Enable = 1,
}

#[derive(Debug, Copy, Clone, TryFromPrimitive, Ord, PartialOrd, Eq, PartialEq)]
#[repr(u8)]
pub enum MqttPasswordFlag {
    Disable = 0,
    Enable = 1,
}

#[derive(Debug, Copy, Clone, TryFromPrimitive, Ord, PartialOrd, Eq, PartialEq)]
#[repr(u8)]
pub enum MqttSessionPresent {
    Disable = 0,
    Enable = 1,
}
