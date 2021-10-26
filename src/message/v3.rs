use crate::tools::types::TypeKind;
use crate::tools::un_pack_tool::{get_type, parse_string, parse_short_int, get_publish_header, get_connect_variable_header};
use crate::tools::protocol::{MqttProtocolLevel, MqttWillMessage, MqttCleanSession, MqttWillFlag, MqttWillTopic, MqttUsernameFlag, MqttPasswordFlag, MqttSessionPresent, MqttDup, MqttQos, MqttRetain};
use std::convert::TryFrom;
use crate::hex::reason_code::{ReasonCodeV3, ReasonCodes};
use crate::tools::pack_tool::{pack_header};
use crate::tools::config::Config;
use crate::hex::reason_code::ReasonCodes::V3;
use crate::packet::{v3_packet, v3_unpacket};
use crate::message::{MqttBytesMessage, MqttMessageType, BaseMessage, ConnectMessagePayload, PingreqMessage, PingrespMessage, WillField};

#[derive(Debug, Clone)]
pub enum MqttMessageV3 {
    Connect(ConnectMessage),
    Connack(ConnackMessage),
    Publish(PublishMessage),
    Puback(PubackMessage),
    Pubrec(PubrecMessage),
    Pubrel(PubrelMessage),
    Pubcomp(PubcompMessage),
    Subscribe(SubscribeMessage),
    Suback(SubackMessage),
    Unsubscribe(UnsubscribeMessage),
    Unsuback(UnsubackMessage),
    Pingreq(PingreqMessage),
    Pingresp(PingrespMessage),
    Disconnect(DisconnectMessage),
}

impl MqttMessageV3 {
    pub fn is_connect(&self) -> bool {
        matches!(self, MqttMessageV3::Connect(_))
    }

    pub fn is_cannack(&self) -> bool {
        matches!(self, MqttMessageV3::Connack(_))
    }

    pub fn is_publish(&self) -> bool {
        matches!(self, MqttMessageV3::Publish(_))
    }

    pub fn is_puback(&self) -> bool {
        matches!(self, MqttMessageV3::Puback(_))
    }

    pub fn is_pubrec(&self) -> bool {
        matches!(self, MqttMessageV3::Pubrec(_))
    }

    pub fn is_pubrel(&self) -> bool {
        matches!(self, MqttMessageV3::Pubrel(_))
    }

    pub fn is_pubcomp(&self) -> bool {
        matches!(self, MqttMessageV3::Pubcomp(_))
    }

    pub fn is_subscribe(&self) -> bool {
        matches!(self, MqttMessageV3::Subscribe(_))
    }

    pub fn is_suback(&self) -> bool {
        matches!(self, MqttMessageV3::Suback(_))
    }

    pub fn is_unsubscribe(&self) -> bool {
        matches!(self, MqttMessageV3::Unsubscribe(_))
    }

    pub fn is_unsuback(&self) -> bool {
        matches!(self, MqttMessageV3::Unsuback(_))
    }

    pub fn is_pingreq(&self) -> bool {
        matches!(self, MqttMessageV3::Pingreq(_))
    }

    pub fn is_pingresp(&self) -> bool {
        matches!(self, MqttMessageV3::Pingresp(_))
    }

    pub fn is_disconnect(&self) -> bool {
        matches!(self, MqttMessageV3::Disconnect(_))
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            MqttMessageV3::Connect(msg) => { msg.as_bytes() }
            MqttMessageV3::Connack(msg) => { msg.as_bytes() }
            MqttMessageV3::Pingreq(msg) => { msg.as_bytes() }
            MqttMessageV3::Pingresp(msg) => { msg.as_bytes() }
            MqttMessageV3::Disconnect(msg) => { msg.as_bytes() }
            MqttMessageV3::Subscribe(msg) => { msg.as_bytes() }
            MqttMessageV3::Suback(msg) => { msg.as_bytes() }
            MqttMessageV3::Unsubscribe(msg) => { msg.as_bytes() }
            MqttMessageV3::Unsuback(msg) => { msg.as_bytes() }
            MqttMessageV3::Puback(msg) => { msg.as_bytes() }
            MqttMessageV3::Pubrec(msg) => { msg.as_bytes() }
            MqttMessageV3::Pubrel(msg) => { msg.as_bytes() }
            MqttMessageV3::Pubcomp(msg) => { msg.as_bytes() }
            MqttMessageV3::Publish(msg) => { msg.as_bytes() }
        }
    }
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

