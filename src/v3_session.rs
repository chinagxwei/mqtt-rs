use std::future::Future;
use std::io::Read;
use std::ops::{Deref, DerefMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::message::v3;
use crate::message::v3::{SubackMessage, ConnackMessage, ConnectMessage, DisconnectMessage, MqttMessageV3, PubackMessage, PublishMessage, SubscribeMessage, UnsubackMessage, UnsubscribeMessage};
use crate::subscript::{ClientID, TopicMessage};
use crate::tools::protocol::{MqttDup, MqttProtocolLevel, MqttQos, MqttRetain, MqttWillFlag};
use crate::tools::types::TypeKind;
use crate::message::{MqttMessageKind, MqttBytesMessage, BaseMessage, MqttMessage, BaseConnect};
use crate::message::MqttMessageKind::*;
use crate::message::v3::MqttMessageV3::*;
use crate::server::ServerHandleKind;
use crate::SUBSCRIPT;

pub enum LinkMessage {
    SocketMessage(Vec<u8>),
    HandleMessage(TopicMessage),
}

pub struct Link {
    session: Session,
    receiver: Receiver<LinkMessage>,
}

impl Link {
    pub fn new() -> Link {
        let (sender, receiver) = mpsc::channel(512);
        Link { session: Session::new(sender), receiver }
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub async fn send_message(&self, msg: LinkMessage) {
        self.session().sender.send(msg).await;
    }

    pub async fn handle<F, Fut>(&mut self, f: F) -> Option<ServerHandleKind>
        where
            F: Fn(Session, BaseMessage) -> Fut + Send + Sync + 'static,
            Fut: Future<Output=Option<ServerHandleKind>> + Send
    {
        match self.receiver.recv().await {
            None => {}
            Some(msg) => {
                match msg {
                    LinkMessage::SocketMessage(msg) => {
                        let base_msg = BaseMessage::from(msg);
                        if base_msg.get_message_type() == TypeKind::CONNECT {
                            let connect = BaseConnect::from(&base_msg);
                            self.session.init_protocol(connect.get_protocol_name(), connect.get_protocol_level());
                            let connect_msg = ConnectMessage::from(base_msg);
                            self.session.init_v3(&connect_msg);
                            return Some(ServerHandleKind::Response(ConnackMessage::default().bytes));
                        }
                        return f(self.session.clone(), base_msg).await;
                    }
                    LinkMessage::HandleMessage(msg) => {
                        return match msg {
                            TopicMessage::ContentV3(from_id, content) => {
                                let client_id = self.session().get_client_id();
                                println!("from: {:?}", from_id);
                                println!("to: {:?}", client_id);
                                if client_id != &from_id {
                                    return Some(ServerHandleKind::Response(content.as_bytes().to_vec()));
                                }
                                None
                            }
                            TopicMessage::Will(content) => {
                                Some(ServerHandleKind::Response(content.as_bytes().to_vec()))
                            }
                            _ => None
                        };
                    }
                }
            }
        }
        None
    }
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

    pub fn get_will_message(&self) -> TopicMessage {
        let msg = v3::PublishMessage::new(
            self.will_qos.unwrap(),
            MqttDup::Disable,
            self.will_retain.unwrap(),
            self.will_topic.as_ref().unwrap().to_owned(),
            0,
            self.will_message.as_ref().unwrap().to_owned(),
        );
        TopicMessage::ContentV3(self.get_client_id().clone(), msg)
    }

    pub fn get_will_topic(&self) -> &String {
        self.will_topic.as_ref().unwrap()
    }

    pub fn init_v3(&mut self, connect_msg: &ConnectMessage) {
        self.client_id = Some(ClientID(connect_msg.payload.client_id.to_owned()));
        self.will_flag = Some(connect_msg.will_flag);
        self.will_qos = Some(connect_msg.will_qos);
        self.will_retain = Some(connect_msg.will_retain);
        self.will_topic = connect_msg.payload.will_topic.clone();
        self.will_message = connect_msg.payload.will_message.clone();
    }

    pub fn get_sender(&self) -> Sender<LinkMessage> {
        self.sender.clone()
    }
}
