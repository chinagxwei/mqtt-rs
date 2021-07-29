use crate::message::BaseMessage;
use crate::tools::un_pack_tool::{get_connect_variable_header, parse_byte, get_connect_payload_data};
use crate::hex::UnPackProperty;
use crate::message::v5::{ConnectMessage, ConnectMessagePayload};

pub struct Unpcak;

impl Unpcak {
    pub fn connect(mut base: BaseMessage) -> ConnectMessage {
        let message_bytes = base.bytes.as_slice();
        let (mut variable_header, last_data) = get_connect_variable_header(message_bytes);

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
            variable_header.will_flag.unwrap(),
            variable_header.username_flag.unwrap(),
            variable_header.password_flag.unwrap(),
        );

        println!("client ID: {}", client_id);
        ConnectMessage {
            msg_type: base.msg_type,
            protocol_name: variable_header.protocol_name.unwrap(),
            protocol_level: variable_header.protocol_level.unwrap(),
            clean_session: variable_header.clean_session.unwrap(),
            will_flag: variable_header.will_flag.unwrap(),
            will_qos: variable_header.will_qos.unwrap(),
            will_retain: variable_header.will_retain.unwrap(),
            keep_alive: variable_header.keep_alive.unwrap(),
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
