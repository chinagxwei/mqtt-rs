use crate::message::BaseMessage;
use crate::tools::un_pack_tool::{get_connect_variable_header, parse_byte, get_connect_payload_data};
use crate::hex::UnPackProperty;
use crate::message::v5::{ConnectMessage, ConnectMessagePayload};

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

        println!("v5 {:?}", last_data);

        let (properties_total_length, last_data) = parse_byte(last_data);

        println!("properties total length: {}", properties_total_length);

        let (properties, last_data) = if properties_total_length > 0 {
            (
                UnPackProperty::connect(properties_total_length as u32, last_data),
                last_data.get(properties_total_length as usize..).unwrap()
            )
        } else {
            (vec![], last_data)
        };

        // println!("{:?}", last_data.get(properties_total_length as usize..).unwrap());
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

        println!("client ID: {}", client_id);
        ConnectMessage {
            msg_type: base.msg_type,
            protocol_name: protocol_name.unwrap(),
            protocol_level: protocol_level.unwrap(),
            clean_session: clean_session.unwrap(),
            will_flag: will_flag.unwrap(),
            will_qos: will_qos.unwrap(),
            will_retain: will_retain.unwrap(),
            keep_alive: keep_alive.unwrap(),
            payload: ConnectMessagePayload {
                client_id,
                will_topic,
                will_message,
                user_name,
                password,
                properties: Some(properties),
            },
            bytes: Some(base.bytes),
        }
    }
}
