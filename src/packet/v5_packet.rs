use crate::message::v5::ConnectMessage;
use crate::tools::pack_tool::{pack_protocol_name, pack_connect_flags, pack_string, pack_short_int, pack_client_id, pack_header};
use crate::protocol::MqttWillFlag;
use crate::hex::pack_property;

pub fn connect(msg: &ConnectMessage) -> Vec<u8> {
    let mut body: Vec<u8> = pack_string(&msg.protocol_name);

    body.push(msg.protocol_level as u8);

    body.push(
        pack_connect_flags(
            msg.clean_session,
            msg.will_flag,
            msg.will_qos,
            msg.will_retain,
            msg.payload.user_name.as_ref(),
            msg.payload.password.as_ref(),
        ).unwrap());

    body = [body, pack_short_int(msg.keep_alive as u16)].concat();

    body = [body, pack_property::connect(msg.payload.properties.as_ref().unwrap())].concat();

    body = [body, pack_client_id(&msg.payload.client_id)].concat();

    if msg.will_flag == MqttWillFlag::Enable {
        if msg.payload.will_topic.is_some() {
            let will_topic = pack_string(msg.payload.will_topic.as_ref().unwrap());
            body = [body, will_topic].concat();
        }
        if msg.payload.will_message.is_some() {
            let will_message = pack_string(msg.payload.will_message.as_ref().unwrap());
            body = [body, will_message].concat();
        }
    }

    if msg.payload.user_name.is_some() {
        let username = pack_string(msg.payload.user_name.as_ref().unwrap());
        body = [body, username].concat();
    }

    if msg.payload.password.is_some() {
        let password = pack_string(msg.payload.password.as_ref().unwrap());
        body = [body, password].concat();
    }

    let header = pack_header(msg.msg_type, body.len());

    [header, body].concat()
}
