use crate::message::MqttMessageType;
use crate::packet::{v5_packet};
use crate::message::entity::{AuthMessage, CommonPayloadMessage, ConnackMessage, ConnectMessage, DisconnectMessage, PingreqMessage, PingrespMessage, PublishMessage, SubackMessage, SubscribeMessage, UnsubackMessage, UnsubscribeMessage};
use crate::tools::pack_tool::pack_header;


#[derive(Debug, Clone)]
pub enum MqttMessageV5 {
    Connect(ConnectMessage),
    Connack(ConnackMessage),
    Publish(PublishMessage),
    Puback(CommonPayloadMessage),
    Pubrec(CommonPayloadMessage),
    Pubrel(CommonPayloadMessage),
    Pubcomp(CommonPayloadMessage),
    Subscribe(SubscribeMessage),
    Suback(SubackMessage),
    Unsubscribe(UnsubscribeMessage),
    Unsuback(UnsubackMessage),
    Pingreq(PingreqMessage),
    Pingresp(PingrespMessage),
    Disconnect(DisconnectMessage),
    Auth(AuthMessage),
}

impl MqttMessageV5 {
    pub fn is_connect(&self) -> bool {
        matches!(self, MqttMessageV5::Connect(_))
    }

    pub fn is_cannack(&self) -> bool {
        matches!(self, MqttMessageV5::Connack(_))
    }

    pub fn is_publish(&self) -> bool {
        matches!(self, MqttMessageV5::Publish(_))
    }

    pub fn is_puback(&self) -> bool {
        matches!(self, MqttMessageV5::Puback(_))
    }

    pub fn is_pubrec(&self) -> bool {
        matches!(self, MqttMessageV5::Pubrec(_))
    }

    pub fn is_pubrel(&self) -> bool {
        matches!(self, MqttMessageV5::Pubrel(_))
    }

    pub fn is_pubcomp(&self) -> bool {
        matches!(self, MqttMessageV5::Pubcomp(_))
    }

    pub fn is_subscribe(&self) -> bool {
        matches!(self, MqttMessageV5::Subscribe(_))
    }

    pub fn is_suback(&self) -> bool {
        matches!(self, MqttMessageV5::Suback(_))
    }

    pub fn is_unsubscribe(&self) -> bool {
        matches!(self, MqttMessageV5::Unsubscribe(_))
    }

    pub fn is_unsuback(&self) -> bool {
        matches!(self, MqttMessageV5::Unsuback(_))
    }

    pub fn is_pingreq(&self) -> bool {
        matches!(self, MqttMessageV5::Pingreq(_))
    }

    pub fn is_pingresp(&self) -> bool {
        matches!(self, MqttMessageV5::Pingresp(_))
    }

    pub fn is_disconnect(&self) -> bool {
        matches!(self, MqttMessageV5::Disconnect(_))
    }

    pub fn is_auth(&self) -> bool {
        matches!(self, MqttMessageV5::Disconnect(_))
    }
}

impl MqttMessageV5 {
    pub fn to_vec(&self) -> Option<Vec<u8>> {
        match self {
            MqttMessageV5::Connect(msg) => { Some(v5_packet::connect(msg)) }
            MqttMessageV5::Connack(msg) => { Some(v5_packet::connack(msg.session_present, msg.return_code.unwrap(), msg.properties.as_ref())) }
            MqttMessageV5::Publish(msg) => { Some(v5_packet::publish(msg)) }
            MqttMessageV5::Puback(msg) => { Some(v5_packet::common(msg.message_id, msg.code.as_byte(), msg.properties.as_ref(), msg.get_message_type())) }
            MqttMessageV5::Pubrec(msg) => { Some(v5_packet::common(msg.message_id, msg.code.as_byte(), msg.properties.as_ref(), msg.get_message_type())) }
            MqttMessageV5::Pubrel(msg) => { Some(v5_packet::common(msg.message_id, msg.code.as_byte(), msg.properties.as_ref(), msg.get_message_type())) }
            MqttMessageV5::Pubcomp(msg) => { Some(v5_packet::common(msg.message_id, msg.code.as_byte(), msg.properties.as_ref(), msg.get_message_type())) }
            MqttMessageV5::Subscribe(msg) => { Some(v5_packet::subscribe(msg)) }
            MqttMessageV5::Suback(msg) => { Some(v5_packet::suback(msg)) }
            MqttMessageV5::Unsubscribe(msg) => { Some(v5_packet::unsubscribe(msg)) }
            MqttMessageV5::Unsuback(msg) => { Some(v5_packet::common(msg.message_id, msg.code.unwrap(), msg.properties.as_ref(), msg.get_message_type())) }
            MqttMessageV5::Pingreq(msg) => { Some(pack_header(msg.get_message_type(), 0)) }
            MqttMessageV5::Pingresp(msg) => { Some(pack_header(msg.get_message_type(), 0)) }
            MqttMessageV5::Disconnect(msg) => { Some(pack_header(msg.get_message_type(), 0)) }
            MqttMessageV5::Auth(msg) => { Some(v5_packet::auth(msg)) }
        }
    }
}

