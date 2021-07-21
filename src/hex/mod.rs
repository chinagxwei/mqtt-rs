use num_enum::TryFromPrimitive;
use crate::types::TypeKind;
use std::convert::{TryFrom, Infallible};
use std::option::Option::Some;
use crate::tools::un_pack_tool::parse_long_int;

pub mod reason_code;

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
            Property::UserProperty |
            Property::MaximumPacketSize => { true }
            _ => { false }
        }
    }

    pub fn connect_property_handle(&self, length: u32, data: &[u8]) -> bool {
        match self {
            Property::SessionExpiryInterval => {
                parse_long_int(data);
                true
            }
            Property::AuthenticationMethod => { true }
            Property::AuthenticationData => { true }
            Property::RequestProblemInformation => { true }
            Property::RequestResponseInformation => { true }
            Property::ReceiveMaximum => { true }
            Property::UserProperty => { true }
            Property::MaximumPacketSize => { true }
            _ => { false }
        }
    }
}

impl Property {
    pub fn handle_property(&self) {
        match self {
            Property::PayloadFormatIndicator => {}
            Property::MessageExpiryInterval => {}
            Property::ContentType => {}
            Property::ResponseTopic => {}
            Property::CorrelationData => {}
            Property::SubscriptionIdentifier => {}
            Property::SessionExpiryInterval => {}
            Property::AssignedClientIdentifier => {}
            Property::ServerKeepAlive => {}
            Property::AuthenticationMethod => {}
            Property::AuthenticationData => {}
            Property::RequestProblemInformation => {}
            Property::WillDelayInterval => {}
            Property::RequestResponseInformation => {}
            Property::ResponseInformation => {}
            Property::ServerReference => {}
            Property::ReasonString => {}
            Property::ReceiveMaximum => {}
            Property::TopicAliasMaximum => {}
            Property::TopicAlias => {}
            Property::MaximumQos => {}
            Property::RetainAvailable => {}
            Property::UserProperty => {}
            Property::MaximumPacketSize => {}
            Property::WildcardSubscriptionAvailable => {}
            Property::SubscriptionIdentifierAvailable => {}
            Property::SharedSubscriptionAvailable => {}
        }
    }
}

pub struct UnPackProperty;

impl UnPackProperty {
    pub fn connect(mut length: u32, data: &[u8]) {
        // let mut properties = vec![];
        // loop {
            let property = data[0];

            match Property::try_from(property) {
                Ok(p) => {
                    if p.is_connect_property() {
                        p.connect_property_handle(length, data);
                    } else {
                        // return Err(format!("Property 0x{:X} not exist",property));
                    }
                }
                Err(e) => {
                    println!("Property {:?} not exist", e)
                }
            }

            println!("{}", property);
        //     if length <= 0 {
        //         break;
        //     }
        // }

        // properties
    }
}
