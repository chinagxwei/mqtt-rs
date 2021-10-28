use std::future::Future;
use crate::message::MqttMessageKind;
use crate::subscript::{ClientID, TopicMessage};
use crate::tools::protocol::{MqttCleanSession, MqttProtocolLevel, MqttQos, MqttRetain, MqttWillFlag};
use async_trait::async_trait;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::SendError;
use crate::handle::HandleEvent;
use crate::message::entity::PublishMessage;
use crate::SUBSCRIPT;

#[async_trait]
pub trait MqttSession: Clone {
    async fn publish(&self, msg: &PublishMessage);
    async fn subscribe(&self, topic: &String);
    async fn exit(&self);
    async fn send(&self, msg: Vec<u8>);
}


#[derive(Clone)]
pub struct ClientSessionV3 {
    sender: mpsc::Sender<HandleEvent>,
}

impl ClientSessionV3 {
    pub async fn send_event(&self, event: HandleEvent) -> Result<(), SendError<HandleEvent>> {
        self.sender.send(event).await
    }
}

#[async_trait]
impl MqttSession for ClientSessionV3 {
    async fn publish(&self, msg: &PublishMessage) {}

    async fn subscribe(&self, topic: &String) {}

    async fn exit(&self) {}

    async fn send(&self, msg: Vec<u8>) {}
}

impl ClientSessionV3 {
    pub fn new(sender: mpsc::Sender<HandleEvent>) -> ClientSessionV3 {
        ClientSessionV3 {
            sender
        }
    }
}

#[derive(Clone)]
pub struct ServerSessionV3 {
    sender: mpsc::Sender<HandleEvent>,
    pub(crate) clean_session: Option<MqttCleanSession>,
    client_id: Option<ClientID>,
    protocol_name: Option<String>,
    pub(crate) protocol_level: Option<MqttProtocolLevel>,
    will_flag: Option<MqttWillFlag>,
    will_qos: Option<MqttQos>,
    will_retain: Option<MqttRetain>,
    will_topic: Option<String>,
    will_message: Option<String>,
}

impl ServerSessionV3 {
    pub fn new(sender: mpsc::Sender<HandleEvent>) -> ServerSessionV3 {
        ServerSessionV3 {
            client_id: None,
            protocol_name: None,
            protocol_level: None,
            will_flag: None,
            will_qos: None,
            will_retain: None,
            will_topic: None,
            will_message: None,
            sender,
            clean_session: None,
        }
    }

    pub fn init(&mut self, client_id: ClientID, will_flag: MqttWillFlag, will_qos: MqttQos, will_retain: MqttRetain, will_topic: String, will_message: String) {
        self.client_id = Some(client_id);
        self.will_flag = Some(will_flag);
        self.will_qos = Some(will_qos);
        self.will_retain = Some(will_retain);
        self.will_topic = Some(will_topic);
        self.will_message = Some(will_message);
    }

    pub fn get_client_id(&self) -> &ClientID {
        self.client_id.as_ref().unwrap()
    }

    pub fn init_protocol(&mut self, protocol_name: String, protocol_level: MqttProtocolLevel) {
        self.protocol_name = Some(protocol_name);
        self.protocol_level = Some(protocol_level);
    }

    pub fn is_will_flag(&self) -> bool {
        self.will_flag.unwrap() == MqttWillFlag::Enable
    }

    pub fn get_will_topic(&self) -> &String {
        self.will_topic.as_ref().unwrap()
    }

    pub fn get_will_message(&self) -> Option<TopicMessage> {
        return match self.protocol_level.as_ref().unwrap() {
            MqttProtocolLevel::Level3_1_1 => {
                Some(
                    TopicMessage::generate_v3_topic_message(
                        self.get_client_id().clone(),
                        self.will_qos.unwrap(),
                        self.will_retain.unwrap(),
                        self.will_topic.as_ref().unwrap().to_owned(),
                        self.will_message.as_ref().unwrap().to_owned(),
                    )
                )
            }
            MqttProtocolLevel::Level5 => {
                Some(
                    TopicMessage::generate_v5_topic_message(
                        self.get_client_id().clone(),
                        self.will_qos.unwrap(),
                        self.will_retain.unwrap(),
                        self.will_topic.as_ref().unwrap().to_owned(),
                        self.will_message.as_ref().unwrap().to_owned(),
                    )
                )
            }
            _ => None
        };
    }

    pub async fn send_event(&self, event: HandleEvent) -> Result<(), SendError<HandleEvent>> {
        self.sender.send(event).await
    }
}

#[async_trait]
impl MqttSession for ServerSessionV3 {
    async fn publish(&self, msg: &PublishMessage) {
        let topic_msg = TopicMessage::Content(self.get_client_id().to_owned(), msg.clone());
        println!("topic: {:?}", topic_msg);
        SUBSCRIPT.broadcast(&msg.topic, &topic_msg).await;
    }

    async fn subscribe(&self, topic: &String) {
        println!("{:?}", topic);
        if SUBSCRIPT.contain(topic).await {
            SUBSCRIPT.subscript(topic, self.get_client_id(), self.sender.clone());
        } else {
            SUBSCRIPT.new_subscript(topic, self.get_client_id(), self.sender.clone()).await;
        }
        println!("broadcast topic len: {}", SUBSCRIPT.len().await);
        println!("broadcast topic list: {:?}", SUBSCRIPT.topics().await);
        println!("broadcast client len: {:?}", SUBSCRIPT.client_len(topic).await);
        println!("broadcast client list: {:?}", SUBSCRIPT.clients(topic).await);
    }

    async fn exit(&self) {
        self.sender.send(HandleEvent::ExitEvent(true)).await;
    }

    async fn send(&self, msg: Vec<u8>) {
        self.sender.send(HandleEvent::OutputEvent(msg)).await;
    }
}
