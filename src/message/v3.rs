use crate::tools::pack_tool::{pack_header};
use crate::packet::v3_packet;
use crate::message::MqttMessageType;
use crate::message::entity::{ConnackMessage, ConnectMessage, DisconnectMessage, PingreqMessage, PingrespMessage, PubackMessage, PubcompMessage, PublishMessage, PubrecMessage, PubrelMessage, SubackMessage, SubscribeMessage, UnsubackMessage, UnsubscribeMessage};

#[derive(Debug, Clone)]
pub enum MqttMessageV3 {
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
}

impl MqttMessageV3 {
    pub fn is_connect(&self) -> bool {
        matches!(self, Self::Connect(_))
    }

    pub fn is_cannack(&self) -> bool {
        matches!(self, Self::Connack(_))
    }

    pub fn is_publish(&self) -> bool {
        matches!(self, Self::Publish(_))
    }

    pub fn is_puback(&self) -> bool {
        matches!(self, Self::Puback(_))
    }

    pub fn is_pubrec(&self) -> bool {
        matches!(self, Self::Pubrec(_))
    }

    pub fn is_pubrel(&self) -> bool {
        matches!(self, Self::Pubrel(_))
    }

    pub fn is_pubcomp(&self) -> bool {
        matches!(self, Self::Pubcomp(_))
    }

    pub fn is_subscribe(&self) -> bool {
        matches!(self, Self::Subscribe(_))
    }

    pub fn is_suback(&self) -> bool {
        matches!(self, Self::Suback(_))
    }

    pub fn is_unsubscribe(&self) -> bool {
        matches!(self, Self::Unsubscribe(_))
    }

    pub fn is_unsuback(&self) -> bool {
        matches!(self, Self::Unsuback(_))
    }

    pub fn is_pingreq(&self) -> bool {
        matches!(self, Self::Pingreq(_))
    }

    pub fn is_pingresp(&self) -> bool {
        matches!(self, Self::Pingresp(_))
    }

    pub fn is_disconnect(&self) -> bool {
        matches!(self, Self::Disconnect(_))
    }
}

impl MqttMessageV3 {
    pub fn to_vec(&self) -> Option<Vec<u8>> {
        match self {
            MqttMessageV3::Connect(msg) => { Some(v3_packet::connect(msg)) }
            MqttMessageV3::Connack(msg) => { Some(v3_packet::connack(msg.session_present, msg.return_code.or(Some(0)))) }
            MqttMessageV3::Publish(msg) => { Some(v3_packet::publish(msg)) }
            MqttMessageV3::Puback(msg) => { Some(v3_packet::common(msg.message_id, msg.get_message_type())) }
            MqttMessageV3::Pubrec(msg) => { Some(v3_packet::common(msg.message_id, msg.get_message_type())) }
            MqttMessageV3::Pubrel(msg) => { Some(v3_packet::common(msg.message_id, msg.get_message_type())) }
            MqttMessageV3::Pubcomp(msg) => { Some(v3_packet::common(msg.message_id, msg.get_message_type())) }
            MqttMessageV3::Subscribe(msg) => { Some(v3_packet::subscribe(msg)) }
            MqttMessageV3::Suback(msg) => { Some(v3_packet::suback(msg)) }
            MqttMessageV3::Unsubscribe(msg) => { Some(v3_packet::unsubscribe(msg)) }
            MqttMessageV3::Unsuback(msg) => { Some(v3_packet::common(msg.message_id, msg.get_message_type())) }
            MqttMessageV3::Pingreq(msg) => { Some(pack_header(msg.get_message_type(), 0)) }
            MqttMessageV3::Pingresp(msg) => { Some(pack_header(msg.get_message_type(), 0)) }
            MqttMessageV3::Disconnect(msg) => { Some(pack_header(msg.get_message_type(), 0)) }
        }
    }
}

#[macro_export]
macro_rules! impl_mqtt_message_v3 {
    ($name:ident,$kind:ident) => {
        impl From<$name> for MqttMessageV3 {
            fn from(msg: $name) -> Self {
                MqttMessageV3::$kind(msg)
            }
        }
    };
}

impl_mqtt_message_v3!(ConnectMessage,Connect);
impl_mqtt_message_v3!(ConnackMessage,Connack);
impl_mqtt_message_v3!(PublishMessage,Publish);
impl_mqtt_message_v3!(PubackMessage,Puback);
impl_mqtt_message_v3!(PubrecMessage,Pubrec);
impl_mqtt_message_v3!(PubrelMessage,Pubrel);
impl_mqtt_message_v3!(PubcompMessage,Pubcomp);
impl_mqtt_message_v3!(SubscribeMessage,Subscribe);
impl_mqtt_message_v3!(SubackMessage,Suback);
impl_mqtt_message_v3!(UnsubscribeMessage,Unsubscribe);
impl_mqtt_message_v3!(PingreqMessage,Pingreq);
impl_mqtt_message_v3!(PingrespMessage,Pingresp);
impl_mqtt_message_v3!(DisconnectMessage,Disconnect);



