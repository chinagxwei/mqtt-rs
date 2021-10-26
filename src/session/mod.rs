use std::future::Future;
use tokio::sync::mpsc::Sender;
use crate::message::{BaseMessage, MqttMessageKind};
use crate::subscript::{ClientID, TopicMessage};
use crate::tools::protocol::{MqttDup, MqttProtocolLevel, MqttQos, MqttRetain, MqttWillFlag};
use crate::server::ServerHandleKind;
use async_trait::async_trait;
use crate::message::v3::PublishMessage;
use crate::SUBSCRIPT;

pub mod v3_server_link;
pub mod v3_client_link;

#[async_trait]
pub trait LinkHandle {
    async fn handle<F, Fut>(&mut self, f: F) -> Option<ServerHandleKind>
        where
            F: Fn(Session, Option<MqttMessageKind>) -> Fut + Copy + Clone + Send + Sync + 'static,
            Fut: Future<Output=Option<ServerHandleKind>> + Send;
}

#[derive(Debug)]
pub enum LinkMessage {
    InputMessage(Vec<u8>),
    OutputMessage(Vec<u8>),
    HandleMessage(TopicMessage),
    ExitMessage(bool),
}

#[derive(Clone)]
pub struct Session {
    sender: Sender<LinkMessage>,
    // receiver: Receiver<LinkMessage>,
    client_id: Option<ClientID>,
    protocol_name: Option<String>,
    protocol_level: Option<MqttProtocolLevel>,
    will_flag: Option<MqttWillFlag>,
    will_qos: Option<MqttQos>,
    will_retain: Option<MqttRetain>,
    will_topic: Option<String>,
    will_message: Option<String>,
}

impl Session {
    pub fn new(sender: Sender<LinkMessage>) -> Session {
        Session {
            client_id: None,
            protocol_name: None,
            protocol_level: None,
            will_flag: None,
            will_qos: None,
            will_retain: None,
            will_topic: None,
            will_message: None,
            sender,
            // receiver,
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

    pub fn get_sender(&self) -> Sender<LinkMessage> {
        self.sender.clone()
    }

    pub async fn publish(&self, msg: &PublishMessage) {
        let topic_msg = TopicMessage::ContentV3(self.get_client_id().to_owned(), msg.clone());
        println!("topic: {:?}", topic_msg);
        SUBSCRIPT.broadcast(&msg.topic, &topic_msg).await;
    }

    pub async fn subscribe(&self, topic: &String) {
        println!("{:?}", topic);
        if SUBSCRIPT.contain(topic).await {
            SUBSCRIPT.subscript(topic, self.get_client_id(), self.get_sender());
        } else {
            SUBSCRIPT.new_subscript(topic, self.get_client_id(), self.get_sender()).await;
        }
        println!("broadcast topic len: {}", SUBSCRIPT.len().await);
        println!("broadcast topic list: {:?}", SUBSCRIPT.topics().await);
        println!("broadcast client len: {:?}", SUBSCRIPT.client_len(topic).await);
        println!("broadcast client list: {:?}", SUBSCRIPT.clients(topic).await);
    }

    pub async fn exit(&self) {
        self.sender.send(LinkMessage::ExitMessage(true)).await;
    }
}
