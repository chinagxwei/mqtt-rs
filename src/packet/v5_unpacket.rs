use crate::message::BaseMessage;
use crate::message::v5::{ConnectMessage, ConnackMessage, PublishMessage, SubscribeMessage, SubackMessage, UnsubackMessage, SubscribeTopic};
use crate::tools::un_pack_tool::{parse_short_int, parse_byte, parse_string, get_connect_variable_header, get_connect_payload_data};
use crate::hex::un_pack_property;
use crate::protocol::{MqttQos, MqttNoLocal, MqttRetainAsPublished, MqttSessionPresent, MqttDup, MqttRetain};
use std::convert::TryFrom;

pub fn connect(mut base: BaseMessage) -> ConnectMessage {
    let message_bytes = base.bytes.get(2..).unwrap();
    let (mut variable_header, last_data) = get_connect_variable_header(message_bytes);

    // println!("v5 {:?}", last_data);

    let (properties_total_length, last_data) = parse_byte(last_data);

    // println!("properties total length: {}", properties_total_length);

    let (properties, last_data) = if properties_total_length > 0 {
        (
            Some(un_pack_property::connect(properties_total_length as u32, last_data)),
            last_data.get(properties_total_length as usize..).unwrap()
        )
    } else {
        (None, last_data)
    };

    let mut payload = get_connect_payload_data(
        last_data,
        variable_header.will_flag.unwrap(),
        variable_header.username_flag.unwrap(),
        variable_header.password_flag.unwrap(),
    );

    payload.properties = properties;

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

    let session_present = MqttSessionPresent::try_from((message_bytes.get(0).unwrap() & 1)).unwrap();

    let (return_code, last_data) = parse_byte(message_bytes);

    let (properties_total_length, last_data) = parse_byte(last_data);

    let properties = if properties_total_length > 0 {
        Some(un_pack_property::connack(properties_total_length as u32, last_data))
    } else {
        None
    };

    ConnackMessage {
        msg_type: base.msg_type,
        session_present,
        return_code,
        properties,
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

    let (properties_total_length, last_data) = parse_byte(last_data.unwrap());

    let properties = if properties_total_length > 0 {
        Some(un_pack_property::publish(properties_total_length as u32, last_data))
    } else {
        None
    };

    PublishMessage {
        msg_type: base.msg_type,
        message_id,
        topic,
        dup: base.dup.unwrap_or(MqttDup::Disable),
        qos: base.qos.unwrap_or(MqttQos::Qos0),
        retain: base.retain.unwrap_or(MqttRetain::Disable),
        msg_body: msg_body.into_owned(),
        properties,
        bytes: Some(base.bytes),
    }
}

pub fn subscribe(mut base: BaseMessage) -> SubscribeMessage {
    let message_bytes = base.bytes.get(2..).unwrap();

    let (message_id, last_data) = parse_short_int(message_bytes);

    let (properties_total_length, last_data) = parse_byte(last_data);

    let properties = if properties_total_length > 0 {
        Some(un_pack_property::subscribe(properties_total_length as u32, last_data))
    } else {
        None
    };

    let mut last_data = last_data.get(properties_total_length as usize..).unwrap();

    let mut topics = vec![];

    loop {
        let (topic, last) = parse_string(last_data).unwrap();
        let byte = *last.unwrap().get(0).unwrap();
        topics.push(SubscribeTopic {
            topic,
            qos: MqttQos::try_from((byte & 3)).unwrap(),
            no_local: MqttNoLocal::try_from((byte >> 2 & 1)).unwrap(),
            retain_as_published: MqttRetainAsPublished::try_from((byte >> 3 & 3)).unwrap(),
            retain_handling: (byte >> 4) as u32,
        });
        if last.unwrap().len() > 1 {
            last_data = last.unwrap().get(1..).unwrap();
        } else {
            break;
        }
    }

    SubscribeMessage {
        msg_type: base.msg_type,
        message_id,
        topics,
        properties,
        bytes: Some(base.bytes),
    }
}

pub fn suback(mut base: BaseMessage) -> SubackMessage {
    let message_bytes = base.bytes.get(2..).unwrap();

    let (message_id, last_data) = parse_short_int(message_bytes);

    let (properties_total_length, last_data) = parse_byte(last_data);

    let properties = if properties_total_length > 0 {
        Some(un_pack_property::suback(properties_total_length as u32, last_data))
    } else {
        None
    };

    let codes = last_data.to_vec();

    SubackMessage {
        msg_type: base.msg_type,
        message_id,
        codes,
        properties,
        bytes: Some(base.bytes),
    }
}

pub fn unsuback(mut base: BaseMessage) -> UnsubackMessage {
    let message_bytes = base.bytes.get(2..).unwrap();

    let (message_id, last_data) = parse_short_int(message_bytes);

    let (properties_total_length, last_data) = parse_byte(last_data);

    let properties = if properties_total_length > 0 {
        Some(un_pack_property::suback(properties_total_length as u32, last_data))
    } else {
        None
    };

    let codes = last_data.to_vec();

    UnsubackMessage {
        msg_type: base.msg_type,
        message_id,
        codes,
        properties,
        bytes: Some(base.bytes),
    }
}
