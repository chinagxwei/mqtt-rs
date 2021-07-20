use crate::types::TypeKind;
use crate::protocol::{MqttSessionPresent, MqttCleanSession, MqttWillFlag, MqttQos, MqttDup, MqttRetain};
use crate::hex::reason_code::ReasonCodeV3;
use crate::message::v3::ConnectMessage;

pub fn pack_string(str: &String) -> Vec<u8> {
    let str = str.as_bytes().to_vec();
    let head = vec![0_u8, str.len() as u8];
    [head, str].concat()
}

pub fn pack_message_id(message_id: u32) -> Vec<u8> {
    let mut body = vec![];
    match message_id.to_ne_bytes() {
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
    body
}

pub fn pack_header(header_type: TypeKind, body_length: usize) -> Vec<u8> {
    [vec![header_type.as_header_byte()], pack_remaining_length(body_length)].concat()
}

pub fn pack_publish_header(header_type: TypeKind, body_length: usize, qos: MqttQos, dup: MqttDup, retain: MqttRetain) -> Vec<u8> {
    let mut r#type = header_type.as_header_byte();
    if dup == MqttDup::Enable {
        r#type |= 1 << 3
    }

    if qos > MqttQos::Qos0 {
        r#type |= (qos as u8) << 1;
    }

    if retain == MqttRetain::Enable {
        r#type |= 1;
    }
    [vec![r#type], pack_remaining_length(body_length)].concat()
}

pub fn pack_protocol_name(msg: &ConnectMessage) -> Vec<u8> {
    pack_string(&msg.protocol_name)
}

pub fn pack_connect_flags(msg: &ConnectMessage) -> Result<u8, String> {
    let mut connect_flags = 0_u8;
    if msg.clean_session == MqttCleanSession::Enable {
        connect_flags |= 1 << 1;
    }
    if msg.will_flag == MqttWillFlag::Enable {
        connect_flags |= 1 << 2;
        if msg.will_qos > MqttQos::Qos2 {
            return Err(format!("{} not supported", msg.will_qos as u8));
        } else {
            connect_flags |= (msg.will_qos as u8) << 3;
        }
        if msg.will_retain == MqttRetain::Enable {
            connect_flags |= 1 << 5;
        }
    }
    if msg.payload.user_name.is_some() {
        connect_flags |= 1 << 6;
    }
    if msg.payload.password.is_some() {
        connect_flags |= 1 << 7;
    }
    Ok(connect_flags)
}

pub fn pack_client_id(msg: &ConnectMessage) -> Vec<u8> {
    pack_string(&msg.payload.client_id)
}

pub fn pack_will_topic(msg: &ConnectMessage) -> Option<Vec<u8>> {
    if msg.payload.will_topic.is_some() {
        return Some(pack_string(msg.payload.will_topic.as_ref().unwrap()));
    }
    None
}

pub fn pack_will_message(msg: &ConnectMessage) -> Option<Vec<u8>> {
    if msg.payload.will_message.is_some() {
        return Some(pack_string(msg.payload.will_message.as_ref().unwrap()));
    }
    None
}

pub fn pack_username(msg: &ConnectMessage) -> Option<Vec<u8>> {
    if msg.payload.user_name.is_some() {
        return Some(pack_string(msg.payload.user_name.as_ref().unwrap()));
    }
    None
}

pub fn pack_password(msg: &ConnectMessage) -> Option<Vec<u8>> {
    if msg.payload.password.is_some() {
        return Some(pack_string(msg.payload.password.as_ref().unwrap()));
    }
    None
}

fn pack_remaining_length(mut length: usize) -> Vec<u8> {
    let mut remaining = vec![];
    loop {
        let mut digit = length % 128;
        length = length >> 7;
        if length > 0 {
            digit = (digit | 128);
        }
        remaining.push(digit as u8);
        if length <= 0 {
            break;
        }
    }
    remaining
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mqtt::config::{Config, ConfigBuilder};

    #[test]
    fn test() {
        // println!("{:?}", pack_protocol_name(&String::from("MQTT")))
    }
}