impl MqttMessageType for ConnectMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl ConnectMessage {
    pub fn new(clean_session: MqttCleanSession, config: Config) -> ConnectMessage {
        let mut msg = ConnectMessage {
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
            bytes: None,
        };

        msg.bytes = Some(v3_packet::connect(&msg));
        msg
    }
}

impl MqttBytesMessage for ConnectMessage {
    fn as_bytes(&self) -> &[u8] {
        self.bytes.as_ref().unwrap()
    }
}

impl From<BaseMessage> for ConnectMessage {
    fn from(data: BaseMessage) -> Self {
        v3_unpacket::connect(data)
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
    pub session_present: MqttSessionPresent,
    pub return_code: u8,
    pub bytes: Vec<u8>,
}

impl MqttMessageType for ConnackMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for ConnackMessage {
    fn as_bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }
}

impl Default for ConnackMessage {
    fn default() -> Self {
        ConnackMessage {
            msg_type: TypeKind::CONNACK,
            session_present: MqttSessionPresent::Disable,
            return_code: ReasonCodeV3::ConnectionAccepted as u8,
            bytes: v3_packet::connack(MqttSessionPresent::Disable, ReasonCodeV3::ConnectionAccepted),
        }
    }
}

impl From<BaseMessage> for ConnackMessage {
    fn from(base: BaseMessage) -> Self {
        v3_unpacket::connack(base)
    }
}

impl ConnackMessage {
    pub fn new(session_present: MqttSessionPresent, return_code: ReasonCodeV3) -> ConnackMessage {
        ConnackMessage {
            msg_type: TypeKind::CONNACK,
            session_present,
            return_code: return_code as u8,
            bytes: v3_packet::connack(session_present, return_code),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SubscribeMessage {
    pub msg_type: TypeKind,
    pub message_id: u16,
    pub topic: String,
    pub qos: MqttQos,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessageType for SubscribeMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for SubscribeMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_ref().unwrap()
    }
}

impl SubscribeMessage {
    pub fn new(message_id: u16, topic: String, qos: MqttQos) -> Self {
        let mut msg = SubscribeMessage {
            msg_type: TypeKind::SUBSCRIBE,
            message_id,
            topic,
            qos,
            bytes: None,
        };
        msg.bytes = Some(v3_packet::subscribe(&msg));
        msg
    }
}

#[derive(Debug, Clone)]
pub struct SubackMessage {
    pub msg_type: TypeKind,
    pub message_id: u16,
    pub codes: Vec<u8>,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessageType for SubackMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for SubackMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_ref().unwrap()
    }
}

impl SubackMessage {
    pub fn new(message_id: u16, qos: MqttQos) -> Self {
        let codes = if (qos as u32) < 3 {
            qos.as_byte().to_ne_bytes().to_vec()
        } else {
            MqttQos::Failure.as_byte().to_ne_bytes().to_vec()
        };
        let mut msg = SubackMessage {
            msg_type: TypeKind::SUBACK,
            message_id,
            codes,
            bytes: None,
        };
        msg.bytes = Some(v3_packet::suback(&msg));
        msg
    }
}

impl From<SubscribeMessage> for SubackMessage {
    fn from(smsg: SubscribeMessage) -> Self {
        let codes = if (smsg.qos as u32) < 3 {
            smsg.qos.as_byte().to_ne_bytes().to_vec()
        } else {
            MqttQos::Failure.as_byte().to_ne_bytes().to_vec()
        };
        let mut msg = SubackMessage {
            msg_type: TypeKind::SUBACK,
            message_id: smsg.message_id,
            codes,
            bytes: None,
        };
        msg.bytes = Some(v3_packet::suback(&msg));
        msg
    }
}

impl From<BaseMessage> for SubackMessage {
    fn from(base: BaseMessage) -> Self {
        v3_unpacket::suback(base)
    }
}

#[derive(Debug, Clone)]
pub struct UnsubscribeMessage {
    pub msg_type: TypeKind,
    pub message_id: u16,
    pub topic: String,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessageType for UnsubscribeMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for UnsubscribeMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_ref().unwrap()
    }
}

impl UnsubscribeMessage {
    pub fn new(message_id: u16, topic: String) -> Self {
        let mut msg = UnsubscribeMessage {
            msg_type: TypeKind::UNSUBSCRIBE,
            message_id,
            topic,
            bytes: None,
        };
        msg.bytes = Some(v3_packet::unsubscribe(&msg));
        msg
    }
}

#[derive(Debug, Clone)]
pub struct UnsubackMessage {
    pub msg_type: TypeKind,
    pub message_id: u16,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessageType for UnsubackMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for UnsubackMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_ref().unwrap()
    }
}

