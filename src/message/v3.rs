use crate::types::TypeKind;
use crate::tools::un_pack_tool::{get_type, parse_string, get_variable_header, parse_number, get_publish_header};
use crate::protocol::{MqttProtocolLevel, MqttWillMessage, MqttCleanSession, MqttWillFlag, MqttWillTopic, MqttUsernameFlag, MqttPasswordFlag, MqttSessionPresent, MqttDup, MqttQos, MqttRetain};
use std::convert::TryFrom;
use crate::hex::reason_code::{ReasonCodeV3, ReasonCodes};
use crate::tools::pack_tool::{pack_header};
use crate::config::Config;
use crate::hex::reason_code::ReasonCodes::V3;
use crate::packet::v3::Pack;
use crate::message::{MqttBytesMessage, MqttMessage, BaseMessage};

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
    pub keep_alive: u32,
    pub payload: ConnectMessagePayload,
    pub bytes: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct ConnectMessagePayload {
    pub client_id: String,
    pub will_topic: Option<String>,
    pub will_message: Option<String>,
    pub user_name: Option<String>,
    pub password: Option<String>,
}

impl MqttMessage for ConnectMessage {
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
            },
            bytes: None,
        };

        msg.bytes = Some(Pack::connect(&msg));
        msg
    }
}

impl MqttBytesMessage for ConnectMessage {
    fn as_bytes(&self) -> &[u8] {
        self.bytes.as_ref().unwrap().as_slice()
    }
}

impl From<BaseMessage> for ConnectMessage {
    fn from(data: BaseMessage) -> Self {
        ConnectMessageBuild::from(data).build()
    }
}

