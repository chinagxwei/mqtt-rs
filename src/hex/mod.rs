use num_enum::TryFromPrimitive;
use crate::types::TypeKind;
use std::convert::{TryFrom, Infallible};
use crate::tools::un_pack_tool::{parse_long_int, parse_string, parse_byte, parse_short_int, get_remaining_data, get_remaining_length, var_int};

pub mod reason_code;

#[derive(Debug, Clone)]
pub enum PropertyValue {
    Long(u32),
    Short(u16),
    Byte(u8),
    String(String),
    Map(String, String),
}

#[derive(Debug, Clone)]
pub struct PropertyItem(Property, PropertyValue);

#[derive(Debug, Copy, Clone, TryFromPrimitive)]
#[repr(u8)]
pub enum Property {
    PayloadFormatIndicator = 0x01,
    MessageExpiryInterval = 0x02,
    ContentType = 0x03,
    ResponseTopic = 0x08,
    CorrelationData = 0x09,
    SubscriptionIdentifier = 0x0B,
    SessionExpiryInterval = 0x11,
    AssignedClientIdentifier = 0x12,
    ServerKeepAlive = 0x13,
    AuthenticationMethod = 0x15,
    AuthenticationData = 0x16,
    RequestProblemInformation = 0x17,
    WillDelayInterval = 0x18,
    RequestResponseInformation = 0x19,
    ResponseInformation = 0x1A,
    ServerReference = 0x1C,
    ReasonString = 0x1F,
    ReceiveMaximum = 0x21,
    TopicAliasMaximum = 0x22,
    TopicAlias = 0x23,
    MaximumQos = 0x24,
    RetainAvailable = 0x25,
    UserProperty = 0x26,
    MaximumPacketSize = 0x27,
    WildcardSubscriptionAvailable = 0x28,
    SubscriptionIdentifierAvailable = 0x29,
    SharedSubscriptionAvailable = 0x2A,
}

impl Property {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Property::PayloadFormatIndicator => { "payload_format_indicator" }
            Property::MessageExpiryInterval => { "message_expiry_interval" }
            Property::ContentType => { "content_type" }
            Property::ResponseTopic => { "response_topic" }
            Property::CorrelationData => { "correlation_data" }
            Property::SubscriptionIdentifier => { "subscription_identifier" }
            Property::SessionExpiryInterval => { "session_expiry_interval" }
            Property::AssignedClientIdentifier => { "assigned_client_identifier" }
            Property::ServerKeepAlive => { "server_keep_alive" }
            Property::AuthenticationMethod => { "authentication_method" }
            Property::AuthenticationData => { "authentication_data" }
            Property::RequestProblemInformation => { "request_problem_information" }
            Property::WillDelayInterval => { "will_delay_interval" }
            Property::RequestResponseInformation => { "request_response_information" }
            Property::ResponseInformation => { "response_information" }
            Property::ServerReference => { "server_reference" }
            Property::ReasonString => { "reason_string" }
            Property::ReceiveMaximum => { "receive_maximum" }
            Property::TopicAliasMaximum => { "topic_alias_maximum" }
            Property::TopicAlias => { "topic_alias" }
            Property::MaximumQos => { "maximum_qos" }
            Property::RetainAvailable => { "retain_available" }
            Property::UserProperty => { "user_property" }
            Property::MaximumPacketSize => { "maximum_packet_size" }
            Property::WildcardSubscriptionAvailable => { "wildcard_subscription_available" }
            Property::SubscriptionIdentifierAvailable => { "subscription_identifier_available" }
            Property::SharedSubscriptionAvailable => { "shared_subscription_available" }
        }
    }
}

impl Property {
    pub fn is_connect_property(&self) -> bool {
        match self {
            Property::SessionExpiryInterval |
            Property::AuthenticationMethod |
            Property::AuthenticationData |
            Property::RequestProblemInformation |
            Property::RequestResponseInformation |
            Property::ReceiveMaximum |
            Property::TopicAlias |
            Property::UserProperty |
            Property::MaximumPacketSize => { true }
            _ => { false }
        }
    }

    pub fn is_connack_property(&self) -> bool {
        match self {
            Property::SessionExpiryInterval |
            Property::AssignedClientIdentifier |
            Property::ServerKeepAlive |
            Property::AuthenticationMethod |
            Property::AuthenticationData |
            Property::ResponseInformation |
            Property::ServerReference |
            Property::ReasonString |
            Property::ReceiveMaximum |
            Property::TopicAliasMaximum |
            Property::MaximumQos |
            Property::RetainAvailable |
            Property::UserProperty |
            Property::MaximumPacketSize |
            Property::WildcardSubscriptionAvailable |
            Property::SubscriptionIdentifierAvailable |
            Property::SharedSubscriptionAvailable => { true }
            _ => { false }
        }
    }

    pub fn is_publish_property(&self) -> bool {
        match self {
            Property::PayloadFormatIndicator |
            Property::MessageExpiryInterval |
            Property::ContentType |
            Property::ResponseTopic |
            Property::CorrelationData |
            Property::SubscriptionIdentifier |
            Property::TopicAlias |
            Property::UserProperty => { true }
            _ => { false }
        }
    }