impl UnsubackMessage {
    pub fn new(message_id: u16) -> Self {
        let mut msg = UnsubackMessage {
            msg_type: TypeKind::UNSUBACK,
            message_id,
            bytes: None,
        };
        msg.bytes = Some(v3_packet::not_payload(msg.message_id, TypeKind::UNSUBACK));
        msg
    }
}

impl From<BaseMessage> for UnsubackMessage {
    fn from(base: BaseMessage) -> Self {
        v3_unpacket::unsuback(base)
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
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessageType for PublishMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for PublishMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_ref().unwrap()
    }
}

impl From<BaseMessage> for PublishMessage {
    fn from(base: BaseMessage) -> Self {
        v3_unpacket::publish(base)
    }
}

impl PublishMessage {
    pub fn new(qos: MqttQos, dup: MqttDup, retain: MqttRetain, topic: String, message_id: u16, message_body: String) -> PublishMessage {
        let mut msg = PublishMessage {
            msg_type: TypeKind::PUBLISH,
            message_id,
            topic,
            dup,
            qos,
            retain,
            msg_body: message_body,
            bytes: None,
        };
        msg.bytes = Some(v3_packet::publish(&msg));
        msg
    }
}

#[derive(Debug, Clone)]
pub struct PubackMessage {
    pub msg_type: TypeKind,
    pub message_id: u16,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessageType for PubackMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for PubackMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_ref().unwrap()
    }
}

impl PubackMessage {
    pub fn new(message_id: u16) -> Self {
        let mut msg = PubackMessage {
            msg_type: TypeKind::PUBACK,
            message_id,
            bytes: None,
        };
        msg.bytes = Some(v3_packet::not_payload(msg.message_id, TypeKind::PUBACK));
        msg
    }
}

impl From<BaseMessage> for PubackMessage {
    fn from(base: BaseMessage) -> Self {
        v3_unpacket::puback(base)
    }
}

#[derive(Debug, Clone)]
pub struct PubrecMessage {
    pub msg_type: TypeKind,
    pub message_id: u16,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessageType for PubrecMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for PubrecMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_ref().unwrap()
    }
}

impl PubrecMessage {
    pub fn new(message_id: u16) -> Self {
        let mut msg = PubrecMessage {
            msg_type: TypeKind::PUBREC,
            message_id,
            bytes: None,
        };
        msg.bytes = Some(v3_packet::not_payload(msg.message_id, TypeKind::PUBREC));
        msg
    }
}

impl From<BaseMessage> for PubrecMessage {
    fn from(base: BaseMessage) -> Self {
        v3_unpacket::pubrec(base)
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
    pub message_id: u16,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessageType for PubrelMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for PubrelMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_ref().unwrap()
    }
}

impl PubrelMessage {
    pub fn new(message_id: u16) -> Self {
        let mut msg = PubrelMessage {
            msg_type: TypeKind::PUBREL,
            message_id,
            bytes: None,
        };
        msg.bytes = Some(v3_packet::not_payload(msg.message_id, TypeKind::PUBREL));
        msg
    }
}

impl From<BaseMessage> for PubrelMessage {
    fn from(base: BaseMessage) -> Self {
        v3_unpacket::pubrel(base)
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
    pub message_id: u16,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessageType for PubcompMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for PubcompMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_ref().unwrap()
    }
}

impl PubcompMessage {
    pub fn new(message_id: u16) -> Self {
        let mut msg = PubcompMessage {
            msg_type: TypeKind::PUBCOMP,
            message_id,
            bytes: None,
        };
        msg.bytes = Some(v3_packet::not_payload(msg.message_id, TypeKind::PUBCOMP));
        msg
    }
}

impl From<BaseMessage> for PubcompMessage {
    fn from(base: BaseMessage) -> Self {
        v3_unpacket::pubcomp(base)
    }
}

impl From<PubrelMessage> for PubcompMessage {
    fn from(pubrel: PubrelMessage) -> Self {
        PubcompMessage::new(pubrel.message_id)
    }
}

#[derive(Debug, Clone)]
pub struct DisconnectMessage {
    msg_type: TypeKind,
    pub bytes: Vec<u8>,
}

impl MqttMessageType for DisconnectMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for DisconnectMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_slice()
    }
}

impl Default for DisconnectMessage {
    fn default() -> Self {
        DisconnectMessage {
            msg_type: TypeKind::DISCONNECT,
            bytes: pack_header(TypeKind::DISCONNECT, 0),
        }
    }
}

impl From<BaseMessage> for DisconnectMessage {
    fn from(base: BaseMessage) -> Self {
        DisconnectMessage { msg_type: base.msg_type, bytes: base.bytes }
    }
}
