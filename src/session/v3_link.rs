use crate::message::v3;
use crate::message::v3::MqttMessageV3::*;
use crate::message::v3::{
    ConnackMessage, DisconnectMessage, MqttMessageV3, PubackMessage, ConnectMessage,
    PublishMessage, SubackMessage, SubscribeMessage, UnsubackMessage, UnsubscribeMessage,
};
use crate::message::MqttMessageKind::*;
use crate::message::{
    BaseConnect, BaseMessage, MqttBytesMessage, MqttMessageKind, MqttMessageType,
};
use crate::session::{LinkMessage, Session};
use crate::subscript::{ClientID, TopicMessage};
use crate::tools::protocol::{MqttDup, MqttProtocolLevel, MqttQos, MqttRetain, MqttWillFlag};
use crate::tools::types::TypeKind;
use crate::SUBSCRIPT;
use async_trait::async_trait;
use std::future::Future;
use std::io::Read;
use std::ops::{Deref, DerefMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::server::ServerHandleKind;

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

impl Link {
    pub async fn handle<F, Fut>(&mut self, f: F) -> Option<ServerHandleKind>
        where
            F: Fn(Session, BaseMessage) -> Fut + Copy + Clone + Send + Sync + 'static,
            Fut: Future<Output=Option<ServerHandleKind>> + Send
    {
        match self.receiver.recv().await {
            None => {}
            Some(msg) => match msg {
                LinkMessage::SocketMessage(msg) => {
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
                            connect_msg.payload.will_message.clone().unwrap()
                        );
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
                    };
                }
            },
        }
        None
    }
}

