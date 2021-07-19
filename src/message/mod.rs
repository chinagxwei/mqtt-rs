use crate::types::TypeKind;
use crate::tools::un_pack_tool::{get_type, get_type_have_data, get_protocol_name_and_version};
use crate::message::v3::MqttMessageV3;
use crate::protocol::MqttProtocolLevel;

pub mod v3;

#[derive(Debug)]
pub enum MqttMessageKind {
    // Connect,
    V3(MqttMessageV3),
    V5,
}

impl MqttMessageKind {
    // pub fn is_connect(&self) -> bool {
    //     matches!(self, MqttMessageKind::Connect)
    // }
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
        // let mut r#type = get_type(data);
        // println!("{:?}", get_type_have_data(data));
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

impl BaseConnect{
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
