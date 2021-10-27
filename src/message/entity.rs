use crate::hex::PropertyItem;
use crate::hex::reason_code::{ReasonCodes, ReasonPhrases};
use crate::message::{BaseMessage, ConnectMessagePayload, MqttMessageType, WillField};
use crate::tools::config::Config;
use crate::tools::pack_tool::pack_header;
use crate::tools::protocol::{MqttCleanSession, MqttDup, MqttNoLocal, MqttProtocolLevel, MqttQos, MqttRetain, MqttRetainAsPublished, MqttSessionPresent, MqttWillFlag};
use crate::tools::types::TypeKind;

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

impl MqttMessageType for ConnectMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl ConnectMessage {
    pub fn new(clean_session: MqttCleanSession, config: Config) -> ConnectMessage {
        ConnectMessage {
            msg_type: TypeKind::CONNECT,
            protocol_name: config.protocol_name(),
            protocol_level: config.protocol_level(),
            clean_session,
            will_flag: config.will().will_flag(),
            will_qos: config.will().will_qos(),
            will_retain: config.will().will_retain(),
            keep_alive: config.keep_alive(),
            payload: ConnectMessagePayload {
                client_id: config.client_id(),
                will_topic: config.will().will_topic(),
                will_message: config.will().will_message(),
                user_name: config.username(),
                password: config.password(),
                properties: None,
            },
            properties: None,
            bytes: None,
        }
    }
}

impl WillField for ConnectMessage {
    fn topic_str(&self) -> Option<&String> {
        self.payload.will_topic.as_ref()
    }

    fn message_str(&self) -> Option<&String> {
        self.payload.will_message.as_ref()
    }

    fn username_str(&self) -> Option<&String> {
        self.payload.user_name.as_ref()
    }