impl From<BaseMessage> for ConnectMessageBuild {
    fn from(mut base: BaseMessage) -> Self {
        let (
            protocol_name,
            keep_alive,
            protocol_level,
            clean_session,
            will_flag,
            will_qos,
            will_retain,
            password_flag,
            username_flag
        ) = get_variable_header(base.bytes.as_slice().get(2..base.bytes.len()).unwrap());

        ConnectMessageBuild {
            msg_type: Some(base.msg_type),
            protocol_name,
            protocol_level,
            clean_session,
            will_flag,
            will_qos,
            will_retain,
            username_flag,
            password_flag,
            keep_alive,
            bytes: base.bytes,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectMessageBuild {
    msg_type: Option<TypeKind>,
    protocol_name: Option<String>,
    protocol_level: Option<MqttProtocolLevel>,
    clean_session: Option<MqttCleanSession>,
    will_flag: Option<MqttWillFlag>,
    will_qos: Option<MqttQos>,
    will_retain: Option<MqttRetain>,
    username_flag: Option<MqttUsernameFlag>,
    password_flag: Option<MqttPasswordFlag>,
    keep_alive: Option<u32>,
    bytes: Vec<u8>,
}

impl ConnectMessageBuild {
    pub fn build(mut self) -> ConnectMessage {
        let (
            client_id,
            will_topic,
            will_message,
            user_name,
            password
        ) = self.get_payload_data();

        ConnectMessage {
            msg_type: self.msg_type.take().unwrap(),
            protocol_name: self.protocol_name.take().unwrap(),
            protocol_level: self.protocol_level.take().unwrap(),
            clean_session: self.clean_session.take().unwrap(),
            will_flag: self.will_flag.take().unwrap(),
            will_qos: self.will_qos.take().unwrap(),
            will_retain: self.will_retain.take().unwrap(),
            keep_alive: self.keep_alive.take().unwrap(),
            payload: ConnectMessagePayload {
                client_id,
                will_topic,
                will_message,
                user_name,
                password,
            },
            bytes: Some(self.bytes),
        }
    }

    fn get_payload_data(&self) -> (String, Option<String>, Option<String>, Option<String>, Option<String>) {
        let (client_id, last_data) = parse_string(self.bytes.as_slice().get(12..self.bytes.len()).unwrap()).unwrap();

        let (will_topic, will_message, last_data) = if MqttWillFlag::Enable == self.will_flag.unwrap() {
            let (will_topic, will_last_data) = parse_string(last_data.clone().unwrap()).unwrap();
            let (will_message, will_last_data) = parse_string(will_last_data.unwrap()).unwrap();
            (Some(will_topic), Some(will_message), will_last_data)
        } else {
            (None, None, last_data)
        };

        let (user_name, last_data) = if MqttUsernameFlag::Enable == self.username_flag.unwrap() {
            parse_string(last_data.clone().unwrap()).unwrap()
        } else {
            ("".to_string(), last_data)
        };

        let (password, _) = if MqttPasswordFlag::Enable == self.password_flag.unwrap() {
            parse_string(last_data.clone().unwrap()).unwrap()
        } else {
            ("".to_string(), last_data)
        };

        (client_id, will_topic, will_message, Some(user_name), Some(password))
    }
}

#[derive(Debug, Clone)]
pub struct ConnackMessage {
    msg_type: TypeKind,
    session_present: MqttSessionPresent,
    return_code: ReasonCodeV3,
    bytes: Vec<u8>,
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

impl Default for ConnackMessage {
    fn default() -> Self {
        ConnackMessage {
            msg_type: TypeKind::CONNACK,
            session_present: MqttSessionPresent::Disable,
            return_code: ReasonCodeV3::ConnectionAccepted,
            bytes: Pack::connack(MqttSessionPresent::Disable, ReasonCodeV3::ConnectionAccepted),
        }
    }
}

impl From<BaseMessage> for ConnackMessage {
    fn from(mut base: BaseMessage) -> Self {
        ConnackMessage {
            msg_type: base.msg_type,
            session_present: MqttSessionPresent::try_from((base.bytes.get(2).unwrap() & 1)).unwrap(),
            return_code: ReasonCodeV3::try_from(*base.bytes.get(3).unwrap()).unwrap(),
            bytes: base.bytes,
        }
    }
}

impl ConnackMessage {
    pub fn new(session_present: MqttSessionPresent, return_code: ReasonCodeV3) -> ConnackMessage {
        ConnackMessage {
            msg_type: TypeKind::CONNACK,
            session_present,
            return_code,
            bytes: Pack::connack(session_present, return_code),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SubscribeMessage {
    pub msg_type: TypeKind,
    pub message_id: u32,
    pub topic: String,
    pub qos: MqttQos,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessage for SubscribeMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for SubscribeMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_ref().unwrap().as_slice()
    }
}

impl From<BaseMessage> for SubscribeMessage {
    fn from(mut base: BaseMessage) -> Self {
        let message_id = parse_number(base.bytes.as_slice().get(2..=3).unwrap());
        let (topic, last_data) = parse_string(base.bytes.as_slice().get(4..base.bytes.len() - 1).unwrap()).unwrap();
        let qos = base.bytes.get(base.bytes.len() - 1).expect("get qos error");
        SubscribeMessage {
            msg_type: base.msg_type,
            message_id,
            topic,
            qos: MqttQos::try_from(*qos).unwrap(),
            bytes: Some(base.bytes),
        }
    }
}

impl SubscribeMessage {
    pub fn new(message_id: u32, topic: String, qos: MqttQos) -> Self {
        let mut msg = SubscribeMessage {
            msg_type: TypeKind::SUBSCRIBE,
            message_id,
            topic,
            qos,
            bytes: None,
        };
        msg.bytes = Some(Pack::subscribe(&msg));
        msg
    }
}

#[derive(Debug, Clone)]
pub struct SubackMessage {
    pub msg_type: TypeKind,
    pub message_id: u32,
    pub qos: MqttQos,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessage for SubackMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for SubackMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_ref().unwrap().as_slice()
    }
}

impl SubackMessage {
    pub fn new(message_id: u32, qos: MqttQos) -> Self {
        let mut msg = SubackMessage {
            msg_type: TypeKind::SUBACK,
            message_id,
            qos,
            bytes: None,
        };
        msg.bytes = Some(Pack::suback(&msg));
        msg
    }
}

impl From<SubscribeMessage> for SubackMessage {
    fn from(mut smsg: SubscribeMessage) -> Self {
        let mut msg = SubackMessage {
            msg_type: TypeKind::SUBACK,
            message_id: smsg.message_id,
            qos: smsg.qos,
            bytes: None,
        };
        msg.bytes = Some(Pack::suback(&msg));
        msg
    }
}

#[derive(Debug, Clone)]
pub struct UnsubscribeMessage {
    pub msg_type: TypeKind,
    pub message_id: u32,
    pub topic: String,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessage for UnsubscribeMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for UnsubscribeMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_ref().unwrap().as_slice()
    }
}

impl From<BaseMessage> for UnsubscribeMessage {
    fn from(mut base: BaseMessage) -> Self {
        let message_id = parse_number(base.bytes.as_slice().get(2..=3).unwrap());
        let (topic, _) = parse_string(base.bytes.as_slice().get(4..base.bytes.len()).unwrap()).unwrap();
        UnsubscribeMessage {
            msg_type: base.msg_type,
            message_id,
            topic,
            bytes: Some(base.bytes),
        }
    }
}

impl UnsubscribeMessage {
    pub fn new(message_id: u32, topic: String) -> Self {
        let mut msg = UnsubscribeMessage {
            msg_type: TypeKind::UNSUBSCRIBE,
            message_id,
            topic,
            bytes: None,
        };
        msg.bytes = Some(Pack::unsubscribe(&msg));
        msg
    }
}

#[derive(Debug, Clone)]
pub struct UnsubackMessage {
    pub msg_type: TypeKind,
    pub message_id: u32,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessage for UnsubackMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for UnsubackMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_ref().unwrap().as_slice()
    }
}

impl UnsubackMessage {
    pub fn new(message_id: u32) -> Self {
        let mut msg = UnsubackMessage {
            msg_type: TypeKind::UNSUBACK,
            message_id,
            bytes: None,
        };
        msg.bytes = Some(Pack::not_payload(msg.message_id, TypeKind::UNSUBACK));
        msg
    }
}

impl From<BaseMessage> for UnsubackMessage {
    fn from(mut base: BaseMessage) -> Self {
        let message_id = parse_number(base.bytes.as_slice().get(2..=3).unwrap());
        UnsubackMessage { msg_type: base.msg_type, message_id, bytes: Some(base.bytes) }
    }
}

#[derive(Debug, Clone)]
pub struct PublishMessage {
    pub msg_type: TypeKind,
    pub message_id: u32,
    pub topic: String,
    pub dup: MqttDup,
    pub qos: MqttQos,
    pub retain: MqttRetain,
    pub msg_body: String,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessage for PublishMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for PublishMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_ref().unwrap().as_slice()
    }
}

impl From<BaseMessage> for PublishMessage {
    fn from(mut base: BaseMessage) -> Self {
        let (retain, qos, dup) = get_publish_header(*base.bytes.get(0).unwrap());
        let (topic, last_data) = parse_string(base.bytes.as_slice().get(2..base.bytes.len()).unwrap()).unwrap();
        let message_id = parse_number(last_data.unwrap());
        let msg_body = String::from_utf8_lossy(last_data.unwrap().get(2..last_data.unwrap().len()).unwrap());
        PublishMessage {
            msg_type: base.msg_type,
            message_id,
            topic,
            dup: dup.unwrap(),
            qos: qos.unwrap(),
            retain: retain.unwrap(),
            msg_body: msg_body.into_owned(),
            bytes: Some(base.bytes),
        }
    }
}

impl PublishMessage {
    pub fn new(qos: MqttQos, dup: MqttDup, retain: MqttRetain, topic: String, message_id: u32, message_body: String) -> PublishMessage {
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
        msg.bytes = Some(Pack::publish(&msg));
        msg
    }
}

#[derive(Debug, Clone)]
pub struct PubackMessage {
    msg_type: TypeKind,
    pub message_id: u32,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessage for PubackMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for PubackMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_ref().unwrap().as_slice()
    }
}

impl PubackMessage {
    pub fn new(message_id: u32) -> Self {
        let mut msg = PubackMessage {
            msg_type: TypeKind::PUBACK,
            message_id,
            bytes: None,
        };
        msg.bytes = Some(Pack::not_payload(msg.message_id, TypeKind::PUBACK));
        msg
    }
}

impl From<BaseMessage> for PubackMessage {
    fn from(mut base: BaseMessage) -> Self {
        let message_id = parse_number(base.bytes.get(2..base.bytes.len()).unwrap());
        PubackMessage { msg_type: base.msg_type, message_id, bytes: Some(base.bytes) }
    }
}

#[derive(Debug, Clone)]
pub struct PubrecMessage {
    msg_type: TypeKind,
    pub message_id: u32,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessage for PubrecMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for PubrecMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_ref().unwrap().as_slice()
    }
}

