use crate::message::v5::{ConnectMessage, SubackMessage, UnsubackMessage, DisconnectMessage, AuthMessage, SubscribeMessage, CommonPayloadMessage};
use crate::tools::pack_tool::{pack_protocol_name, pack_connect_flags, pack_string, pack_short_int, pack_client_id, pack_header, pack_message_short_id, pack_publish_header};
use crate::protocol::{MqttWillFlag, MqttSessionPresent, MqttQos, MqttDup};
use crate::hex::{pack_property, PropertyItem};
use crate::types::TypeKind;
use crate::hex::reason_code::{ReasonCodeV5, ReasonPhrases};

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

    body.extend(pack_short_int(msg.keep_alive as u16));

    if msg.properties.is_some() {
        body.extend(pack_property::connect(msg.properties.as_ref().unwrap()));
    }

    body.extend(pack_client_id(&msg.payload.client_id));

    if msg.will_flag == MqttWillFlag::Enable {
        if msg.payload.properties.is_some() {
            body.extend(pack_property::will_properties(msg.payload.properties.as_ref().unwrap()));
        }

        if msg.payload.will_topic.is_some() {
            let will_topic = pack_string(msg.payload.will_topic.as_ref().unwrap());
            body.extend(will_topic);
        }
        if msg.payload.will_message.is_some() {
            let will_message = pack_string(msg.payload.will_message.as_ref().unwrap());
            body.extend(will_message);
        }
    }

    if msg.payload.user_name.is_some() {
        let username = pack_string(msg.payload.user_name.as_ref().unwrap());
        body.extend(username);
    }

    if msg.payload.password.is_some() {
        let password = pack_string(msg.payload.password.as_ref().unwrap());
        body.extend(password);
    }

    let mut package = pack_header(msg.msg_type, body.len());

    package.extend(body);

    package
}

pub fn connack(session_present: MqttSessionPresent, return_code: ReasonCodeV5, properties: Option<&Vec<PropertyItem>>) -> Vec<u8> {
    let mut body = vec![session_present as u8, return_code.as_byte()];

    if properties.is_some() {
        body.extend(pack_property::connack(properties.unwrap()));
    }

    let mut package = pack_header(TypeKind::CONNACK, body.len());

    package.extend(body);

    package
}

pub fn subscribe(mut data: Vec<SubscribeMessage>) -> Vec<u8> {
    let collect = data.iter_mut().map(|msg| {
        let mut body = pack_message_short_id(msg.message_id);

        if msg.properties.is_some() {
            body.extend(pack_property::subscribe(msg.properties.as_ref().unwrap()));
        }

        let topic = pack_string(&msg.topic);

        body.extend(topic);

        let mut option = 0;

        if let Some(qos) = msg.qos {
            option |= qos.as_byte();
        }

        if let Some(nl) = msg.no_local {
            option |= (nl as u8) << 2;
        }

        if let Some(rap) = msg.retain_as_published {
            option |= (rap as u8) << 3;
        }

        if let Some(rh) = msg.retain_handling {
            option |= rh << 4;
        }

        body.push(option);

        let mut package = pack_header(TypeKind::SUBSCRIBE, body.len());

        package.extend(body);

        package
    }).collect::<Vec<Vec<u8>>>();

    collect.concat()
}

pub fn suback(msg: &SubackMessage) -> Vec<u8> {
    let mut body = pack_message_short_id(msg.message_id);

    if msg.properties.is_some() {
        body.extend(pack_property::suback(msg.properties.as_ref().unwrap()));
    }

    body.extend(msg.codes.clone());

    let mut package = pack_header(TypeKind::SUBACK, body.len());

    package.extend(body);

    package
}

pub fn unsuback(msg: &UnsubackMessage) -> Vec<u8> {
    let mut body = pack_message_short_id(msg.message_id);

    if msg.properties.is_some() {
        body.extend(pack_property::suback(msg.properties.as_ref().unwrap()));
    }

    body.extend(msg.codes.clone());

    let mut package = pack_header(TypeKind::UNSUBACK, body.len());

    package.extend(body);

    package
}

pub fn disconnect(msg: &DisconnectMessage) -> Vec<u8> {
    let mut body = msg.code.to_ne_bytes().to_vec();

    if msg.properties.is_some() {
        body.extend(pack_property::disconnect(msg.properties.as_ref().unwrap()));
    }

    let mut package = pack_header(TypeKind::DISCONNECT, body.len());

    package.extend(body);

    package
}

pub fn auth(msg: &AuthMessage) -> Vec<u8> {
    let mut body = msg.code.to_ne_bytes().to_vec();

    if msg.properties.is_some() {
        body.extend(pack_property::auth(msg.properties.as_ref().unwrap()));
    }

    let mut package = pack_header(TypeKind::AUTH, body.len());

    package.extend(body);

    package
}

pub fn common(message_id: u16, code: ReasonPhrases, properties: Option<&Vec<PropertyItem>>, kind: TypeKind) -> Vec<u8> {
    let mut body = pack_message_short_id(message_id);

    if properties.is_some() {
        body.extend(pack_property::suback(properties.unwrap()));
    }

    body.push(code.as_byte());

    let mut package = if kind.is_pubrel() {
        pack_publish_header(kind, body.len(), Option::from(MqttQos::Qos1), Option::from(MqttDup::Disable), None)
    } else {
        pack_header(kind, body.len())
    };

    package.extend(body);

    package
}


