use crate::tools::pack_tool::{pack_protocol_name, pack_connect_flags, pack_client_id, pack_will_topic, pack_will_message, pack_username, pack_password, pack_header, pack_message_short_id, pack_string, pack_publish_header, pack_short_int};
use crate::protocol::{MqttWillFlag, MqttSessionPresent, MqttQos, MqttDup, MqttRetain};
use crate::hex::reason_code::{ReasonCodes, ReasonCodeV3};
use crate::types::TypeKind;
use crate::message::v3::{ConnectMessage, PublishMessage, SubscribeMessage, SubackMessage, UnsubscribeMessage, ConnackMessage, UnsubackMessage, PubackMessage, PubrecMessage, PubrelMessage, PubcompMessage};
use crate::message::{BaseMessage, ConnectMessagePayload};
use crate::tools::un_pack_tool::{get_connect_variable_header, get_connect_payload_data, parse_short_int, parse_string};
use std::convert::TryFrom;

pub fn connect(msg: &ConnectMessage) -> Vec<u8> {
    let mut body: Vec<u8> = pack_protocol_name(msg);

    body.push(msg.protocol_level as u8);

    body.push(
        pack_connect_flags(
            msg.clean_session,
            msg.will_flag,
            msg.will_qos,
            msg.will_retain,
            msg.payload.user_name.as_ref(),
            msg.payload.password.as_ref(),
        ).unwrap());

    body.extend(pack_short_int(msg.keep_alive as u16));

    body.extend(pack_client_id(&msg.payload.client_id));

    if msg.will_flag == MqttWillFlag::Enable {
        if let Some(will_topic) = pack_will_topic(msg) {
            body.extend(will_topic);
        }
        if let Some(will_message) = pack_will_message(msg) {
            body.extend(will_message);
        }
    }

    if let Some(username) = pack_username(msg) {
        body.extend(username);
    }

    if let Some(password) = pack_password(msg) {
        body.extend(password);
    }
    let mut package = pack_header(msg.msg_type, body.len());

    package.extend(body);

    package
}

pub fn connack(session_present: MqttSessionPresent, return_code: ReasonCodeV3) -> Vec<u8> {
    let body = vec![session_present as u8, return_code as u8];

    let mut package = pack_header(TypeKind::CONNACK, body.len());

    package.extend(body);

    package
}

pub fn publish(msg: &PublishMessage) -> Vec<u8> {
    let mut body = pack_string(&msg.topic);

    if msg.qos > MqttQos::Qos0 {
        body.extend(pack_message_short_id(msg.message_id));
    }

    body.extend(msg.msg_body.as_bytes().to_vec());

    let mut package = pack_publish_header(msg.msg_type, body.len(), msg.qos, msg.dup, msg.retain);

    package.extend(body);

    package
}

pub fn subscribe(msg: &SubscribeMessage) -> Vec<u8> {
    let mut body = pack_message_short_id(msg.message_id);

    body.extend(pack_string(&msg.topic));

    body.push(msg.qos as u8);

    let mut package = pack_header(TypeKind::SUBSCRIBE, body.len());

    package.extend(body);

    package
}

pub fn suback(msg: &SubackMessage) -> Vec<u8> {
    let mut body = pack_message_short_id(msg.message_id);

    body.extend(msg.codes.clone());

    let mut package = pack_header(TypeKind::SUBACK, body.len());

    package.extend(body);

    package
}

pub fn unsubscribe(msg: &UnsubscribeMessage) -> Vec<u8> {
    let mut body = pack_message_short_id(msg.message_id);

    body.extend(pack_string(&msg.topic));

    let mut package = pack_header(TypeKind::UNSUBSCRIBE, body.len());

    package.extend(body);

    package
}

pub fn not_payload(message_id: u16, msg_type: TypeKind) -> Vec<u8> {
    let body = pack_message_short_id(message_id);

    let mut package = pack_header(msg_type, body.len());

    package.extend(body);

    package
}