impl PubrecMessage {
    pub fn new(message_id: u32) -> Self {
        let mut msg = PubrecMessage {
            msg_type: TypeKind::PUBREC,
            message_id,
            bytes: None,
        };
        msg.bytes = Some(Pack::not_payload(msg.message_id, TypeKind::PUBREC));
        msg
    }
}

impl From<BaseMessage> for PubrecMessage {
    fn from(mut base: BaseMessage) -> Self {
        let message_id = parse_number(base.bytes.get(2..base.bytes.len()).unwrap());
        PubrecMessage { msg_type: base.msg_type, message_id, bytes: Some(base.bytes) }
    }
}

#[derive(Debug, Clone)]
pub struct PubrelMessage {
    msg_type: TypeKind,
    pub message_id: u32,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessage for PubrelMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for PubrelMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_ref().unwrap().as_slice()
    }
}

impl PubrelMessage {
    pub fn new(message_id: u32) -> Self {
        let mut msg = PubrelMessage {
            msg_type: TypeKind::PUBREL,
            message_id,
            bytes: None,
        };
        msg.bytes = Some(Pack::not_payload(msg.message_id, TypeKind::PUBREL));
        msg
    }
}

impl From<BaseMessage> for PubrelMessage {
    fn from(mut base: BaseMessage) -> Self {
        let message_id = parse_number(base.bytes.get(2..base.bytes.len()).unwrap());
        PubrelMessage { msg_type: base.msg_type, message_id, bytes: Some(base.bytes) }
    }
}

