use crate::tools::types::TypeKind;
use std::convert::{TryFrom, TryInto};
use crate::tools::protocol::{MqttProtocolLevel, MqttCleanSession, MqttWillFlag, MqttUsernameFlag, MqttPasswordFlag, MqttRetain, MqttQos, MqttDup};
use crate::message::v3::VariableHeader;
use crate::message::ConnectMessagePayload;
use crate::hex::un_pack_property;

///
/// 获取报文种类
///
pub fn get_type(data: &[u8]) -> (Option<TypeKind>, Option<MqttRetain>, Option<MqttQos>, Option<MqttDup>, &[u8]) {
    let kind = TypeKind::try_from((data[0] >> 4)).ok();
    if kind.unwrap() == TypeKind::PUBLISH {
        let (retain, qos, dup) = get_publish_header(data[0]);
        return (kind, retain, qos, dup, get_remaining_data(data));
    }
    (kind, None, None, None, get_remaining_data(data))
}

///
/// 获取协议名称和协议版本
///
pub fn get_protocol_name_and_version(data: &[u8]) -> (Option<String>, Option<MqttProtocolLevel>) {
    let slice = get_remaining_data(data);
    let protocol_name = Option::from(String::from_utf8_lossy(slice).into_owned());
    let mqtt_version = MqttProtocolLevel::try_from(data[6]).ok();
    (protocol_name, mqtt_version)
}

///
/// 获取发布消息头
///
pub fn get_publish_header(data: u8) -> (Option<MqttRetain>, Option<MqttQos>, Option<MqttDup>) {
    let retain = data & 1;
    let qos = (data >> 1) & 3;
    let dup = (data >> 3) & 1;
    (
        MqttRetain::try_from(retain).ok(),
        MqttQos::try_from(qos).ok(),
        MqttDup::try_from(dup).ok()
    )
}

///
/// 获取 初始连接的 负载数据
///
pub fn get_connect_payload_data(protocol_level: MqttProtocolLevel, data: &[u8], will_flag: MqttWillFlag, username_flag: MqttUsernameFlag, password_flag: MqttPasswordFlag) -> ConnectMessagePayload {
    let (client_id, last_data) = parse_string(data).unwrap();

    let (properties, will_topic, will_message, last_data) = if MqttWillFlag::Enable == will_flag {
        let (properties, last_data) = if protocol_level == MqttProtocolLevel::Level5 {
            let (properties_total_length, last_data) = parse_byte(last_data.unwrap());

            if properties_total_length > 0 {
                (Some(un_pack_property::will_properties(properties_total_length as u32, last_data)), last_data.get(properties_total_length as usize..))
            } else {
                (None, Some(last_data))
            }
        } else {
            (None, last_data)
        };

        let (will_topic, will_last_data) = parse_string(last_data.unwrap()).unwrap();
        let (will_message, will_last_data) = parse_string(will_last_data.unwrap()).unwrap();
        (properties, Some(will_topic), Some(will_message), will_last_data)
    } else {
        (None, None, None, last_data)
    };

    let (user_name, last_data) = if MqttUsernameFlag::Enable == username_flag {
        parse_string(last_data.unwrap()).unwrap()
    } else {
        ("".to_string(), last_data)
    };

    let (password, _) = if MqttPasswordFlag::Enable == password_flag {
        parse_string(last_data.unwrap()).unwrap()
    } else {
        ("".to_string(), last_data)
    };
    println!("client ID: {}", client_id);
    ConnectMessagePayload {
        client_id,
        will_topic,
        will_message,
        user_name: Some(user_name),
        password: Some(password),
        properties,
    }
}

///
/// 获取可变报文头数据
///
pub fn get_connect_variable_header(data: &[u8]) -> (VariableHeader, &[u8]) {
    let slice = get_remaining_data(data);
    let protocol_name = Option::from(String::from_utf8_lossy(slice).into_owned());
    let clean_session = (data[7] >> 1) & 1;
    let will_flag = (data[7] >> 2) & 1;
    let will_qos = (data[7] >> 3) & 3;
    let will_retain = (data[7] >> 5) & 1;
    let password_flag = (data[7] >> 6) & 1;
    let username_flag = (data[7] >> 7) & 1;
    let (keep_alive, _) = parse_short_int(data.get(8..10).unwrap());

    (
        VariableHeader {
            protocol_name,
            keep_alive: Some(keep_alive),
            protocol_level: MqttProtocolLevel::try_from(data[6]).ok(),
            clean_session: MqttCleanSession::try_from(clean_session).ok(),
            will_flag: MqttWillFlag::try_from(will_flag).ok(),
            will_qos: MqttQos::try_from(will_qos).ok(),
            will_retain: MqttRetain::try_from(will_retain).ok(),
            password_flag: MqttPasswordFlag::try_from(password_flag).ok(),
            username_flag: MqttUsernameFlag::try_from(username_flag).ok(),
        },
        data.get(10..).unwrap()
    )
}

