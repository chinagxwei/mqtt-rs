use crate::types::TypeKind;
use crate::tools::un_pack_tool::{get_type, parse_string, parse_number, get_publish_header, get_connect_variable_header};
use crate::protocol::{MqttProtocolLevel, MqttWillMessage, MqttCleanSession, MqttWillFlag, MqttWillTopic, MqttUsernameFlag, MqttPasswordFlag, MqttSessionPresent, MqttDup, MqttQos, MqttRetain};
use std::convert::TryFrom;
use crate::hex::reason_code::{ReasonCodeV3, ReasonCodes};
use crate::tools::pack_tool::{pack_header};
use crate::config::Config;
use crate::hex::reason_code::ReasonCodes::V3;
use crate::packet::v3::{Pack, Unpcak};
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
        Unpcak::connect(data)
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
        let (message_id, last_data) = parse_number(base.bytes.as_slice());
        let (topic, last_data) = parse_string(last_data).unwrap();
        let qos = if last_data.unwrap().len() == 1 { last_data.unwrap()[0] } else { 0 };
        SubscribeMessage {
            msg_type: base.msg_type,
            message_id,
            topic,
            qos: MqttQos::try_from(qos).unwrap(),
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
        let (message_id, last_data) = parse_number(base.bytes.as_slice());
        let (topic, _) = parse_string(last_data).unwrap();
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
        let (message_id, _) = parse_number(base.bytes.as_slice());
        // let message_id = parse_number(base.bytes.as_slice().get(2..=3).unwrap());
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
        // let (retain, qos, dup) = get_publish_header(*base.bytes.get(0).unwrap());
        let (topic, last_data) = parse_string(base.bytes.as_slice()).unwrap();
        // let message_id = parse_number(last_data.unwrap());
        let (message_id, last_data) = parse_number(last_data.unwrap());
        let msg_body = String::from_utf8_lossy(last_data);
        PublishMessage {
            msg_type: base.msg_type,
            message_id,
            topic,
            dup: base.dup.unwrap(),
            qos: base.qos.unwrap(),
            retain: base.retain.unwrap(),
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
        let (message_id, _) = parse_number(base.bytes.as_slice());
        // let message_id = parse_number(base.bytes.get(2..base.bytes.len()).unwrap());
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
        // let message_id = parse_number(base.bytes.get(2..base.bytes.len()).unwrap());
        let (message_id, _) = parse_number(base.bytes.as_slice());
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
        // let message_id = parse_number(base.bytes.get(2..base.bytes.len()).unwrap());
        let (message_id, _) = parse_number(base.bytes.as_slice());
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
        // let message_id = parse_number(base.bytes.get(2..base.bytes.len()).unwrap());
        let (message_id, _) = parse_number(base.bytes.as_slice());
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
