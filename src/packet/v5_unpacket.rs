use crate::message::BaseMessage;
use crate::tools::un_pack_tool::{parse_short_int, parse_byte, parse_string, get_connect_variable_header, get_connect_payload_data, get_remaining_data};
use crate::hex::un_pack_property;
use crate::tools::protocol::{MqttQos, MqttNoLocal, MqttRetainAsPublished, MqttSessionPresent, MqttDup, MqttRetain, MqttProtocolLevel};
use std::convert::TryFrom;
use crate::hex::reason_code::ReasonPhrases;
use crate::message::entity::{AuthMessage, CommonPayloadMessage, ConnackMessage, ConnectMessage, DisconnectMessage, PublishMessage, SubackMessage, SubscribeMessage, UnsubackMessage, UnsubscribeMessage};
use crate::message::v5::MqttMessageV5;

pub fn connect(base: BaseMessage) -> MqttMessageV5 {

    let (variable_header, last_data) = get_connect_variable_header(base.bytes.as_slice());

    let (properties_total_length, last_data) = parse_byte(last_data);

    let (properties, last_data) = if properties_total_length > 0 {
        (
            Some(un_pack_property::connect(properties_total_length as u32, last_data)),
            last_data.get(properties_total_length as usize..).unwrap()
        )
    } else {
        (Some(Vec::default()), last_data)
    };

    let payload = get_connect_payload_data(
        variable_header.protocol_level.unwrap(),
        last_data,
        variable_header.will_flag.unwrap(),
        variable_header.username_flag.unwrap(),
        variable_header.password_flag.unwrap(),
    );
    MqttMessageV5::Connect(
        ConnectMessage {
            msg_type: base.msg_type,
            protocol_name: variable_header.protocol_name.unwrap(),
            protocol_level: variable_header.protocol_level.unwrap(),
            clean_session: variable_header.clean_session.unwrap(),
            will_flag: variable_header.will_flag.unwrap(),
            will_qos: variable_header.will_qos.unwrap(),
            will_retain: variable_header.will_retain.unwrap(),
            keep_alive: variable_header.keep_alive.unwrap(),
            properties,
            payload,
            bytes: Some(base.bytes),
        }
    )
}

pub fn connack(base: BaseMessage) -> MqttMessageV5 {
    let message_bytes = base.bytes.get(2..).unwrap();

    let session_present = MqttSessionPresent::try_from((message_bytes.get(0).unwrap() & 1)).unwrap();

    let (return_code, last_data) = parse_byte(message_bytes);

    let (properties_total_length, last_data) = parse_byte(last_data);

    let properties = if properties_total_length > 0 {
        Some(un_pack_property::connack(properties_total_length as u32, last_data))
    } else {
        Some(Vec::default())
    };
    MqttMessageV5::Connack(
        ConnackMessage {
            msg_type: base.msg_type,
            protocol_level: None,
            session_present,
            return_code: Some(return_code),
            properties,
            bytes: Some(base.bytes),
        }
    )
}

pub fn publish(base: BaseMessage) -> MqttMessageV5 {
    let message_bytes = base.bytes.get(2..).unwrap();

    let (topic, last_data) = parse_string(message_bytes).unwrap();

    let (message_id, last_data) = if base.qos.is_some() {
        let qos = base.qos.unwrap();
        if qos > MqttQos::Qos0 {
            let (message_id, last_data) = parse_short_int(last_data.unwrap());
            (message_id, last_data)
        } else {
            (0, last_data.unwrap())
        }
    } else {
        (0, last_data.unwrap())
    };

    let (properties_total_length, last_data) = parse_byte(last_data);

    let (properties, msg_body) = if properties_total_length > 0 {
        let msg_body = String::from_utf8_lossy(last_data.get(properties_total_length as usize..).unwrap());
        (
            Some(un_pack_property::publish(properties_total_length as u32, last_data)),
            msg_body
        )
    } else {
        let msg_body = String::from_utf8_lossy(last_data);
        (
            Some(Vec::default()),
            msg_body
        )
    };
    MqttMessageV5::Publish(
        PublishMessage {
            msg_type: base.msg_type,
            protocol_level: None,
            message_id,
            topic,
            dup: base.dup.unwrap_or(MqttDup::Disable),
            qos: base.qos.unwrap_or(MqttQos::Qos0),
            retain: base.retain.unwrap_or(MqttRetain::Disable),
            msg_body: msg_body.into_owned(),
            properties,
            bytes: Some(base.bytes),
        }
    )
}

