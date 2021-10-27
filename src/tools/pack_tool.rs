use crate::tools::types::TypeKind;
use crate::tools::protocol::{MqttCleanSession, MqttWillFlag, MqttQos, MqttDup, MqttRetain};
use crate::message::WillField;

///
/// 包装报文字符串数组
///
pub fn pack_string(str: &String) -> Vec<u8> {
    let str = str.as_bytes().to_vec();
    let mut content = pack_short_int((str.len() as u16));
    content.extend(str);
    content
}

///
/// 包装报文 byte 数组
///
pub fn pack_byte(data: u8) -> Vec<u8> {
    data.to_ne_bytes()
        .iter()
        .rev()
        .cloned()
        .collect::<Vec<u8>>()
}

///
/// 包装报文 short int 数组
///
pub fn pack_short_int(data: u16) -> Vec<u8> {
    data.to_ne_bytes()
        .iter()
        .rev()
        .cloned()
        .collect::<Vec<u8>>()
}

///
/// 包装报文 long int 数组
///
pub fn pack_long_int(data: u32) -> Vec<u8> {
    data.to_ne_bytes()
        .iter()
        .rev()
        .cloned()
        .collect::<Vec<u8>>()
}

pub fn pack_var_int(len: usize) -> Vec<u8> {
    pack_remaining_length(len)
}

pub fn pack_message_short_id(message_id: u16) -> Vec<u8> {
    pack_short_int(message_id)
}

pub fn pack_header(header_type: TypeKind, body_length: usize) -> Vec<u8> {
    let mut header = vec![header_type.as_header_byte()];

    header.extend(pack_remaining_length(body_length));

    header
}

pub fn pack_publish_header(header_type: TypeKind, body_length: usize, qos: Option<MqttQos>, dup: Option<MqttDup>, retain: Option<MqttRetain>) -> Vec<u8> {
    let mut r#type = header_type.as_header_byte();
    if dup.is_some() && dup.unwrap() == MqttDup::Enable {
        r#type |= 1 << 3
    }

    if qos.is_some() && qos.unwrap() > MqttQos::Qos0 {
        r#type |= (qos.unwrap() as u8) << 1;
    }

    if retain.is_some() && retain.unwrap() == MqttRetain::Enable {
        r#type |= 1;
    }

    let mut header = vec![r#type];

    header.extend(pack_remaining_length(body_length));

    header
}

pub fn pack_protocol_name(name_str: &String) -> Vec<u8> {
    pack_string(name_str)
}

pub fn pack_connect_flags(
    clean_session: MqttCleanSession,
    will_flag: MqttWillFlag,
    will_qos: MqttQos,
    will_retain: MqttRetain,
    username_flag: Option<&String>,
    password_flag: Option<&String>,
) -> Result<u8, String> {
    let mut connect_flags = 0_u8;
    if clean_session == MqttCleanSession::Enable {
        connect_flags |= 1 << 1;
    }
    if will_flag == MqttWillFlag::Enable {
        connect_flags |= 1 << 2;
        if will_qos > MqttQos::Qos2 {
            return Err(format!("{} not supported", will_qos as u8));
        } else {
            connect_flags |= (will_qos as u8) << 3;
        }
        if will_retain == MqttRetain::Enable {
            connect_flags |= 1 << 5;
        }
    }
    if username_flag.is_some() {
        connect_flags |= 1 << 6;
    }
    if password_flag.is_some() {
        connect_flags |= 1 << 7;
    }
    Ok(connect_flags)
}

pub fn pack_client_id(client_id: &String) -> Vec<u8> {
    pack_string(client_id)
}

pub fn pack_will_topic<T:WillField>(msg: &T) -> Option<Vec<u8>> {
    if msg.topic_str().is_some() {
        return Some(pack_string(msg.topic_str().unwrap()));
    }
    None
}

pub fn pack_will_message<T:WillField>(msg: &T) -> Option<Vec<u8>> {
    if msg.message_str().is_some() {
        return Some(pack_string(msg.message_str().unwrap()));
    }
    None
}

pub fn pack_username<T:WillField>(msg: &T) -> Option<Vec<u8>> {
    if msg.username_str().is_some() {
        return Some(pack_string(msg.username_str().unwrap()));
    }
    None
}

pub fn pack_password<T:WillField>(msg: &T) -> Option<Vec<u8>> {
    if msg.password_str().is_some() {
        return Some(pack_string(msg.password_str().unwrap()));
    }
    None
}

///
/// 包装报文剩余长度数据
///
/// from http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.pdf 第19页
///
///
fn pack_remaining_length(mut length: usize) -> Vec<u8> {
    let mut remaining = vec![];

    loop {
        let mut digit = length % 128;
        length = length / 128;
        if length > 0 {
            digit = (digit | 128);
        }
        remaining.push(digit as u8);
        if length <= 0 { break; }
    }

    remaining
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        // println!("{:?}", pack_protocol_name(&String::from("MQTT")))
        // let b = (3600_u32 as u16).to_ne_bytes();
    }
}