#[derive(Debug, Clone)]
pub struct PubcompMessage {
    msg_type: TypeKind,
    pub message_id: u32,
    pub bytes: Option<Vec<u8>>,
}

impl MqttMessage for PubcompMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl MqttBytesMessage for PubcompMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_ref().unwrap().as_slice()
    }
}

impl PubcompMessage {
    pub fn new(message_id: u32) -> Self {
        let mut msg = PubcompMessage {
            msg_type: TypeKind::PUBCOMP,
            message_id,
            bytes: None,
        };
        msg.bytes = Some(Pack::not_payload(msg.message_id, TypeKind::PUBCOMP));
        msg
    }
}

impl From<BaseMessage> for PubcompMessage {
    fn from(mut base: BaseMessage) -> Self {
        let message_id = parse_number(base.bytes.get(2..base.bytes.len()).unwrap());
        PubcompMessage { msg_type: base.msg_type, message_id, bytes: Some(base.bytes) }
    }
}

#[derive(Debug, Clone)]
pub struct DisconnectMessage {
    msg_type: TypeKind,
    bytes: Vec<u8>,
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

impl Default for DisconnectMessage {
    fn default() -> Self {
        DisconnectMessage {
            msg_type: TypeKind::DISCONNECT,
            bytes: pack_header(TypeKind::DISCONNECT, 0),
        }
    }
}

impl From<BaseMessage> for DisconnectMessage {
    fn from(mut base: BaseMessage) -> Self {
        DisconnectMessage { msg_type: base.msg_type, bytes: base.bytes }
    }
}

#[derive(Debug, Clone)]
pub struct PingreqMessage {
    msg_type: TypeKind,
    bytes: Vec<u8>,
}

impl MqttMessage for PingreqMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl From<BaseMessage> for PingreqMessage {
    fn from(mut base: BaseMessage) -> Self {
        PingreqMessage { msg_type: base.msg_type, bytes: base.bytes }
    }
}

impl MqttBytesMessage for PingreqMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_slice()
    }
}

impl Default for PingreqMessage {
    fn default() -> Self {
        PingreqMessage {
            msg_type: TypeKind::PINGREQ,
            bytes: pack_header(TypeKind::PINGREQ, 0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PingrespMessage {
    msg_type: TypeKind,
    bytes: Vec<u8>,
}

impl MqttMessage for PingrespMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl From<BaseMessage> for PingrespMessage {
    fn from(mut base: BaseMessage) -> Self {
        PingrespMessage { msg_type: base.msg_type, bytes: base.bytes }
    }
}

impl MqttBytesMessage for PingrespMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_slice()
    }
}

impl Default for PingrespMessage {
    fn default() -> Self {
        PingrespMessage {
            msg_type: TypeKind::PINGRESP,
            bytes: pack_header(TypeKind::PINGRESP, 0),
        }
    }
}