pub fn subscribe(base: BaseMessage) -> Vec<MqttMessageV5> {
    println!("{:?}", base.bytes);
    let mut subs = vec![];
    let mut data_bytes = base.bytes.as_slice();

    loop {
        let remain_data = get_remaining_data(data_bytes);
        let (message_id, last_data) = parse_short_int(remain_data);
        let (properties_total_length, last_data) = parse_byte(last_data);
        let (properties, last_data) = if properties_total_length > 0 {
            (
                Some(un_pack_property::subscribe(properties_total_length as u32, last_data)),
                last_data.get(properties_total_length as usize..).unwrap()
            )
        } else {
            (
                Some(Vec::default()),
                last_data
            )
        };

        let (topic, last_data) = parse_string(last_data).unwrap();
        let (byte_data, _) = parse_byte(last_data.unwrap());
        let qos = byte_data & 3;
        let no_local = byte_data >> 2 & 1;
        let retain_as_published = byte_data >> 3 & 1;
        let retain_handling = byte_data >> 4;
        subs.push(
            MqttMessageV5::Subscribe(
                SubscribeMessage {
                    msg_type: base.msg_type,
                    protocol_level: None,
                    message_id,
                    topic,
                    qos: MqttQos::try_from(qos).ok(),
                    no_local: MqttNoLocal::try_from(no_local).ok(),
                    retain_as_published: MqttRetainAsPublished::try_from(retain_as_published).ok(),
                    retain_handling: Option::from(retain_handling),
                    properties,
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

    println!("{:?}", subs);
    subs
}

pub fn unsubscribe(base: BaseMessage) -> Vec<MqttMessageV5> {
    let mut subs = vec![];
    let mut data_bytes = base.bytes.as_slice();

    loop {
        let remain_data = get_remaining_data(data_bytes);
        let (message_id, last_data) = parse_short_int(remain_data);

        let (properties_total_length, last_data) = parse_byte(last_data);
        let (properties, last_data) = if properties_total_length > 0 {
            (
                Some(un_pack_property::unsubscribe(properties_total_length as u32, last_data)),
                last_data.get(properties_total_length as usize..).unwrap()
            )
        } else {
            (
                Some(Vec::default()),
                last_data
            )
        };

        let (topic, _) = parse_string(last_data).unwrap();

        subs.push(
            MqttMessageV5::Unsubscribe(
                UnsubscribeMessage {
                    msg_type: base.msg_type,
                    protocol_level: None,
                    message_id,
                    topic,
                    properties,
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

    println!("{:?}", subs);
    subs
}

pub fn suback(base: BaseMessage) -> MqttMessageV5 {
    let message_bytes = base.bytes.get(2..).unwrap();

    let (message_id, last_data) = parse_short_int(message_bytes);

    let (properties_total_length, last_data) = parse_byte(last_data);

    let properties = if properties_total_length > 0 {
        Some(un_pack_property::suback(properties_total_length as u32, last_data))
    } else {
        Some(Vec::default())
    };

    let codes = last_data.to_vec();

    MqttMessageV5::Suback(
        SubackMessage {
            msg_type: base.msg_type,
            protocol_level: None,
            message_id,
            codes,
            properties,
            bytes: Some(base.bytes),
        }
    )
}

pub fn unsuback(base: BaseMessage) -> MqttMessageV5 {
    let message_bytes = base.bytes.get(2..).unwrap();

    let (message_id, last_data) = parse_short_int(message_bytes);

    let (properties_total_length, last_data) = parse_byte(last_data);

    let properties = if properties_total_length > 0 {
        Some(un_pack_property::suback(properties_total_length as u32, last_data))
    } else {
        Some(Vec::default())
    };

    let code = *last_data.get(base.bytes.len() - 1).unwrap();
    MqttMessageV5::Unsuback(
        UnsubackMessage {
            msg_type: base.msg_type,
            protocol_level: None,
            message_id,
            code: Some(code),
            properties,
            bytes: Some(base.bytes),
        }
    )
}

pub fn disconnect(base: BaseMessage) -> MqttMessageV5 {
    let message_bytes = base.bytes.get(2..).unwrap();

    let (code, mut last_data) = if message_bytes.len() > 0 {
        parse_byte(message_bytes)
    } else {
        (ReasonPhrases::Success as u8, message_bytes)
    };

    let mut properties_total_length = 0;

    if last_data.len() > 0 {
        let (length, last) = parse_byte(last_data);
        properties_total_length = length;
        last_data = last
    }

    let properties = if properties_total_length > 0 {
        Some(un_pack_property::suback(properties_total_length as u32, last_data))
    } else {
        Some(Vec::default())
    };
    MqttMessageV5::Disconnect(
        DisconnectMessage {
            msg_type: base.msg_type,
            code: Some(code),
            protocol_level: None,
            properties,
            bytes: Some(base.bytes),
        }
    )
}

pub fn auth(base: BaseMessage) -> MqttMessageV5 {
    let message_bytes = base.bytes.get(2..).unwrap();

    let (code, mut last_data) = if message_bytes.len() > 0 {
        parse_byte(message_bytes)
    } else {
        (ReasonPhrases::Success as u8, message_bytes)
    };

    let mut properties_total_length = 0;

    if last_data.len() > 0 {
        let (length, last) = parse_byte(last_data);
        properties_total_length = length;
        last_data = last
    }

    let properties = if properties_total_length > 0 {
        Some(un_pack_property::auth(properties_total_length as u32, last_data))
    } else {
        Some(Vec::default())
    };
    MqttMessageV5::Auth(
        AuthMessage {
            msg_type: base.msg_type,
            protocol_level: Some(MqttProtocolLevel::Level5),
            code,
            properties,
            bytes: Some(base.bytes),
        }
    )
}

pub fn puback(base: BaseMessage) -> MqttMessageV5 {
    MqttMessageV5::Puback(
        get_reason_code(base).into()
    )
}

pub fn pubrec(base: BaseMessage) -> MqttMessageV5 {
    MqttMessageV5::Pubrec(
        get_reason_code(base).into()
    )
}

pub fn pubrel(base: BaseMessage) -> MqttMessageV5 {
    MqttMessageV5::Pubrel(
        get_reason_code(base).into()
    )
}

pub fn pubcomp(base: BaseMessage) -> MqttMessageV5 {
    MqttMessageV5::Pubcomp(
        get_reason_code(base).into()
    )
}

pub fn get_reason_code(base: BaseMessage) -> CommonPayloadMessage {
    let message_bytes = base.bytes.get(2..).unwrap();

    let (message_id, last_data) = parse_short_int(message_bytes);

    let (code, mut last_data) = if message_bytes.len() > 0 {
        parse_byte(last_data)
    } else {
        (ReasonPhrases::Success as u8, last_data)
    };

    let mut properties_total_length = 0;

    if last_data.len() > 0 {
        let (length, last) = parse_byte(last_data);
        properties_total_length = length;
        last_data = last
    }

    let properties = if properties_total_length > 0 {
        Some(un_pack_property::pub_and_sub(properties_total_length as u32, last_data))
    } else {
        Some(Vec::default())
    };

    CommonPayloadMessage {
        msg_type: base.msg_type,
        message_id,
        code: crate::hex::reason_code::ReasonPhrases::try_from(code).unwrap(),
        properties,
        bytes: Some(base.bytes),
    }
}
