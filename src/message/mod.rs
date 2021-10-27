use crate::tools::types::TypeKind;
use crate::tools::un_pack_tool::get_type;
use crate::tools::protocol::{MqttProtocolLevel, MqttDup, MqttQos, MqttRetain, MqttCleanSession, MqttWillFlag, MqttPasswordFlag, MqttUsernameFlag};
use crate::hex::PropertyItem;
use crate::message::entity::{DisconnectMessage, PingrespMessage};
use crate::message::v3::MqttMessageV3;
use crate::packet::{v3_unpacket, v5_unpacket};
use crate::message::v5::MqttMessageV5;

pub mod v3;
pub mod v5;
pub mod entity;

#[derive(Debug)]
pub enum MqttMessageKind {
    RequestV3(MqttMessageV3),
    RequestV3Vec(Vec<MqttMessageV3>),
    RequestV5(MqttMessageV5),
    RequestV5Vec(Vec<MqttMessageV5>),
}

impl MqttMessageKind {
    pub fn is_v3(&self) -> bool {
        matches!(self, MqttMessageKind::RequestV3(_))
    }

    pub fn is_v3s(&self) -> bool {
        matches!(self, MqttMessageKind::RequestV3Vec(_))
    }

    pub fn is_v5(&self) -> bool {
        matches!(self, MqttMessageKind::RequestV5(_))
    }

    pub fn is_v5s(&self) -> bool {
        matches!(self, MqttMessageKind::RequestV5Vec(_))
    }

    pub fn get_v3(&self) -> Option<&MqttMessageV3> {
        match self {
            MqttMessageKind::RequestV3(kind) => {
                Some(kind)
            }
            _ => { None }
        }
    }

    pub fn get_v5(&self) -> Option<&MqttMessageV5> {
        match self {
            MqttMessageKind::RequestV5(kind) => {
                Some(kind)
            }
            _ => { None }
        }
    }

    pub fn get_v3s(&self) -> Option<&Vec<MqttMessageV3>> {
        match self {
            MqttMessageKind::RequestV3Vec(kind) => {
                Some(kind)
            }
            _ => { None }
        }
    }

    pub fn get_v5s(&self) -> Option<&Vec<MqttMessageV5>> {
        match self {
            MqttMessageKind::RequestV5Vec(kind) => {
                Some(kind)
            }
            _ => { None }
        }
    }
}

impl MqttMessageKind {
    pub fn to_v3_request(base_msg: BaseMessage) -> Option<MqttMessageKind> {
        match base_msg.get_message_type() {
            TypeKind::CONNECT => { Some(Self::RequestV3(v3_unpacket::connect(base_msg))) }
            TypeKind::CONNACK => { Some(Self::RequestV3(v3_unpacket::connack(base_msg))) }
            TypeKind::PUBLISH => { Some(Self::RequestV3(v3_unpacket::publish(base_msg))) }
            TypeKind::PUBACK => { Some(Self::RequestV3(v3_unpacket::puback(base_msg))) }
            TypeKind::PUBREC => { Some(Self::RequestV3(v3_unpacket::pubrec(base_msg))) }
            TypeKind::PUBREL => { Some(Self::RequestV3(v3_unpacket::pubrel(base_msg))) }
            TypeKind::PUBCOMP => { Some(Self::RequestV3(v3_unpacket::pubcomp(base_msg))) }
            TypeKind::SUBSCRIBE => { Some(Self::RequestV3Vec(v3_unpacket::subscribe(base_msg))) }
            TypeKind::SUBACK => { Some(Self::RequestV3(v3_unpacket::suback(base_msg))) }
            TypeKind::UNSUBSCRIBE => { Some(Self::RequestV3Vec(v3_unpacket::unsubscribe(base_msg))) }
            TypeKind::UNSUBACK => { Some(Self::RequestV3(v3_unpacket::unsuback(base_msg))) }
            TypeKind::PINGREQ => { Some(Self::RequestV3(MqttMessageV3::Pingresp(PingrespMessage::from(base_msg)))) }
            TypeKind::PINGRESP => { Some(Self::RequestV3(MqttMessageV3::Pingresp(PingrespMessage::from(base_msg)))) }
            TypeKind::DISCONNECT => { Some(Self::RequestV3(MqttMessageV3::Disconnect((DisconnectMessage::default())))) }
            _ => { None }
        }
    }
}

