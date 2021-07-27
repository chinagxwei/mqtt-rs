use num_enum::TryFromPrimitive;
use std::convert::TryFrom;

#[derive(Debug, Copy, Clone, TryFromPrimitive)]
#[repr(u32)]
pub enum QosReasonPhrases {
    GrantedQos0 = 0x00,
    GrantedQos1 = 0x01,
    GrantedQos2 = 0x02,
}

impl QosReasonPhrases {
    pub fn as_str(&self) -> &'static str {
        match *self {
            QosReasonPhrases::GrantedQos0 => { "Granted QoS 0" }
            QosReasonPhrases::GrantedQos1 => { "Granted QoS 1" }
            QosReasonPhrases::GrantedQos2 => { "Granted QoS 2" }
        }
    }
}

#[derive(Debug, Copy, Clone, TryFromPrimitive)]
#[repr(u8)]
pub enum ReasonCodeV3 {
    ConnectionAccepted = 0x00,
    UnacceptableProtocolVersion = 0x01,
    IdentifierRejected = 0x02,
    ServerUnavailable = 0x03,
    BadUsernameOrPassword = 0x04,
    NotAuthorized = 0x05,
}

impl ReasonCodeV3 {
    pub fn as_str(&self) -> &'static str {
        match *self {
            ReasonCodeV3::ConnectionAccepted => { "Connection accepted" }
            ReasonCodeV3::UnacceptableProtocolVersion => { "The Server does not support the level of the MQTT protocol requested by the Client" }
            ReasonCodeV3::IdentifierRejected => { "The Client identifier is correct UTF-8 but not allowed by the Server" }
            ReasonCodeV3::ServerUnavailable => { "The Network Connection has been made but the MQTT service is unavailable" }
            ReasonCodeV3::BadUsernameOrPassword => { "The data in the user name or password is malformed" }
            ReasonCodeV3::NotAuthorized => { "The Client is not authorized to connect" }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ReasonCodes {
    V3(ReasonCodeV3),
    V5(ReasonCodeV5),
}

impl ReasonCodes {
    pub fn is_v3(&self) -> bool {
        matches!(self, ReasonCodes::V3(_))
    }

    pub fn is_v5(&self) -> bool {
        matches!(self, ReasonCodes::V5(_))
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ReasonCodes::V3(code) => { code.as_str() }
            ReasonCodes::V5(code) => { "not found" }
        }
    }

    pub fn as_byte(&self) -> u8 {
        match self {
            ReasonCodes::V3(code) => { *code as u8 }
            ReasonCodes::V5(code) => { b'n' }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ReasonCodeV5 {
    General
}

#[derive(Debug, Copy, Clone, TryFromPrimitive)]
#[repr(u32)]
pub enum ReasonPhrases {
    Success = 0x00,
    DisconnectWithWillMessage = 0x04,
    NoMatchingSubscribers = 0x10,
    NoSubscriptionExisted = 0x11,
    ContinueAuthentication = 0x18,
    ReAuthenticate = 0x19,
    UnspecifiedError = 0x80,
    MalformedPacket = 0x81,
    ProtocolError = 0x82,
    ImplementationSpecificError = 0x83,
    UnsupportedProtocolVersion = 0x84,
    ClientIdentifierNotValid = 0x85,
    BadUserNameOrPassword = 0x86,
    NotAuthorized = 0x87,
    ServerUnavailable = 0x88,
    ServerBusy = 0x89,
    Banned = 0x8A,
    ServerShuttingDown = 0x8B,
    BadAuthenticationMethod = 0x8C,
    KeepAliveTimeout = 0x8D,
    SessionTakenOver = 0x8E,
    TopicFilterInvalid = 0x8F,
    TopicNameInvalid = 0x90,
    PacketIdentifierInUse = 0x91,
    PacketIdentifierNotFound = 0x92,
    ReceiveMaximumExceeded = 0x93,
    TopicAliasInvalid = 0x94,
    PacketTooLarge = 0x95,
    MessageRateTooHigh = 0x96,
    QuotaExceeded = 0x97,
    AdministrativeAction = 0x98,
    PayloadFormatInvalid = 0x99,
    RetainNotSupported = 0x9A,
    QosNotSupported = 0x9B,
    UseAnotherServer = 0x9C,
    ServerMoved = 0x9D,
    SharedSubscriptionsNotSupported = 0x9E,
    ConnectionRateExceeded = 0x9F,
    MaximumConnectTime = 0xA0,
    SubscriptionIdentifiersNotSupported = 0xA1,
    WildcardSubscriptionsNotSupported = 0xA2,
}

impl ReasonPhrases {
    pub fn as_str(&self) -> &'static str {
        match *self {
            ReasonPhrases::Success => { "Success" }
            ReasonPhrases::DisconnectWithWillMessage => { "Disconnect with will mqtt.message" }
            ReasonPhrases::NoMatchingSubscribers => { "No matching subscribers" }
            ReasonPhrases::NoSubscriptionExisted => { "No subscription existed" }
            ReasonPhrases::ContinueAuthentication => { "continue authentication" }
            ReasonPhrases::ReAuthenticate => { "Re authenticate" }
            ReasonPhrases::UnspecifiedError => { "Unspecified error" }
            ReasonPhrases::MalformedPacket => { "Malformed packet" }
            ReasonPhrases::ProtocolError => { "Protocol error" }
            ReasonPhrases::ImplementationSpecificError => { "Implementation specific error" }
            ReasonPhrases::UnsupportedProtocolVersion => { "Unsupported protocol version" }
            ReasonPhrases::ClientIdentifierNotValid => { "ClientIdentifier not valid" }
            ReasonPhrases::BadUserNameOrPassword => { "Bad user name or password" }
            ReasonPhrases::NotAuthorized => { "Not authorized" }
            ReasonPhrases::ServerUnavailable => { "Server unavailable" }
            ReasonPhrases::ServerBusy => { "Server busy" }
            ReasonPhrases::Banned => { "Banned" }
            ReasonPhrases::ServerShuttingDown => { "Server shutting down" }
            ReasonPhrases::BadAuthenticationMethod => { "Bad authentication method" }
            ReasonPhrases::KeepAliveTimeout => { "Keep alive timeout" }
            ReasonPhrases::SessionTakenOver => { "Session taken over" }
            ReasonPhrases::TopicFilterInvalid => { "Topic filter invalid" }
            ReasonPhrases::TopicNameInvalid => { "Topic name invalid" }
            ReasonPhrases::PacketIdentifierInUse => { "Packet identifierInUse" }
            ReasonPhrases::PacketIdentifierNotFound => { "Packet identifier not found" }
            ReasonPhrases::ReceiveMaximumExceeded => { "Receive maximum exceeded" }
            ReasonPhrases::TopicAliasInvalid => { "Topic alias invalid" }
            ReasonPhrases::PacketTooLarge => { "Packet too large" }
            ReasonPhrases::MessageRateTooHigh => { "Message rate too high" }
            ReasonPhrases::QuotaExceeded => { "Quota exceeded" }
            ReasonPhrases::AdministrativeAction => { "Administrative action" }
            ReasonPhrases::PayloadFormatInvalid => { "Payload format invalid" }
            ReasonPhrases::RetainNotSupported => { "Retain not supported" }
            ReasonPhrases::QosNotSupported => { "Qos not supported" }
            ReasonPhrases::UseAnotherServer => { "Use another server" }
            ReasonPhrases::ServerMoved => { "Server moved" }
            ReasonPhrases::SharedSubscriptionsNotSupported => { "Shared subscriptions not supported" }
            ReasonPhrases::ConnectionRateExceeded => { "Connection rate exceeded" }
            ReasonPhrases::MaximumConnectTime => { "Maximum connect time" }
            ReasonPhrases::SubscriptionIdentifiersNotSupported => { "Subscription identifiers not supported" }
            ReasonPhrases::WildcardSubscriptionsNotSupported => { "Wildcard subscriptions not supported" }
        }
    }
}

