use crate::tools::pack_tool::{pack_protocol_name, pack_connect_flags, pack_client_id, pack_will_topic, pack_will_message, pack_username, pack_password, pack_header, pack_message_short_id, pack_string, pack_publish_header, pack_short_int};
use crate::tools::protocol::{MqttWillFlag, MqttSessionPresent, MqttQos, MqttDup, MqttRetain};
use crate::hex::reason_code::{ReasonCodes, ReasonCodeV3};
use crate::tools::types::TypeKind;
use crate::message::v3::{ConnectMessage, PublishMessage, SubscribeMessage, SubackMessage, UnsubscribeMessage, ConnackMessage, UnsubackMessage, PubackMessage, PubrecMessage, PubrelMessage, PubcompMessage};
use crate::message::{BaseMessage, ConnectMessagePayload};
use crate::tools::un_pack_tool::{get_connect_variable_header, get_connect_payload_data, parse_short_int, parse_string, parse_byte, get_remaining_data};
use std::convert::TryFrom;

pub fn connect(mut base: BaseMessage) -> ConnectMessage {
    let message_bytes = base.bytes.get(2..).unwrap();
    let (variable_header, last_data) = get_connect_variable_header(message_bytes);
    let payload = get_connect_payload_data(
        variable_header.protocol_level.unwrap(),
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

pub fn connack(base: BaseMessage) -> ConnackMessage {
    let message_bytes = base.bytes.get(2..).unwrap();
    let session_present = MqttSessionPresent::try_from((message_bytes.get(0).unwrap() & 1)).unwrap();
    let return_code = *message_bytes.get(1).unwrap();
    ConnackMessage {
        msg_type: base.msg_type,
        protocol_level: None,
        session_present,
        return_code,
        bytes: Some(base.bytes),
    }
}

pub fn publish(base: BaseMessage) -> PublishMessage {
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
        protocol_level: None,
        message_id,
        topic,
        dup: base.dup.unwrap_or(MqttDup::Disable),
        qos: base.qos.unwrap_or(MqttQos::Qos0),
        retain: base.retain.unwrap_or(MqttRetain::Disable),
        msg_body: msg_body.into_owned(),
        bytes: Some(base.bytes),
    }
}

pub fn subscribe(base: BaseMessage) -> Vec<SubscribeMessage> {
    let mut subs = vec![];
    let mut data_bytes = base.bytes.as_slice();
    loop {
        let remain_data = get_remaining_data(data_bytes);
        let (message_id, last_data) = parse_short_int(remain_data);
        let (topic, last_data) = parse_string(last_data).unwrap();
        let (qos, _) = parse_byte(last_data.unwrap());
        subs.push(
            SubscribeMessage {
                msg_type: base.msg_type,
                protocol_level: None,
                message_id,
                topic,
                qos: MqttQos::try_from(qos).unwrap(),
                bytes: Some(data_bytes.get(..remain_data.len() + 2).unwrap().to_vec()),
            }
        );
        if let Some(last_data) = data_bytes.get(remain_data.len() + 2..) {
            if last_data.len() > 0 { data_bytes = last_data; } else { break; }
        } else {
            break;
        }
    }
    subs
}

pub fn unsubscribe(base: BaseMessage) -> Vec<UnsubscribeMessage> {
    let mut subs = vec![];
    let mut data_bytes = base.bytes.as_slice();

    loop {
        let remain_data = get_remaining_data(data_bytes);
        let (message_id, last_data) = parse_short_int(remain_data);
        let (topic, last_data) = parse_string(last_data).unwrap();
        subs.push(
            UnsubscribeMessage {
                msg_type: base.msg_type,
                protocol_level: None,
                message_id,
                topic,
                bytes: Some(data_bytes.get(..remain_data.len() + 2).unwrap().to_vec()),
            }
        );
        if let Some(last_data) = data_bytes.get(remain_data.len() + 2..) {
            if last_data.len() > 0 { data_bytes = last_data; } else { break; }
        } else {
            break;
        }
    }
    subs
}

pub fn unsuback(base: BaseMessage) -> UnsubackMessage {
    let message_bytes = base.bytes.get(2..).unwrap();
    let (message_id, _) = parse_short_int(message_bytes);
    UnsubackMessage { msg_type: base.msg_type, protocol_level: None, message_id, bytes: Some(base.bytes) }
}

pub fn suback(base: BaseMessage) -> SubackMessage {
    let message_bytes = base.bytes.get(2..).unwrap();
    let (message_id, last_data) = parse_short_int(message_bytes);
    let codes = last_data.to_vec();
    SubackMessage {
        msg_type: base.msg_type,
        protocol_level: None,
        message_id,
        codes,
        bytes: Some(base.bytes),
    }
}

pub fn puback(base: BaseMessage) -> PubackMessage {
    let message_bytes = base.bytes.get(2..).unwrap();
    let (message_id, _) = parse_short_int(message_bytes);
    PubackMessage { msg_type: base.msg_type, protocol_level: None, message_id, bytes: Some(base.bytes) }
}

pub fn pubrec(base: BaseMessage) -> PubrecMessage {
    let message_bytes = base.bytes.get(2..).unwrap();
    let (message_id, _) = parse_short_int(message_bytes);
    PubrecMessage { msg_type: base.msg_type, protocol_level: None, message_id, bytes: Some(base.bytes) }
}

pub fn pubrel(base: BaseMessage) -> PubrelMessage {
    let message_bytes = base.bytes.get(2..).unwrap();
    let (message_id, _) = parse_short_int(message_bytes);
    PubrelMessage { msg_type: base.msg_type, protocol_level: None, message_id, bytes: Some(base.bytes) }
}

pub fn pubcomp(base: BaseMessage) -> PubcompMessage {
    let message_bytes = base.bytes.get(2..).unwrap();
    let (message_id, _) = parse_short_int(message_bytes);
    PubcompMessage { msg_type: base.msg_type, protocol_level: None, message_id, bytes: Some(base.bytes) }
}