    fn password_str(&self) -> Option<&String> {
        self.payload.password.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct ConnackMessage {
    pub msg_type: TypeKind,
    pub protocol_level: Option<MqttProtocolLevel>,
    pub session_present: MqttSessionPresent,
    pub return_code: Option<u8>,
    pub properties: Option<Vec<PropertyItem>>,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessageType for ConnackMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl Default for ConnackMessage {
    fn default() -> Self {
        ConnackMessage {
            msg_type: TypeKind::CONNACK,
            protocol_level: None,
            session_present: MqttSessionPresent::Disable,
            return_code: None,
            properties: None,
            bytes: None,
        }
    }
}

impl ConnackMessage {
    pub fn new(session_present: MqttSessionPresent, return_code: ReasonCodes) -> ConnackMessage {
        ConnackMessage {
            msg_type: TypeKind::CONNACK,
            protocol_level: None,
            session_present,
            return_code: Some(return_code.as_byte()),
            properties: None,
            bytes: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SubscribeMessage {
    pub msg_type: TypeKind,
    pub protocol_level: Option<MqttProtocolLevel>,
    pub message_id: u16,
    pub topic: String,
    pub qos: Option<MqttQos>,
    pub no_local: Option<MqttNoLocal>,
    pub retain_as_published: Option<MqttRetainAsPublished>,
    pub retain_handling: Option<u8>,
    pub properties: Option<Vec<PropertyItem>>,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessageType for SubscribeMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl SubscribeMessage {
    pub fn new(message_id: u16, topic: String, qos: MqttQos) -> Self {
        SubscribeMessage {
            msg_type: TypeKind::SUBSCRIBE,
            protocol_level: None,
            message_id,
            topic,
            qos:Some(qos),
            no_local: None,
            retain_as_published: None,
            retain_handling: None,
            properties: None,
            bytes: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SubackMessage {
    pub msg_type: TypeKind,
    pub protocol_level: Option<MqttProtocolLevel>,
    pub message_id: u16,
    pub codes: Vec<u8>,
    pub properties: Option<Vec<PropertyItem>>,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessageType for SubackMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl SubackMessage {
    pub fn new(message_id: u16, qos: MqttQos) -> Self {
        let codes = if (qos as u32) < 3 {
            qos.as_byte().to_ne_bytes().to_vec()
        } else {
            MqttQos::Failure.as_byte().to_ne_bytes().to_vec()
        };
        SubackMessage {
            msg_type: TypeKind::SUBACK,
            protocol_level: None,
            message_id,
            codes,
            properties: None,
            bytes: None,
        }
    }
}

impl From<SubscribeMessage> for SubackMessage {
    fn from(smsg: SubscribeMessage) -> Self {
        let codes = if (smsg.qos.unwrap() as u32) < 3 {
            smsg.qos.unwrap().as_byte().to_ne_bytes().to_vec()
        } else {
            MqttQos::Failure.as_byte().to_ne_bytes().to_vec()
        };
        SubackMessage {
            msg_type: TypeKind::SUBACK,
            protocol_level: None,
            message_id: smsg.message_id,
            codes,
            properties: None,
            bytes: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnsubscribeMessage {
    pub msg_type: TypeKind,
    pub protocol_level: Option<MqttProtocolLevel>,
    pub message_id: u16,
    pub topic: String,
    pub properties: Option<Vec<PropertyItem>>,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessageType for UnsubscribeMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl UnsubscribeMessage {
    pub fn new(message_id: u16, topic: String) -> Self {
        UnsubscribeMessage {
            msg_type: TypeKind::UNSUBSCRIBE,
            protocol_level: None,
            message_id,
            topic,
            properties: None,
            bytes: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnsubackMessage {
    pub msg_type: TypeKind,
    pub protocol_level: Option<MqttProtocolLevel>,
    pub message_id: u16,
    pub code: Option<u8>,
    pub properties: Option<Vec<PropertyItem>>,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessageType for UnsubackMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl UnsubackMessage {
    pub fn new(message_id: u16, code: Option<u8>) -> Self {
        UnsubackMessage {
            msg_type: TypeKind::UNSUBACK,
            protocol_level: None,
            message_id,
            code,
            properties: None,
            bytes: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PublishMessage {
    pub msg_type: TypeKind,
    pub protocol_level: Option<MqttProtocolLevel>,
    pub message_id: u16,
    pub topic: String,
    pub dup: MqttDup,
    pub qos: MqttQos,
    pub retain: MqttRetain,
    pub msg_body: String,
    pub properties: Option<Vec<PropertyItem>>,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessageType for PublishMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl PublishMessage {
    pub fn new(qos: MqttQos, dup: MqttDup, retain: MqttRetain, topic: String, message_id: u16, message_body: String, properties: Option<Vec<PropertyItem>>) -> PublishMessage {
        PublishMessage {
            msg_type: TypeKind::PUBLISH,
            protocol_level: None,
            message_id,
            topic,
            dup,
            qos,
            retain,
            msg_body: message_body,
            properties,
            bytes: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PubackMessage {
    pub msg_type: TypeKind,
    pub protocol_level: Option<MqttProtocolLevel>,
    pub message_id: u16,
    pub code: Option<ReasonPhrases>,
    pub properties: Option<Vec<PropertyItem>>,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessageType for PubackMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl PubackMessage {
    pub fn new(message_id: u16) -> Self {
        PubackMessage {
            msg_type: TypeKind::PUBACK,
            protocol_level: None,
            message_id,
            code: Some(ReasonPhrases::Success),
            properties: None,
            bytes: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PubrecMessage {
    pub msg_type: TypeKind,
    pub protocol_level: Option<MqttProtocolLevel>,
    pub message_id: u16,
    pub code: Option<ReasonPhrases>,
    pub properties: Option<Vec<PropertyItem>>,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessageType for PubrecMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl PubrecMessage {
    pub fn new(message_id: u16) -> Self {
        PubrecMessage {
            msg_type: TypeKind::PUBREC,
            protocol_level: None,
            message_id,
            code: Some(ReasonPhrases::Success),
            properties: None,
            bytes: None,
        }
    }
}

impl From<PubackMessage> for PubrecMessage {
    fn from(puback: PubackMessage) -> Self {
        PubrecMessage::new(puback.message_id)
    }
}

#[derive(Debug, Clone)]
pub struct PubrelMessage {
    pub msg_type: TypeKind,
    pub code: Option<ReasonPhrases>,
    pub properties: Option<Vec<PropertyItem>>,
    pub protocol_level: Option<MqttProtocolLevel>,
    pub message_id: u16,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessageType for PubrelMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl PubrelMessage {
    pub fn new(message_id: u16) -> Self {
        PubrelMessage {
            msg_type: TypeKind::PUBREL,
            properties: None,
            protocol_level: None,
            code: Some(ReasonPhrases::Success),
            message_id,
            bytes: None,
        }
    }
}

impl From<PubrecMessage> for PubrelMessage {
    fn from(pubrec: PubrecMessage) -> Self {
        PubrelMessage::new(pubrec.message_id)
    }
}

#[derive(Debug, Clone)]
pub struct PubcompMessage {
    pub msg_type: TypeKind,
    pub protocol_level: Option<MqttProtocolLevel>,
    pub message_id: u16,
    pub code: Option<ReasonPhrases>,
    pub properties: Option<Vec<PropertyItem>>,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessageType for PubcompMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl PubcompMessage {
    pub fn new(message_id: u16) -> Self {
        PubcompMessage {
            msg_type: TypeKind::PUBCOMP,
            protocol_level: None,
            message_id,
            code: Some(ReasonPhrases::Success),
            properties: None,
            bytes: None,
        }
    }
}

impl From<PubrelMessage> for PubcompMessage {
    fn from(pubrel: PubrelMessage) -> Self {
        PubcompMessage::new(pubrel.message_id)
    }
}

#[derive(Debug, Clone)]
pub struct DisconnectMessage {
    pub msg_type: TypeKind,
    pub code: Option<u8>,
    pub protocol_level: Option<MqttProtocolLevel>,
    pub properties: Option<Vec<PropertyItem>>,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessageType for DisconnectMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl Default for DisconnectMessage {
    fn default() -> Self {
        DisconnectMessage {
            msg_type: TypeKind::DISCONNECT,
            code: None,
            protocol_level: None,
            properties: None,
            bytes: Some(pack_header(TypeKind::DISCONNECT, 0)),
        }
    }
}

impl From<BaseMessage> for DisconnectMessage {
    fn from(base: BaseMessage) -> Self {
        DisconnectMessage { msg_type: base.msg_type, code: None, protocol_level: None, properties: None, bytes: Some(base.bytes) }
    }
}


#[derive(Debug, Clone)]
pub struct PingreqMessage {
    pub msg_type: TypeKind,
    pub protocol_level: Option<MqttProtocolLevel>,
    pub properties: Option<Vec<PropertyItem>>,
    pub bytes: Vec<u8>,
}

impl MqttMessageType for PingreqMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl From<BaseMessage> for PingreqMessage {
    fn from(base: BaseMessage) -> Self {
        PingreqMessage { msg_type: base.msg_type, protocol_level: None, properties: None, bytes: base.bytes }
    }
}

impl Default for PingreqMessage {
    fn default() -> Self {
        PingreqMessage {
            msg_type: TypeKind::PINGREQ,
            protocol_level: None,
            properties: None,
            bytes: pack_header(TypeKind::PINGREQ, 0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PingrespMessage {
    pub msg_type: TypeKind,
    pub protocol_level: Option<MqttProtocolLevel>,
    pub properties: Option<Vec<PropertyItem>>,
    pub bytes: Vec<u8>,
}

impl MqttMessageType for PingrespMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl From<BaseMessage> for PingrespMessage {
    fn from(base: BaseMessage) -> Self {
        PingrespMessage { msg_type: base.msg_type, protocol_level: None, properties: None, bytes: base.bytes }
    }
}

impl Default for PingrespMessage {
    fn default() -> Self {
        PingrespMessage {
            msg_type: TypeKind::PINGRESP,
            protocol_level: None,
            properties: None,
            bytes: pack_header(TypeKind::PINGRESP, 0),
        }
    }
}


#[derive(Debug, Clone)]
pub struct AuthMessage {
    pub msg_type: TypeKind,
    pub code: u8,
    pub properties: Option<Vec<PropertyItem>>,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessageType for AuthMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl Default for AuthMessage {
    fn default() -> Self {
        AuthMessage {
            msg_type: TypeKind::AUTH,
            code: ReasonPhrases::Success.as_byte(),
            properties: Some(Vec::default()),
            bytes: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommonPayloadMessage {
    pub msg_type: TypeKind,
    pub message_id: u16,
    pub code: ReasonPhrases,
    pub properties: Option<Vec<PropertyItem>>,
    pub bytes: Option<Vec<u8>>,
}

impl CommonPayloadMessage {
    pub fn new(kind: TypeKind, message_id: u16) -> CommonPayloadMessage {
        CommonPayloadMessage {
            msg_type: kind,
            message_id,
            code: ReasonPhrases::Success,
            properties: Some(Vec::default()),
            bytes: None,
        }
    }
}

impl MqttMessageType for CommonPayloadMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}