///
/// 解析报文 byte 数据
///
pub fn parse_byte(data: &[u8]) -> (u8, &[u8]) {
    (data[0], data.get(1..).unwrap())
}

///
/// 解析报文 short int 数据
///
pub fn parse_short_int(data: &[u8]) -> (u16, &[u8]) {
    // println!("parse_short_int: {:?}", data.get(..2).unwrap());
    let bytes = data.get(..2).unwrap();
    let short_int_bytes = bytes.iter().rev().cloned().collect::<Vec<u8>>();
    let short_int = u16::from_le_bytes(short_int_bytes.try_into().unwrap());
    // println!("short int: {}", short_int);
    (short_int, data.get(2..).unwrap())
}

///
/// 解析报文 long int 数据
///
pub fn parse_long_int(data: &[u8]) -> (u32, &[u8]) {
    // println!("parse_long_int: {:?}", data.get(..4).unwrap());
    let bytes = data.get(..4).unwrap();
    let long_int_bytes = bytes.iter().rev().cloned().collect::<Vec<u8>>();
    let long_int = u32::from_le_bytes(long_int_bytes.try_into().unwrap());
    // println!("long int: {}", long_int);
    (long_int, data.get(4..).unwrap())
}

///
/// 解析报文 string 数据
///
pub fn parse_string(data: &[u8]) -> Result<(String, Option<&[u8]>), &str> {
    let length = data[1];
    if length as usize > data.len() {
        return Err("parse string length error");
    }
    let value = get_remaining_data(data);
    Ok((String::from_utf8(value.to_vec()).expect("parse utf-8 string"), data.get(2 + (length as usize)..)))
}

///
/// 获取报文剩余长度数据
///
/// from http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.pdf 第19页
///
///
pub fn get_remaining_length(data: &[u8]) -> Result<(usize, usize), &'static str> {
    let (mut head_index, mut digit, mut multiplier, mut value) = (1_usize, 0, 1, 0);

    loop {
        digit = data[head_index] & 127;
        value += digit as usize * multiplier;
        multiplier *= 128;
        if multiplier > 128 * 128 * 128 {
            return Err("Malformed Variable Byte Integer");
        }
        head_index += 1;
        if (digit & 128) == 0 { break; }
    }

    Ok((value, head_index))
}

///
/// 后续需要处理的数据
///
pub fn get_remaining_data(data: &[u8]) -> &[u8] {
    // println!("remaining_data handle data: {:?}", data);
    let (remaining_length, head_bytes) = get_remaining_length(data).unwrap();
    // println!("remaining_length: {}", remaining_length);
    // println!("last_data: {:?}", data.get(head_bytes..(remaining_length + head_bytes)).unwrap());
    data.get(head_bytes..(remaining_length + head_bytes)).unwrap()
}


pub fn unpack_var_int(data: &[u8]) -> (String, &[u8]) {
    let (remaining_length, head_bytes) = get_remaining_length(data).unwrap();
    let (mut result, mut shift) = (0, 0);

    for i in 0..head_bytes {
        shift += 1;
        result |= (data[i] & 127) << (shift * 7);
    }
    let val = String::from_utf8_lossy(data.get(head_bytes..(remaining_length + head_bytes)).unwrap()).into_owned();
    (val, data.get((remaining_length + head_bytes)..).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        // let data = vec![192_u8, 0_u8];
        // assert_eq!(TypeKind::PINGREQ, get_type(&data));
        // println!("{:?}", format!("{:b}", 192));
        let mut arr = [0, 0, 14, 16];
        let a = arr.iter().rev();
        println!("{:?}", a.cloned().collect::<Vec<i32>>());
        // println!("{}", u32::from_le_bytes([16, 14, 0, 0]));
        // let a =  3600_u32.to_ne_bytes();
    }

    fn read_be_u16(input: &mut &[u8]) -> u16 {
        let (int_bytes, rest) = input.split_at(std::mem::size_of::<u16>());
        *input = rest;
        u16::from_be_bytes(int_bytes.try_into().unwrap())
    }
}
