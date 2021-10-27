use crate::message::{MqttMessageType, MqttMessageVersion};
use crate::packet::{v5_packet};
use crate::message::entity::{AuthMessage, CommonPayloadMessage, ConnackMessage, ConnectMessage, DisconnectMessage, PingreqMessage, PingrespMessage, PubackMessage, PubcompMessage, PublishMessage, PubrecMessage, PubrelMessage, SubackMessage, SubscribeMessage, UnsubackMessage, UnsubscribeMessage};
use crate::tools::pack_tool::pack_header;
use crate::tools::protocol::MqttProtocolLevel;


#[derive(Debug, Clone)]
pub enum MqttMessageV5 {
    Connect(ConnectMessage),
    Connack(ConnackMessage),
    Publish(PublishMessage),
    Puback(PubackMessage),
    Pubrec(PubrecMessage),
    Pubrel(PubrelMessage),
    Pubcomp(PubcompMessage),
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
            MqttMessageV5::Puback(msg) => { Some(v5_packet::common(msg.message_id, msg.code.unwrap().as_byte(), msg.properties.as_ref(), msg.get_message_type())) }
            MqttMessageV5::Pubrec(msg) => { Some(v5_packet::common(msg.message_id, msg.code.unwrap().as_byte(), msg.properties.as_ref(), msg.get_message_type())) }
            MqttMessageV5::Pubrel(msg) => { Some(v5_packet::common(msg.message_id, msg.code.unwrap().as_byte(), msg.properties.as_ref(), msg.get_message_type())) }
            MqttMessageV5::Pubcomp(msg) => { Some(v5_packet::common(msg.message_id, msg.code.unwrap().as_byte(), msg.properties.as_ref(), msg.get_message_type())) }
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

impl MqttMessageVersion for MqttMessageV5 {
    fn version(&self) -> Option<MqttProtocolLevel> {
        match self {
            MqttMessageV5::Connect(msg) => Some(msg.protocol_level),
            MqttMessageV5::Connack(msg) => msg.protocol_level,
            MqttMessageV5::Publish(msg) => msg.protocol_level,
            MqttMessageV5::Puback(msg) => msg.protocol_level,
            MqttMessageV5::Pubrec(msg) => msg.protocol_level,
            MqttMessageV5::Pubrel(msg) => msg.protocol_level,
            MqttMessageV5::Pubcomp(msg) => msg.protocol_level,
            MqttMessageV5::Subscribe(msg) => msg.protocol_level,
            MqttMessageV5::Suback(msg) => msg.protocol_level,
            MqttMessageV5::Unsubscribe(msg) => msg.protocol_level,
            MqttMessageV5::Unsuback(msg) => msg.protocol_level,
            MqttMessageV5::Pingreq(msg) => msg.protocol_level,
            MqttMessageV5::Pingresp(msg) => msg.protocol_level,
            MqttMessageV5::Disconnect(msg) => msg.protocol_level,
            MqttMessageV5::Auth(msg) => msg.protocol_level,
        }
    }
}

#[macro_export]
macro_rules! impl_mqtt_message_v5 {
    ($name:ident,$kind:ident) => {
        impl From<$name> for MqttMessageV5 {
            fn from(msg: $name) -> Self {
                MqttMessageV5::$kind(msg)
            }
        }
    };
}

impl_mqtt_message_v5!(ConnectMessage,Connect);
impl_mqtt_message_v5!(ConnackMessage,Connack);
impl_mqtt_message_v5!(PublishMessage,Publish);
impl_mqtt_message_v5!(PubackMessage,Puback);
impl_mqtt_message_v5!(PubrecMessage,Pubrec);
impl_mqtt_message_v5!(PubrelMessage,Pubrel);
impl_mqtt_message_v5!(PubcompMessage,Pubcomp);
impl_mqtt_message_v5!(SubscribeMessage,Subscribe);
impl_mqtt_message_v5!(SubackMessage,Suback);
impl_mqtt_message_v5!(UnsubscribeMessage,Unsubscribe);
impl_mqtt_message_v5!(PingreqMessage,Pingreq);
impl_mqtt_message_v5!(PingrespMessage,Pingresp);
impl_mqtt_message_v5!(DisconnectMessage,Disconnect);
impl_mqtt_message_v5!(AuthMessage,Auth);