impl MqttMessageKind {
    pub fn to_v5_request(base_msg: BaseMessage) -> Option<MqttMessageKind> {
        match base_msg.msg_type {
            TypeKind::CONNECT => { Some(Self::RequestV5(v5_unpacket::connect(base_msg))) }
            TypeKind::CONNACK => { Some(Self::RequestV5(v5_unpacket::connack(base_msg))) }
            TypeKind::PUBLISH => { Some(Self::RequestV5(v5_unpacket::publish(base_msg))) }
            TypeKind::PUBACK => { Some(Self::RequestV5(v5_unpacket::puback(base_msg))) }
            TypeKind::PUBREC => { Some(Self::RequestV5(v5_unpacket::pubrec(base_msg))) }
            TypeKind::PUBREL => { Some(Self::RequestV5(v5_unpacket::pubrel(base_msg))) }
            TypeKind::PUBCOMP => { Some(Self::RequestV5(v5_unpacket::pubcomp(base_msg))) }
            TypeKind::SUBSCRIBE => { Some(Self::RequestV5Vec(v5_unpacket::subscribe(base_msg))) }
            TypeKind::SUBACK => { Some(Self::RequestV5(v5_unpacket::suback(base_msg))) }
            TypeKind::UNSUBSCRIBE => { Some(Self::RequestV5Vec(v5_unpacket::unsubscribe(base_msg))) }
            TypeKind::UNSUBACK => { Some(Self::RequestV5(v5_unpacket::unsuback(base_msg))) }
            TypeKind::PINGREQ => { Some(Self::RequestV5(MqttMessageV5::Pingresp(PingrespMessage::from(base_msg)))) }
            TypeKind::PINGRESP => { Some(Self::RequestV5(MqttMessageV5::Pingresp(PingrespMessage::from(base_msg)))) }
            TypeKind::DISCONNECT => { Some(Self::RequestV5(v5_unpacket::disconnect(base_msg))) }
            TypeKind::AUTH => { Some(Self::RequestV5(v5_unpacket::auth(base_msg))) }
        }
    }
}

pub trait MqttMessageType {
    fn get_message_type(&self) -> TypeKind;
}

pub trait MqttMessageVersion {
    fn version(&self) -> Option<MqttProtocolLevel>;
}

pub trait MqttBytesMessage: MqttMessageType {
    fn to_vec(&self) -> Vec<u8>;
}


#[derive(Debug)]
pub struct BaseMessage {
    pub msg_type: TypeKind,
    pub dup: Option<MqttDup>,
    pub qos: Option<MqttQos>,
    pub retain: Option<MqttRetain>,
    pub bytes: Vec<u8>,
}

impl MqttMessageType for BaseMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl From<Vec<u8>> for BaseMessage {
    fn from(data: Vec<u8>) -> Self {
        let (r#type2, retain, qos, dup, _last_bytes) = get_type(data.as_slice());
        BaseMessage { msg_type: r#type2.unwrap(), dup, qos, retain, bytes: data }
    }
}

impl From<&[u8]> for BaseMessage {
    fn from(data: &[u8]) -> Self {
        let (r#type2, retain, qos, dup, _last_bytes) = get_type(data);
        BaseMessage { msg_type: r#type2.unwrap(), dup, qos, retain, bytes: data.to_vec() }
    }
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

#[derive(Debug)]
pub struct VariableHeader {
    pub protocol_name: Option<String>,
    pub keep_alive: Option<u16>,
    pub protocol_level: Option<MqttProtocolLevel>,
    pub clean_session: Option<MqttCleanSession>,
    pub will_flag: Option<MqttWillFlag>,
    pub will_qos: Option<MqttQos>,
    pub will_retain: Option<MqttRetain>,
    pub password_flag: Option<MqttPasswordFlag>,
    pub username_flag: Option<MqttUsernameFlag>,
}

pub trait WillField {
    fn topic_str(&self) -> Option<&String>;
    fn message_str(&self) -> Option<&String>;
    fn username_str(&self) -> Option<&String>;
    fn password_str(&self) -> Option<&String>;
}
