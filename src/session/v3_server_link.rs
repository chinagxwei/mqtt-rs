use crate::message::{PingreqMessage, v3};
use crate::message::v3::MqttMessageV3::*;
use crate::message::v3::{
    ConnackMessage, DisconnectMessage, MqttMessageV3, PubackMessage, ConnectMessage,
    PublishMessage, SubackMessage, SubscribeMessage, UnsubackMessage, UnsubscribeMessage,
};
use crate::message::MqttMessageKind::*;
use crate::message::{
    BaseConnect, BaseMessage, MqttBytesMessage, MqttMessageKind, MqttMessageType,
};
use crate::session::{LinkHandle, LinkMessage, Session};
use crate::subscript::{ClientID, TopicMessage};
use crate::tools::protocol::{MqttDup, MqttProtocolLevel, MqttQos, MqttRetain, MqttWillFlag};
use crate::tools::types::TypeKind;
use crate::SUBSCRIPT;
use std::future::Future;
use std::io::Read;
use std::ops::{Deref, DerefMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::server::ServerHandleKind;
use async_trait::async_trait;

pub struct Link {
    session: Session,
    receiver: Receiver<LinkMessage>,
}

impl Link {
    pub fn new() -> Link {
        let (sender, receiver) = mpsc::channel(512);
        Link {
            session: Session::new(sender),
            receiver,
        }
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub async fn send_message(&self, msg: LinkMessage) {
        self.session().get_sender().send(msg).await;
    }
}

#[async_trait]
impl LinkHandle for Link {
    async fn handle<F, Fut>(&mut self, f: F) -> Option<ServerHandleKind>
        where
            F: Fn(Session, BaseMessage) -> Fut + Copy + Clone + Send + Sync + 'static,
            Fut: Future<Output=Option<ServerHandleKind>> + Send
    {
        return match self.receiver.recv().await {
            Some(msg) => return match msg {
                LinkMessage::InputMessage(msg) => {
                    let base_msg = BaseMessage::from(msg);
                    if base_msg.get_message_type() == TypeKind::CONNECT {
                        let connect = BaseConnect::from(&base_msg);
                        self.session.init_protocol(
                            connect.get_protocol_name(),
                            connect.get_protocol_level(),
                        );
                        let connect_msg = ConnectMessage::from(base_msg);
                        self.session.init(
                            connect_msg.payload.client_id.into(),
                            connect_msg.will_flag,
                            connect_msg.will_qos,
                            connect_msg.will_retain,
                            connect_msg.payload.will_topic.clone().unwrap(),
                            connect_msg.payload.will_message.clone().unwrap(),
                        );
                        return Some(ServerHandleKind::Response(ConnackMessage::default().bytes));
                    }
                    f(self.session.clone(), base_msg).await
                }
                LinkMessage::HandleMessage(msg) => {
                    match msg {
                        TopicMessage::ContentV3(from_id, content) => {
                            let client_id = self.session().get_client_id();
                            println!("from: {:?}", from_id);
                            println!("to: {:?}", client_id);
                            if client_id != &from_id {
                                return Some(ServerHandleKind::Response(
                                    content.as_bytes().to_vec(),
                                ));
                            }
                            None
                        }
                        TopicMessage::WillV3(content) => {
                            Some(ServerHandleKind::Response(content.as_bytes().to_vec()))
                        }
                        _ => None,
                    }
                }
                _ => None
            },
             _=> None
        }
    }
}

