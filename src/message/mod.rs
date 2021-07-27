use crate::types::TypeKind;
use crate::tools::un_pack_tool::{get_type, get_protocol_name_and_version};
use crate::message::v3::{
    ConnackMessage, ConnectMessage, DisconnectMessage, MqttMessageV3,
    PingrespMessage, PubackMessage, PubcompMessage, PublishMessage, PubrecMessage, PubrelMessage,
    SubackMessage, SubscribeMessage, UnsubackMessage, UnsubscribeMessage, PingreqMessage,
};
use crate::protocol::{MqttProtocolLevel, MqttDup, MqttQos, MqttRetain};
use crate::message::v3::MqttMessageV3::Pubrec;
use crate::TopicMessage;

pub mod v3;
pub mod v5;

#[derive(Debug)]
pub enum MqttMessageKind {
    Response(Vec<u8>),
    RequestV3(MqttMessageV3),
    RequestV5,
}

impl MqttMessageKind {
    // pub fn is_topic(&self) -> bool {
    //     matches!(self, MqttMessageKind::Topic(_))
    // }

    pub fn is_v3(&self) -> bool {
        matches!(self, MqttMessageKind::RequestV3(_))
    }

    pub fn get_v3(&self) -> Option<&MqttMessageV3> {
        match self {
            MqttMessageKind::RequestV3(kind) => {
                Some(kind)
            }
            _ => { None }
        }
    }
}

impl MqttMessageKind {
    pub fn v3(base_msg: BaseMessage) -> Option<MqttMessageKind> {
        match base_msg.get_message_type() {
            TypeKind::CONNECT => { Some(Self::RequestV3(MqttMessageV3::Connect(ConnectMessage::from(base_msg)))) }
            TypeKind::CONNACK => { Some(Self::RequestV3(MqttMessageV3::Connack(ConnackMessage::from(base_msg)))) }
            TypeKind::PUBLISH => { Some(Self::RequestV3(MqttMessageV3::Publish(PublishMessage::from(base_msg)))) }
            TypeKind::PUBACK => { Some(Self::RequestV3(MqttMessageV3::Puback(PubackMessage::from(base_msg)))) }
            TypeKind::PUBREC => { Some(Self::RequestV3(MqttMessageV3::Pubrec(PubrecMessage::from(base_msg)))) }
            TypeKind::PUBREL => { Some(Self::RequestV3(MqttMessageV3::Pubrel(PubrelMessage::from(base_msg)))) }
            TypeKind::PUBCOMP => { Some(Self::RequestV3(MqttMessageV3::Pubcomp(PubcompMessage::from(base_msg)))) }
            TypeKind::SUBSCRIBE => { Some(Self::RequestV3(MqttMessageV3::Subscribe(SubscribeMessage::from(base_msg)))) }
            // TypeKind::SUBACK => { Some(Self::RequestV3(MqttMessageV3::Suback(SubackMessage::from(base_msg)))) }
            TypeKind::UNSUBSCRIBE => { Some(Self::RequestV3(MqttMessageV3::Unsubscribe(UnsubscribeMessage::from(base_msg)))) }
            TypeKind::UNSUBACK => { Some(Self::RequestV3(MqttMessageV3::Unsuback(UnsubackMessage::from(base_msg)))) }
            // TypeKind::PINGREQ => { Some(Self::RequestV3(MqttMessageV3::Pingreq(PingreqMessage::from(base_msg)))) }
            TypeKind::PINGREQ => { Some(Self::RequestV3(MqttMessageV3::Pingresp(PingrespMessage::default()))) }
            TypeKind::DISCONNECT => { Some(Self::RequestV3(MqttMessageV3::Disconnect((DisconnectMessage::default())))) }
            TypeKind::AUTH => { None }
            _ => { None }
        }
    }
}


pub trait MqttMessage {
    fn get_message_type(&self) -> TypeKind;
}

pub trait MqttBytesMessage: MqttMessage {
    fn as_bytes(&self) -> &[u8];
}

#[derive(Debug)]
pub struct BaseMessage {
    pub msg_type: TypeKind,
    pub dup: Option<MqttDup>,
    pub qos: Option<MqttQos>,
    pub retain: Option<MqttRetain>,
    pub bytes: Vec<u8>,
}

impl MqttMessage for BaseMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl From<Vec<u8>> for BaseMessage {
    fn from(data: Vec<u8>) -> Self {
        let (mut r#type2, retain, qos, dup, last_bytes) = get_type(data.as_slice());
        BaseMessage { msg_type: r#type2.take().unwrap(), dup, qos, retain, bytes: last_bytes.to_vec() }
    }
}

impl From<&[u8]> for BaseMessage {
    fn from(data: &[u8]) -> Self {
        let (mut r#type2, retain, qos, dup, last_bytes) = get_type(data);
        BaseMessage { msg_type: r#type2.take().unwrap(), dup, qos, retain, bytes: last_bytes.to_vec() }
    }
}

#[derive(Debug)]
pub struct BaseConnect {
    msg_type: TypeKind,
    protocol_name: String,
    protocol_level: MqttProtocolLevel,
}

impl BaseConnect {
    pub fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }

    pub fn get_protocol_level(&self) -> MqttProtocolLevel {
        self.protocol_level
    }

    pub fn get_protocol_name(&self) -> String {
        self.protocol_name.to_owned()
    }
}

impl From<&BaseMessage> for BaseConnect {
    fn from(data: &BaseMessage) -> Self {
        let (
            mut protocol_name,
            mut protocol_level
        ) = get_protocol_name_and_version(data.bytes.as_slice());
        BaseConnect {
            msg_type: data.msg_type,
            protocol_name: protocol_name.take().unwrap(),
            protocol_level: protocol_level.take().unwrap(),
        }
    }
}
