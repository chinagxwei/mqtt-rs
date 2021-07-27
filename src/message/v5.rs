use crate::types::TypeKind;
use crate::protocol::{MqttProtocolLevel, MqttCleanSession, MqttWillFlag, MqttQos, MqttRetain};
use crate::hex::PropertyItem;

pub enum MqttMessageV5 {
    Connect(ConnectMessage),
}

#[derive(Debug, Clone)]
pub struct ConnectMessagePayload {
    pub client_id: String,
    pub will_topic: Option<String>,
    pub will_message: Option<String>,
    pub user_name: Option<String>,
    pub password: Option<String>,
    pub properties: Option<Vec<PropertyItem>>,
}

#[derive(Debug, Clone)]
pub struct ConnectMessage {
    pub msg_type: TypeKind,
    pub protocol_name: String,
    pub protocol_level: MqttProtocolLevel,
    pub clean_session: MqttCleanSession,
    pub will_flag: MqttWillFlag,
    pub will_qos: MqttQos,
    pub will_retain: MqttRetain,
    pub keep_alive: u16,
    pub payload: ConnectMessagePayload,
    pub bytes: Option<Vec<u8>>,
}
