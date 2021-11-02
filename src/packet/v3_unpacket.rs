use crate::tools::protocol::{MqttSessionPresent, MqttQos, MqttDup, MqttRetain};
use crate::message::BaseMessage;
use crate::tools::un_pack_tool::{get_connect_variable_header, get_connect_payload_data, parse_short_int, parse_string, parse_byte, get_remaining_data};
use std::convert::TryFrom;
use crate::message::entity::{ConnackMessage, ConnectMessage, PubackMessage, PubcompMessage, PublishMessage, PubrecMessage, PubrelMessage, SubackMessage, SubscribeMessage, UnsubackMessage, UnsubscribeMessage};
use crate::message::v3::MqttMessageV3;

pub fn connect(base: BaseMessage) -> MqttMessageV3 {
    let (variable_header, last_data) = get_connect_variable_header(base.bytes.as_slice());
    let payload = get_connect_payload_data(
        variable_header.protocol_level.unwrap(),
        last_data,
        variable_header.will_flag.unwrap(),
        variable_header.username_flag.unwrap(),
        variable_header.password_flag.unwrap(),
    );
    MqttMessageV3::Connect(
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
            properties: None,
            bytes: Some(base.bytes),
        }
    )
}

pub fn connack(base: BaseMessage) -> MqttMessageV3 {
    let message_bytes = base.bytes.get(2..).unwrap();
    let session_present = MqttSessionPresent::try_from((message_bytes.get(0).unwrap() & 1)).unwrap();
    let return_code = *message_bytes.get(1).unwrap();
    MqttMessageV3::Connack(
        ConnackMessage {
            msg_type: base.msg_type,
            protocol_level: None,
            session_present,
            return_code: Some(return_code),
            properties: None,
            bytes: Some(base.bytes),
        }
    )
}

pub fn publish(base: BaseMessage) -> MqttMessageV3 {
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
    MqttMessageV3::Publish(
        PublishMessage {
            msg_type: base.msg_type,
            protocol_level: None,
            message_id,
            topic,
            dup: base.dup.unwrap_or(MqttDup::Disable),
            qos: base.qos.unwrap_or(MqttQos::Qos0),
            retain: base.retain.unwrap_or(MqttRetain::Disable),
            msg_body: msg_body.into_owned(),
            properties: None,
            bytes: Some(base.bytes),
        }
    )
}

pub fn subscribe(base: BaseMessage) -> Vec<MqttMessageV3> {
    let mut subs = vec![];
    let mut data_bytes = base.bytes.as_slice();
    loop {
        let remain_data = get_remaining_data(data_bytes);
        let (message_id, last_data) = parse_short_int(remain_data);
        let (topic, last_data) = parse_string(last_data).unwrap();
        let (qos, _) = parse_byte(last_data.unwrap());
        subs.push(
            MqttMessageV3::Subscribe(
                SubscribeMessage {
                    msg_type: base.msg_type,
                    protocol_level: None,
                    message_id,
                    topic,
                    qos: MqttQos::try_from(qos).ok(),
                    no_local: None,
                    retain_as_published: None,
                    retain_handling: None,
                    properties: None,
                    bytes: Some(data_bytes.get(..remain_data.len() + 2).unwrap().to_vec()),
                }
            )
        );
        if let Some(last_data) = data_bytes.get(remain_data.len() + 2..) {
            if last_data.len() > 0 { data_bytes = last_data; } else { break; }
        } else {
            break;
        }
    }
    subs
}

pub fn unsubscribe(base: BaseMessage) -> Vec<MqttMessageV3> {
    let mut subs = vec![];
    let mut data_bytes = base.bytes.as_slice();

    loop {
        let remain_data = get_remaining_data(data_bytes);
        let (message_id, last_data) = parse_short_int(remain_data);
        let (topic, last_data) = parse_string(last_data).unwrap();
        subs.push(
            MqttMessageV3::Unsubscribe(
                UnsubscribeMessage {
                    msg_type: base.msg_type,
                    protocol_level: None,
                    message_id,
                    topic,
                    properties: None,
                    bytes: Some(data_bytes.get(..remain_data.len() + 2).unwrap().to_vec()),
                }
            )
        );
        if let Some(last_data) = data_bytes.get(remain_data.len() + 2..) {
            if last_data.len() > 0 { data_bytes = last_data; } else { break; }
        } else {
            break;
        }
    }
    subs
}

pub fn unsuback(base: BaseMessage) -> MqttMessageV3 {
    let message_bytes = base.bytes.get(2..).unwrap();
    let (message_id, _) = parse_short_int(message_bytes);
    MqttMessageV3::Unsuback(
        UnsubackMessage {
            msg_type: base.msg_type,
            protocol_level: None,
            message_id,
            code: None,
            properties: None,
            bytes: Some(base.bytes)
        }
    )
}

pub fn suback(base: BaseMessage) -> MqttMessageV3 {
    let message_bytes = base.bytes.get(2..).unwrap();
    let (message_id, last_data) = parse_short_int(message_bytes);
    let codes = last_data.to_vec();
    MqttMessageV3::Suback(
        SubackMessage {
            msg_type: base.msg_type,
            protocol_level: None,
            message_id,
            codes,
            properties: None,
            bytes: Some(base.bytes),
        }
    )
}

pub fn puback(base: BaseMessage) -> MqttMessageV3 {
    let message_bytes = base.bytes.get(2..).unwrap();
    let (message_id, _) = parse_short_int(message_bytes);
    MqttMessageV3::Puback(
        PubackMessage { msg_type: base.msg_type, protocol_level: None, message_id, code: None, properties: None, bytes: Some(base.bytes) }
    )
}

pub fn pubrec(base: BaseMessage) -> MqttMessageV3 {
    let message_bytes = base.bytes.get(2..).unwrap();
    let (message_id, _) = parse_short_int(message_bytes);
    MqttMessageV3::Pubrec(
        PubrecMessage { msg_type: base.msg_type, protocol_level: None, message_id, code: None, properties: None, bytes: Some(base.bytes) }
    )
}

pub fn pubrel(base: BaseMessage) -> MqttMessageV3 {
    let message_bytes = base.bytes.get(2..).unwrap();
    let (message_id, _) = parse_short_int(message_bytes);
    MqttMessageV3::Pubrel(
        PubrelMessage { msg_type: base.msg_type, code: None, properties: None, protocol_level: None, message_id, bytes: Some(base.bytes) }
    )
}

pub fn pubcomp(base: BaseMessage) -> MqttMessageV3 {
    let message_bytes = base.bytes.get(2..).unwrap();
    let (message_id, _) = parse_short_int(message_bytes);
    MqttMessageV3::Pubcomp(
        PubcompMessage { msg_type: base.msg_type, protocol_level: None, message_id, code: None, properties: None, bytes: Some(base.bytes) }
    )
}
