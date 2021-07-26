use crate::tools::pack_tool::{pack_protocol_name, pack_connect_flags, pack_client_id, pack_will_topic, pack_will_message, pack_username, pack_password, pack_header, pack_message_short_id, pack_string, pack_publish_header};
use crate::protocol::{MqttWillFlag, MqttSessionPresent, MqttQos};
use crate::hex::reason_code::{ReasonCodes, ReasonCodeV3};
use crate::types::TypeKind;
use crate::message::v3::{ConnectMessage, PublishMessage, SubscribeMessage, SubackMessage, UnsubscribeMessage, ConnectMessagePayload};
use crate::message::BaseMessage;
use crate::tools::un_pack_tool::{get_connect_variable_header, get_connect_payload_data};

pub struct Pack;

impl Pack {
    pub fn connect(msg: &ConnectMessage) -> Vec<u8> {
        let mut body: Vec<u8> = pack_protocol_name(msg);

        body.push(msg.protocol_level as u8);

        body.push(pack_connect_flags(msg).unwrap());

        match msg.keep_alive.to_ne_bytes() {
            [a, 0_u8, 0_u8, 0_u8] => {
                body.push(0_u8);
                body.push(a);
            }
            [a, b, 0_u8, 0_u8] => {
                body.push(b);
                body.push(a);
            }
            _ => {}
        }

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

    pub fn not_payload(message_id: u32, msg_type: TypeKind) -> Vec<u8> {
        let mut body = pack_message_short_id(message_id);

        let head = pack_header(msg_type, body.len());

        [head, body].concat()
    }
}

pub struct Unpcak;

impl Unpcak {
    pub fn connect(mut base: BaseMessage) -> ConnectMessage {
        let message_bytes = base.bytes.as_slice();
        let (
            (
                mut protocol_name,
                mut keep_alive,
                mut protocol_level,
                mut clean_session,
                mut will_flag,
                mut will_qos,
                mut will_retain,
                mut password_flag,
                mut username_flag
            ),
            last_data
        ) = get_connect_variable_header(message_bytes);

        let (
            client_id,
            will_topic,
            will_message,
            user_name,
            password
        ) = get_connect_payload_data(
            last_data,
            will_flag.unwrap(),
            username_flag.unwrap(),
            password_flag.unwrap(),
        );

        ConnectMessage {
            msg_type: base.msg_type,
            protocol_name: protocol_name.take().unwrap(),
            protocol_level: protocol_level.take().unwrap(),
            clean_session: clean_session.take().unwrap(),
            will_flag: will_flag.take().unwrap(),
            will_qos: will_qos.take().unwrap(),
            will_retain: will_retain.take().unwrap(),
            keep_alive: keep_alive.take().unwrap(),
            payload: ConnectMessagePayload {
                client_id,
                will_topic,
                will_message,
                user_name,
                password,
            },
            bytes: Some(base.bytes),
        }
    }
}
