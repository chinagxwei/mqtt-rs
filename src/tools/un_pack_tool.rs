use crate::types::TypeKind;
use std::convert::TryFrom;
use crate::protocol::{MqttProtocolLevel, MqttCleanSession, MqttWillFlag, MqttUsernameFlag, MqttPasswordFlag, MqttRetain, MqttQos, MqttDup};

pub fn get_type(data: &[u8]) -> Option<TypeKind> {
    TypeKind::try_from((data[0] >> 4)).ok()
}

pub fn get_type_have_data(data: &[u8]) -> (Option<TypeKind>, Vec<u8>) {
    (TypeKind::try_from((data[0] >> 4)).ok(), get_remaining_data(data))
}

pub fn get_protocol_name_and_version(data: &[u8]) -> (Option<String>, Option<MqttProtocolLevel>) {
    let length = data[1];
    let slice = data.get(2..(2 + length) as usize);
    let protocol_name = Option::from(String::from_utf8_lossy(slice.unwrap()).into_owned());
    let mqtt_version = MqttProtocolLevel::try_from(data[6]).ok();
    (protocol_name, mqtt_version)
}

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

pub fn get_variable_header(data: &[u8]) -> (
    Option<String>,
    Option<u32>,
    Option<MqttProtocolLevel>,
    Option<MqttCleanSession>,
    Option<MqttWillFlag>,
    Option<MqttQos>,
    Option<MqttRetain>,
    Option<MqttPasswordFlag>,
    Option<MqttUsernameFlag>
) {
    println!("get_variable_header data: {:?}", data);
    let length = data[1];
    let slice = data.get(2..(2 + length) as usize);
    let protocol_name = Option::from(String::from_utf8_lossy(slice.unwrap()).into_owned());
    let clean_session = (data[7] >> 1) & 1;
    let will_flag = (data[7] >> 2) & 1;
    let will_qos = (data[7] >> 3) & 3;
    let will_retain = (data[7] >> 5) & 1;
    let password_flag = (data[7] >> 6) & 1;
    let username_flag = (data[7] >> 7) & 1;
    let keep_alive = parse_number(data.get(8..10).unwrap());
    (
        protocol_name,
        Some(keep_alive),
        MqttProtocolLevel::try_from(data[6]).ok(),
        MqttCleanSession::try_from(clean_session).ok(),
        MqttWillFlag::try_from(will_flag).ok(),
        MqttQos::try_from(will_qos).ok(),
        MqttRetain::try_from(will_retain).ok(),
        MqttPasswordFlag::try_from(password_flag).ok(),
        MqttUsernameFlag::try_from(username_flag).ok(),
    )
}

pub fn get_payload_data(data: &[u8], will_flag: MqttWillFlag, username_flag: MqttUsernameFlag, password_flag: MqttPasswordFlag)
                        -> (String, Option<String>, Option<String>, Option<String>, Option<String>) {
    let (client_id, last_data) = parse_string(data.get(10..).unwrap()).unwrap();

    let (will_topic, will_message, last_data) = if MqttWillFlag::Enable == will_flag {
        let (will_topic, will_last_data) = parse_string(last_data.clone().unwrap()).unwrap();
        let (will_message, will_last_data) = parse_string(will_last_data.unwrap()).unwrap();
        (Some(will_topic), Some(will_message), will_last_data)
    } else {
        (None, None, last_data)
    };

    let (user_name, last_data) = if MqttUsernameFlag::Enable == username_flag {
        parse_string(last_data.clone().unwrap()).unwrap()
    } else {
        ("".to_string(), last_data)
    };

    let (password, _) = if MqttPasswordFlag::Enable == password_flag {
        parse_string(last_data.clone().unwrap()).unwrap()
    } else {
        ("".to_string(), last_data)
    };

    (client_id, will_topic, will_message, Some(user_name), Some(password))
}

pub fn parse_number(data: &[u8]) -> u32 {
    u32::from_le_bytes([data[1], data[0], 0, 0])
}

pub fn parse_number_to_vec(data: &[u8]) -> (u32, Vec<u8>) {
    let num = u32::from_le_bytes([data[1], data[0], 0, 0]);
    let last_data = data.get(2..).unwrap().to_vec();
    (num, last_data)
}

pub fn parse_string(data: &[u8]) -> Result<(String, Option<&[u8]>), &str> {
    println!("parse_string: {:?}", data);
    let length = data[1];
    if length as usize > data.len() {
        return Err("parse string length error");
    }
    let value = &data[2..(length + 2) as usize];
    Ok((String::from_utf8(value.to_vec()).expect("parse utf-8 string"), data.get(2 + (length as usize)..)))
}

pub fn parse_string_to_vec(data: &[u8]) -> Result<(String, Vec<u8>), &str> {
    let length = data[1];
    if length as usize > data.len() {
        return Err("parse string length error");
    }
    let value = get_remaining_data(data);
    let utf_8_str = String::from_utf8(value.to_vec()).expect("parse utf-8 string");
    let last_data = data.get(2 + (length as usize)..).unwrap().to_vec();
    Ok((utf_8_str, last_data))
}

fn get_remaining_length(data: &[u8]) -> Result<(usize, usize), &'static str> {
    let (ref mut head_index, mut digit, mut multiplier, mut value) = (1_usize, 0, 1, 0);
    loop {
        digit = data[*head_index] & 127;
        value += digit as usize * multiplier;
        if multiplier > 128 * 128 * 128 {
            return Err("Malformed Variable Byte Integer");
        }
        multiplier *= 128;
        *head_index += 1;
        if (digit & 128) == 0 {
            break;
        }
    }
    Ok((value, *head_index))
}

pub fn get_remaining_data(data: &[u8]) -> Vec<u8> {
    let (remaining_length, head_bytes) = get_remaining_length(data).unwrap();
    data.get(head_bytes..(remaining_length + head_bytes)).unwrap().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        // let data = vec![192_u8, 0_u8];
        // assert_eq!(TypeKind::PINGREQ, get_type(&data));
        // println!("{:?}", format!("{:b}", 192).as_bytes());
        println!("{}", u32::from_le_bytes([254, 164, 0, 0]));
        // let a =  3600_u32.to_ne_bytes();
    }
}
