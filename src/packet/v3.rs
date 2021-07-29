use crate::tools::pack_tool::{pack_protocol_name, pack_connect_flags, pack_client_id, pack_will_topic, pack_will_message, pack_username, pack_password, pack_header, pack_message_short_id, pack_string, pack_publish_header, pack_short_int};
use crate::protocol::{MqttWillFlag, MqttSessionPresent, MqttQos, MqttDup, MqttRetain};
use crate::hex::reason_code::{ReasonCodes, ReasonCodeV3};
use crate::types::TypeKind;
use crate::message::v3::{ConnectMessage, PublishMessage, SubscribeMessage, SubackMessage, UnsubscribeMessage, ConnackMessage, UnsubackMessage, PubackMessage, PubrecMessage, PubrelMessage, PubcompMessage};
use crate::message::{BaseMessage, ConnectMessagePayload};
use crate::tools::un_pack_tool::{get_connect_variable_header, get_connect_payload_data, parse_short_int, parse_string};
use std::convert::TryFrom;

pub struct Pack;

impl Pack {
    pub fn connect(msg: &ConnectMessage) -> Vec<u8> {
        let mut body: Vec<u8> = pack_protocol_name(msg);

        body.push(msg.protocol_level as u8);

        body.push(pack_connect_flags(msg).unwrap());

        body = [body, pack_short_int(msg.keep_alive as u16)].concat();

        body = [body, pack_client_id(msg)].concat();

        if msg.will_flag == MqttWillFlag::Enable {
            if let Some(will_topic) = pack_will_topic(msg) {
                body = [body, will_topic].concat();
            }
            if let Some(will_message) = pack_will_message(msg) {
                body = [body, will_message].concat();
            }
        }

        if let Some(username) = pack_username(msg) {
            body = [body, username].concat();
        }

        if let Some(password) = pack_password(msg) {
            body = [body, password].concat();
        }
        let header = pack_header(msg.msg_type, body.len());

        [header, body].concat()
    }

    pub fn connack(session_present: MqttSessionPresent, return_code: ReasonCodeV3) -> Vec<u8> {
        let body = vec![session_present as u8, return_code as u8];
        let head = pack_header(TypeKind::CONNACK, body.len());
        [head, body].concat()
    }

    pub fn publish(msg: &PublishMessage) -> Vec<u8> {
        let mut body = pack_string(&msg.topic);

        if msg.qos > MqttQos::Qos0 {
            body = [body, pack_message_short_id(msg.message_id)].concat();
        }

        body = [body, msg.msg_body.as_bytes().to_vec()].concat();

        let head = pack_publish_header(msg.msg_type, body.len(), msg.qos, msg.dup, msg.retain);

        [head, body].concat()
    }

    pub fn subscribe(msg: &SubscribeMessage) -> Vec<u8> {
        let mut body = pack_message_short_id(msg.message_id);

        body = [body, pack_string(&msg.topic)].concat();

        body.push(msg.qos as u8);

        let head = pack_header(TypeKind::SUBSCRIBE, body.len());

        [head, body].concat()
    }

    pub fn suback(msg: &SubackMessage) -> Vec<u8> {
        let mut body = pack_message_short_id(msg.message_id);

        body.push(msg.qos as u8);

        let head = pack_header(TypeKind::SUBACK, body.len());

        [head, body].concat()
    }

    pub fn unsubscribe(msg: &UnsubscribeMessage) -> Vec<u8> {
        let mut body = pack_message_short_id(msg.message_id);

        body = [body, pack_string(&msg.topic)].concat();

        let head = pack_header(TypeKind::UNSUBSCRIBE, body.len());

        [head, body].concat()
    }

    pub fn not_payload(message_id: u16, msg_type: TypeKind) -> Vec<u8> {
        let mut body = pack_message_short_id(message_id);

        let head = pack_header(msg_type, body.len());

        [head, body].concat()
    }
}

pub struct Unpcak;

