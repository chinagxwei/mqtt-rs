use crate::types::TypeKind;
use crate::protocol::{MqttProtocolLevel, MqttCleanSession, MqttWillFlag, MqttQos, MqttRetain, MqttSessionPresent, MqttDup, MqttRetainAsPublished, MqttNoLocal};
use crate::hex::{PropertyItem, Property, PropertyValue};
use crate::message::{ConnectMessagePayload, BaseMessage, MqttMessage, MqttBytesMessage};
use crate::packet::{v5_packet, v5_unpacket};
use crate::hex::reason_code::{ReasonPhrases, ReasonCodeV5};

pub enum MqttMessageV5 {
    Connect(ConnectMessage),
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
    pub properties: Option<Vec<PropertyItem>>,
    pub bytes: Option<Vec<u8>>,
}

impl From<BaseMessage> for ConnectMessage {
    fn from(base: BaseMessage) -> Self {
        v5_unpacket::connect(base)
    }
}

impl MqttMessage for ConnectMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for ConnectMessage {
    fn as_bytes(&self) -> &[u8] {
        self.bytes.as_ref().unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct ConnackMessage {
    pub msg_type: TypeKind,
    pub session_present: MqttSessionPresent,
    pub return_code: u8,
    pub properties: Option<Vec<PropertyItem>>,
    pub bytes: Vec<u8>,
}

impl MqttMessage for ConnackMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for ConnackMessage {
    fn as_bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }
}

impl From<BaseMessage> for ConnackMessage {
    fn from(base: BaseMessage) -> Self {
        v5_unpacket::connack(base)
    }
}

impl Default for ConnackMessage {
    fn default() -> Self {
        let properties = Some(
            vec![
                PropertyItem(Property::MaximumPacketSize, PropertyValue::Long(1048576)),
                PropertyItem(Property::RetainAvailable, PropertyValue::Byte(1)),
                PropertyItem(Property::SharedSubscriptionAvailable, PropertyValue::Byte(1)),
                PropertyItem(Property::SubscriptionIdentifierAvailable, PropertyValue::Byte(1)),
                PropertyItem(Property::TopicAliasMaximum, PropertyValue::Short(65535)),
                PropertyItem(Property::WildcardSubscriptionAvailable, PropertyValue::Byte(1)),
            ]
        );
        let bytes = v5_packet::connack(
            MqttSessionPresent::Disable,
            ReasonCodeV5::ReasonPhrases(ReasonPhrases::Success),
            properties.as_ref(),
        );
        ConnackMessage {
            msg_type: TypeKind::CONNACK,
            session_present: MqttSessionPresent::Disable,
            return_code: ReasonPhrases::Success.as_byte(),
            properties,
            bytes,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PublishMessage {
    pub msg_type: TypeKind,
    pub message_id: u16,
    pub topic: String,
    pub dup: MqttDup,
    pub qos: MqttQos,
    pub retain: MqttRetain,
    pub msg_body: String,
    pub properties: Option<Vec<PropertyItem>>,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessage for PublishMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for PublishMessage {
    fn as_bytes(&self) -> &[u8] {
        self.bytes.as_ref().unwrap()
    }
}

impl From<BaseMessage> for PublishMessage {
    fn from(base: BaseMessage) -> Self {
        v5_unpacket::publish(base)
    }
}

#[derive(Debug, Clone)]
pub struct SubscribeTopic {
    pub topic: String,
    pub qos: Option<MqttQos>,
    pub no_local: Option<MqttNoLocal>,
    pub retain_as_published: Option<MqttRetainAsPublished>,
    pub retain_handling: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct SubscribeMessage {
    pub msg_type: TypeKind,
    pub message_id: u16,
    pub topics: Vec<SubscribeTopic>,
    pub properties: Option<Vec<PropertyItem>>,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessage for SubscribeMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for SubscribeMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_ref().unwrap()
    }
}

impl From<BaseMessage> for SubscribeMessage {
    fn from(base: BaseMessage) -> Self {
        v5_unpacket::subscribe(base)
    }
}

#[derive(Debug, Clone)]
pub struct UnsubscribeMessage {
    pub msg_type: TypeKind,
    pub message_id: u16,
    pub topics: Vec<SubscribeTopic>,
    pub properties: Option<Vec<PropertyItem>>,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessage for UnsubscribeMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for UnsubscribeMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_ref().unwrap()
    }
}

impl From<BaseMessage> for UnsubscribeMessage {
    fn from(base: BaseMessage) -> Self {
        v5_unpacket::unsubscribe(base)
    }
}

#[derive(Debug, Clone)]
pub struct SubackMessage {
    pub msg_type: TypeKind,
    pub message_id: u16,
    pub codes: Vec<u8>,
    pub properties: Option<Vec<PropertyItem>>,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessage for SubackMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for SubackMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_ref().unwrap()
    }
}

impl From<BaseMessage> for SubackMessage {
    fn from(base: BaseMessage) -> Self {
        v5_unpacket::suback(base)
    }
}

#[derive(Debug, Clone)]
pub struct UnsubackMessage {
    pub msg_type: TypeKind,
    pub message_id: u16,
    pub codes: Vec<u8>,
    pub properties: Option<Vec<PropertyItem>>,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessage for UnsubackMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for UnsubackMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_ref().unwrap()
    }
}

impl From<BaseMessage> for UnsubackMessage {
    fn from(base: BaseMessage) -> Self {
        v5_unpacket::unsuback(base)
    }
}

#[derive(Debug, Clone)]
pub struct DisconnectMessage {
    pub msg_type: TypeKind,
    pub code: u8,
    pub properties: Option<Vec<PropertyItem>>,
    pub bytes: Vec<u8>,
}

impl MqttMessage for DisconnectMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for DisconnectMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_slice()
    }
}

#[derive(Debug, Clone)]
pub struct AuthMessage {
    pub msg_type: TypeKind,
    pub code: u8,
    pub properties: Option<Vec<PropertyItem>>,
    pub bytes: Vec<u8>,
}

impl MqttMessage for AuthMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for AuthMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_slice()
    }
}

pub struct CommonPayload {
    pub msg_type: TypeKind,
    pub code: u8,
    pub properties: Option<Vec<PropertyItem>>,
    pub bytes: Vec<u8>,
}

impl MqttMessage for CommonPayload {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for CommonPayload {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_slice()
    }
}