    pub fn is_pub_and_sub_property(&self) -> bool {
        match self {
            Property::ReasonString |
            Property::UserProperty => { true }
            _ => { false }
        }
    }

    pub fn is_subscribe_property(&self) -> bool {
        match self {
            Property::SubscriptionIdentifier |
            Property::UserProperty => { true }
            _ => { false }
        }
    }

    pub fn is_unsubscribe_property(&self) -> bool {
        match self {
            Property::UserProperty => { true }
            _ => { false }
        }
    }

    pub fn is_disconnect_property(&self) -> bool {
        match self {
            Property::SessionExpiryInterval |
            Property::ServerReference |
            Property::ReasonString |
            Property::UserProperty => { true }
            _ => { false }
        }
    }

    pub fn is_auth_property(&self) -> bool {
        match self {
            Property::AuthenticationMethod |
            Property::AuthenticationData |
            Property::ReasonString |
            Property::UserProperty => { true }
            _ => { false }
        }
    }

    pub fn is_will_properties(&self) -> bool {
        match self {
            Property::PayloadFormatIndicator |
            Property::MessageExpiryInterval |
            Property::ContentType |
            Property::ResponseTopic |
            Property::CorrelationData |
            Property::WillDelayInterval |
            Property::UserProperty => { true }
            _ => { false }
        }
    }

    pub fn property_handle<'a>(&self, length: &mut u32, data: &'a [u8]) -> Option<(PropertyItem, &'a [u8])> {
        match self {
            Property::SessionExpiryInterval |
            Property::MessageExpiryInterval |
            Property::WillDelayInterval |
            Property::MaximumPacketSize => {
                let (val, last_data) = parse_long_int(data);
                *length -= 5;
                Some((PropertyItem(Property::SessionExpiryInterval, PropertyValue::Long(val)), last_data))
            }
            Property::ContentType |
            Property::ResponseTopic |
            Property::CorrelationData |
            Property::AssignedClientIdentifier |
            Property::ResponseInformation |
            Property::ServerReference |
            Property::ReasonString |
            Property::AuthenticationMethod |
            Property::AuthenticationData => {
                let (val, last_data) = parse_string(data).unwrap();
                *length -= (val.len() as u32 + 3);
                Some((PropertyItem(*self, PropertyValue::String(val)), last_data.unwrap()))
            }
            Property::PayloadFormatIndicator |
            Property::MaximumQos |
            Property::RetainAvailable |
            Property::WildcardSubscriptionAvailable |
            Property::SubscriptionIdentifierAvailable |
            Property::SharedSubscriptionAvailable |
            Property::RequestProblemInformation |
            Property::RequestResponseInformation => {
                let (val, last_data) = parse_byte(data);
                *length -= 2;
                Some((PropertyItem(*self, PropertyValue::Byte(val)), last_data))
            }
            Property::ServerKeepAlive |
            Property::ReceiveMaximum |
            Property::TopicAlias |
            Property::TopicAliasMaximum => {
                let (val, last_data) = parse_short_int(data);
                *length -= 3;
                Some((PropertyItem(*self, PropertyValue::Short(val)), last_data))
            }
            Property::UserProperty => {
                let (user_key, last_data) = parse_string(data).unwrap();
                let (user_value, last_data) = parse_string(last_data.unwrap()).unwrap();
                *length -= (user_key.len() as u32 + user_value.len() as u32 + 5);
                Some((PropertyItem(Property::UserProperty, PropertyValue::Map(user_key, user_value)), last_data.unwrap()))
            }
            Property::SubscriptionIdentifier => {
                let (val, last_data) = var_int(data);
                *length -= (val.len() as u32 + 1);
                Some((PropertyItem(Property::SubscriptionIdentifier, PropertyValue::String(val)), last_data))
            }
        }
    }
}

pub struct UnPackProperty;

impl UnPackProperty {
    pub fn connect(mut length: u32, mut data: &[u8]) -> Vec<PropertyItem> {
        let mut properties = vec![];
        loop {
            let property = data[0];
            match Property::try_from(property) {
                Ok(p) => {
                    if p.is_connect_property() {
                        if let Some((item, last_data)) = p.property_handle(&mut length, data.get(1..).unwrap()) {
                            data = last_data;
                            properties.push(item);
                        }
                    } else {
                        // return Err(format!("Property 0x{:X} not exist",property));
                    }
                }
                Err(e) => {
                    println!("Property {:?} not exist", e)
                }
            }

            println!("{}", property);

            if length <= 0 {
                break;
            }
        }
        println!("{:?}", properties);
        properties
    }

    pub fn connack(mut length: u32, mut data: &[u8])-> Vec<PropertyItem>{
        let mut properties = vec![];
        loop {
            let property = data[0];
            match Property::try_from(property) {
                Ok(p) => {
                    if p.is_connack_property() {
                        if let Some((item, last_data)) = p.property_handle(&mut length, data.get(1..).unwrap()) {
                            data = last_data;
                            properties.push(item);
                        }
                    } else {
                        // return Err(format!("Property 0x{:X} not exist",property));
                    }
                }
                Err(e) => {
                    println!("Property {:?} not exist", e)
                }
            }

            println!("{}", property);

            if length <= 0 {
                break;
            }
        }
        println!("{:?}", properties);
        properties
    }
}
