use crate::types::TypeKind;
use crate::tools::un_pack_tool::{get_type, get_type_have_data, get_protocol_name_and_version};
use crate::message::v3::{
    ConnackMessage, ConnectMessage, ConnectMessageBuild, DisconnectMessage, MqttMessageV3,
    PingrespMessage, PubackMessage, PubcompMessage, PublishMessage, PubrecMessage, PubrelMessage,
    SubackMessage, SubscribeMessage, UnsubackMessage, UnsubscribeMessage,PingreqMessage
};
use crate::protocol::MqttProtocolLevel;
use crate::message::v3::MqttMessageV3::Pubrec;

pub mod v3;

#[derive(Debug)]
pub enum MqttMessageKind {
    V3(MqttMessageV3),
    V5,
}

impl MqttMessageKind {
    pub fn is_v3(&self) -> bool {
        matches!(self, MqttMessageKind::V3(_))
    }

    pub fn get_v3(&self) -> Option<&MqttMessageV3> {
        match self {
            MqttMessageKind::V3(kind) => {
                Some(kind)
            }
            _ => { None }
        }
    }


}

impl MqttMessageKind{
    pub fn v3(base_msg: BaseMessage) -> Option<MqttMessageKind> {
        match base_msg.get_message_type() {
            TypeKind::CONNECT => { Some(Self::V3(MqttMessageV3::Connect(ConnectMessage::from(base_msg)))) }
            TypeKind::CONNACK => { Some(Self::V3(MqttMessageV3::Connack(ConnackMessage::from(base_msg)))) }
            TypeKind::PUBLISH => { Some(Self::V3(MqttMessageV3::Publish(PublishMessage::from(base_msg)))) }
            TypeKind::PUBACK => { Some(Self::V3(MqttMessageV3::Puback(PubackMessage::from(base_msg)))) }
            TypeKind::PUBREC => { Some(Self::V3(MqttMessageV3::Pubrec(PubrecMessage::from(base_msg)))) }
            TypeKind::PUBREL => { Some(Self::V3(MqttMessageV3::Pubrel(PubrelMessage::from(base_msg)))) }
            TypeKind::PUBCOMP => { Some(Self::V3(MqttMessageV3::Pubcomp(PubcompMessage::from(base_msg)))) }
            TypeKind::SUBSCRIBE => { Some(Self::V3(MqttMessageV3::Subscribe(SubscribeMessage::from(base_msg)))) }
            // TypeKind::SUBACK => { Some(Self::V3(MqttMessageV3::Suback(SubackMessage::from(base_msg)))) }
            TypeKind::UNSUBSCRIBE => { Some(Self::V3(MqttMessageV3::Unsubscribe(UnsubscribeMessage::from(base_msg)))) }
            TypeKind::UNSUBACK => { Some(Self::V3(MqttMessageV3::Unsuback(UnsubackMessage::from(base_msg)))) }
            TypeKind::PINGREQ => { Some(Self::V3(MqttMessageV3::Pingreq(PingreqMessage::from(base_msg)))) }
            TypeKind::PINGRESP => { Some(Self::V3(MqttMessageV3::Pingresp(PingrespMessage::default()))) }
            TypeKind::DISCONNECT => { Some(Self::V3(MqttMessageV3::Disconnect((DisconnectMessage::default())))) }
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
    msg_type: TypeKind,
    bytes: Vec<u8>,
}

impl MqttMessage for BaseMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl From<Vec<u8>> for BaseMessage {
    fn from(data: Vec<u8>) -> Self {
        let mut r#type = get_type(data.as_slice());
        BaseMessage { msg_type: r#type.take().unwrap(), bytes: data }
    }
}

impl From<&[u8]> for BaseMessage {
    fn from(data: &[u8]) -> Self {
        let (mut r#type, _) = get_type_have_data(data);
        let bytes = data.to_vec();
        BaseMessage { msg_type: r#type.take().unwrap(), bytes }
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
        ) = get_protocol_name_and_version(data.bytes.as_slice().get(2..).unwrap());
        BaseConnect {
            msg_type: data.msg_type,
            protocol_name: protocol_name.take().unwrap(),
            protocol_level: protocol_level.take().unwrap(),
        }
    }
}
