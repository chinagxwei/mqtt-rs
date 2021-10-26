use crate::message::v3;
use crate::message::v3::MqttMessageV3::*;
use crate::message::v3::{
    ConnackMessage, DisconnectMessage, MqttMessageV3, PubackMessage, ConnectMessage,
    PublishMessage, SubackMessage, SubscribeMessage, UnsubackMessage, UnsubscribeMessage,
};
use crate::message::MqttMessageKind::*;
use crate::message::{
    BaseMessage, MqttBytesMessage, MqttMessageKind, MqttMessageType,
};
use crate::session::{LinkHandle, LinkMessage, Session};
use crate::subscript::{ClientID, TopicMessage};
use crate::tools::protocol::{MqttCleanSession, MqttDup, MqttProtocolLevel, MqttQos, MqttRetain, MqttWillFlag};
use crate::tools::types::TypeKind;
use crate::{SUBSCRIPT, MESSAGE_CONTAINER};
use std::future::Future;
use std::io::Read;
use std::ops::{Deref, DerefMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::server::ServerHandleKind;
use async_trait::async_trait;
use crate::container::MessageFrame;

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
        // self.session.get_sender().send(msg).await;
        if let Err(e) = self.session.sender.send(msg).await {
            println!("failed to send message; err = {:?}", e);
        }
    }
}

#[async_trait]
impl LinkHandle for Link {
    async fn handle<F, Fut>(&mut self, f: F) -> Option<ServerHandleKind>
        where
            F: Fn(Session, Option<MqttMessageKind>) -> Fut + Copy + Clone + Send + Sync + 'static,
            Fut: Future<Output=Option<ServerHandleKind>> + Send
    {
        return match self.receiver.recv().await {
            Some(msg) => return match msg {
                LinkMessage::InputMessage(msg) => {
                    let base_msg = BaseMessage::from(msg);
                    let v3_request = handle_v3_request(base_msg, &mut self.session).await;
                    f(self.session.clone(), v3_request).await
                }
                LinkMessage::HandleMessage(msg) => {
                    match msg {
                        TopicMessage::ContentV3(from_id, content) => {
                            let client_id = self.session().get_client_id();
                            println!("from: {:?}", from_id);
                            println!("to: {:?}", client_id);
                            if content.qos == MqttQos::Qos2 {
                                MESSAGE_CONTAINER.append(
                                    client_id.clone(),
                                    content.message_id,
                                    MessageFrame::new(
                                        from_id.clone(),
                                        client_id.clone(),
                                        content.bytes.as_ref().unwrap().clone(),
                                        content.message_id,
                                    ),
                                ).await;
                            }
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
                LinkMessage::ExitMessage(will) => {
                    if will && self.session.is_will_flag() {
                        if let Some(ref topic_msg) = self.session.get_will_message() {
                            SUBSCRIPT.broadcast(self.session.get_will_topic(), topic_msg).await;
                        }
                    }
                    SUBSCRIPT.exit(self.session.get_client_id()).await;
                    Some(ServerHandleKind::Exit)
                }
                _ => None
            },
            _ => None
        };
    }
}

async fn handle_v3_request(base_msg: BaseMessage, session: &mut Session) -> Option<MqttMessageKind> {
    let mut v3_request = MqttMessageKind::v3(base_msg);
    if let Some(kind) = v3_request.as_mut() {
        match kind {
            RequestV3(v3) => {
                match v3 {
                    Connect(connect_msg) => {
                        println!("{:?}", connect_msg);
                        session.init_protocol(
                            connect_msg.protocol_name.clone(),
                            connect_msg.protocol_level,
                        );
                        session.init(
                            connect_msg.payload.client_id.clone().into(),
                            connect_msg.will_flag,
                            connect_msg.will_qos,
                            connect_msg.will_retain,
                            connect_msg.payload.will_topic.clone().unwrap(),
                            connect_msg.payload.will_message.clone().unwrap(),
                        );
                    }
                    Unsubscribe(msg) => {
                        if SUBSCRIPT.contain(&msg.topic).await {
                            if SUBSCRIPT.is_subscript(&msg.topic, session.get_client_id()).await {
                                SUBSCRIPT.unsubscript(&msg.topic, session.get_client_id()).await;
                            }
                        }
                        msg.protocol_level = session.protocol_level;
                    }
                    Pubrel(msg) => {
                        MESSAGE_CONTAINER.complete(session.get_client_id(), msg.message_id).await;
                        msg.protocol_level = session.protocol_level;
                    }
                    Disconnect(msg) => {
                        if session.is_will_flag() {
                            if let Some(ref topic_msg) = session.get_will_message() {
                                SUBSCRIPT.broadcast(session.get_will_topic(), topic_msg).await;
                            }
                        }
                        SUBSCRIPT.exit(session.get_client_id()).await;

                        if session.clean_session.unwrap() == MqttCleanSession::Enable {
                            MESSAGE_CONTAINER.remove(session.get_client_id());
                        }
                        msg.protocol_level = session.protocol_level;
                    }
                    Connack(msg) => msg.protocol_level = session.protocol_level,
                    Publish(msg) => msg.protocol_level = session.protocol_level,
                    Puback(msg) => msg.protocol_level = session.protocol_level,
                    Pubrec(msg) => msg.protocol_level = session.protocol_level,
                    Pubcomp(msg) => msg.protocol_level = session.protocol_level,
                    Subscribe(msg) => msg.protocol_level = session.protocol_level,
                    Suback(msg) => msg.protocol_level = session.protocol_level,
                    Unsuback(msg) => msg.protocol_level = session.protocol_level,
                    _=>{}
                }
            }
            _ => {}
        }
    }
    v3_request
}