impl Unpcak {
    pub fn connect(mut base: BaseMessage) -> ConnectMessage {
        let message_bytes = base.bytes.get(2..).unwrap();
        let (mut variable_header, last_data) = get_connect_variable_header(message_bytes);

        let payload = get_connect_payload_data(
            last_data,
            variable_header.will_flag.unwrap(),
            variable_header.username_flag.unwrap(),
            variable_header.password_flag.unwrap(),
        );

        ConnectMessage {
            msg_type: base.msg_type,
            protocol_name: variable_header.protocol_name.unwrap(),
            protocol_level: variable_header.protocol_level.unwrap(),
            clean_session: variable_header.clean_session.unwrap(),
            will_flag: variable_header.will_flag.unwrap(),
            will_qos: variable_header.will_qos.unwrap(),
            will_retain: variable_header.will_retain.unwrap(),
            keep_alive: variable_header.keep_alive.unwrap(),
            payload,
            bytes: Some(base.bytes),
        }
    }

    pub fn connack(mut base: BaseMessage) -> ConnackMessage {
        let message_bytes = base.bytes.get(2..).unwrap();
        let session_present =  MqttSessionPresent::try_from((message_bytes.get(0).unwrap() & 1)).unwrap();
        let return_code = ReasonCodeV3::try_from(*message_bytes.get(1).unwrap()).unwrap();
        ConnackMessage {
            msg_type: base.msg_type,
            session_present,
            return_code,
            bytes: base.bytes,
        }
    }

    pub fn publish(mut base: BaseMessage) -> PublishMessage {
        let message_bytes = base.bytes.get(2..).unwrap();
        let (topic, last_data) = parse_string(message_bytes).unwrap();
        let (message_id, msg_body) = if base.qos.is_some() {
            let qos = base.qos.unwrap();
            if qos > MqttQos::Qos0 {
                let (message_id, last_data) = parse_short_int(last_data.unwrap());
                let msg_body = String::from_utf8_lossy(last_data);
                (message_id, msg_body)
            } else {
                let msg_body = String::from_utf8_lossy(last_data.unwrap());
                (0, msg_body)
            }
        } else {
            let msg_body = String::from_utf8_lossy(last_data.unwrap());
            (0, msg_body)
        };

        PublishMessage {
            msg_type: base.msg_type,
            message_id,
            topic,
            dup: base.dup.unwrap_or(MqttDup::Disable),
            qos: base.qos.unwrap_or(MqttQos::Qos0),
            retain: base.retain.unwrap_or(MqttRetain::Disable),
            msg_body: msg_body.into_owned(),
            bytes: Some(base.bytes),
        }
    }

    pub fn subscribe(mut base: BaseMessage) -> SubscribeMessage {
        let message_bytes = base.bytes.get(2..).unwrap();
        let (message_id, last_data) = parse_short_int(message_bytes);
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

    pub fn unsubscribe(mut base: BaseMessage) -> UnsubscribeMessage {
        let message_bytes = base.bytes.get(2..).unwrap();
        let (message_id, last_data) = parse_short_int(message_bytes);
        let (topic, _) = parse_string(last_data).unwrap();
        UnsubscribeMessage {
            msg_type: base.msg_type,
            message_id,
            topic,
            bytes: Some(base.bytes),
        }
    }

    pub fn unsuback(mut base: BaseMessage) -> UnsubackMessage {
        let message_bytes = base.bytes.get(2..).unwrap();
        let (message_id, _) = parse_short_int(message_bytes);
        UnsubackMessage { msg_type: base.msg_type, message_id, bytes: Some(base.bytes) }
    }

    pub fn puback(mut base: BaseMessage) -> PubackMessage {
        let message_bytes = base.bytes.get(2..).unwrap();
        let (message_id, _) = parse_short_int(message_bytes);
        PubackMessage { msg_type: base.msg_type, message_id, bytes: Some(base.bytes) }
    }

    pub fn pubrec(mut base: BaseMessage) -> PubrecMessage {
        let message_bytes = base.bytes.get(2..).unwrap();
        let (message_id, _) = parse_short_int(message_bytes);
        PubrecMessage { msg_type: base.msg_type, message_id, bytes: Some(base.bytes) }
    }

    pub fn pubrel(mut base: BaseMessage) -> PubrelMessage {
        let message_bytes = base.bytes.get(2..).unwrap();
        let (message_id, _) = parse_short_int(message_bytes);
        PubrelMessage { msg_type: base.msg_type, message_id, bytes: Some(base.bytes) }
    }

    pub fn pubcomp(mut base: BaseMessage) -> PubcompMessage {
        let message_bytes = base.bytes.get(2..).unwrap();
        let (message_id, _) = parse_short_int(message_bytes);
        PubcompMessage { msg_type: base.msg_type, message_id, bytes: Some(base.bytes) }
    }
}
