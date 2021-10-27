use crate::message::v3::MqttMessageV3::*;
use crate::message::MqttMessageKind::*;
use crate::message::{
    BaseMessage, MqttMessageKind};
use crate::session::{LinkHandle, LinkMessage, MqttSession, ServerSessionV3};
use crate::subscript::TopicMessage;
use crate::tools::protocol::{MqttCleanSession, MqttQos};
use crate::{SUBSCRIPT, MESSAGE_CONTAINER};
use std::future::Future;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use crate::server::ServerHandleKind;
use async_trait::async_trait;
use crate::container::MessageFrame;
use crate::message::v3::MqttMessageV3;

pub struct Link {
    session: ServerSessionV3,
    receiver: Receiver<LinkMessage>,
}

impl Link {
    pub fn new() -> Link {
        let (sender, receiver) = mpsc::channel(512);
        Link {
            session: ServerSessionV3::new(sender),
            receiver,
        }
    }

    pub fn session(&self) -> &ServerSessionV3 {
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
    type Ses = ServerSessionV3;

    async fn handle<F, Fut>(&mut self, f: F) -> Option<ServerHandleKind>
        where
            F: Fn(Self::Ses, Option<MqttMessageKind>) -> Fut + Copy + Clone + Send + Sync + 'static,
            Fut: Future<Output=()> + Send,
    {
        return match self.receiver.recv().await {
            Some(msg) => return match msg {
                LinkMessage::InputMessage(msg) => {
                    let base_msg = BaseMessage::from(msg);
                    let mut v3_request = MqttMessageKind::to_v3_request(base_msg);
                    self.init_session(&v3_request);
                    self.handle_v3_request(&mut v3_request).await;
                    f(self.session.clone(), v3_request).await;
                    None
                }
                LinkMessage::HandleMessage(msg) => {
                    match msg {
                        TopicMessage::Content(from_id, content) => {
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
                                    MqttMessageV3::Publish(content).to_vec().unwrap()
                                ));
                            }
                            None
                        }
                        // TopicMessage::Will(content) => {
                        //     Some(ServerHandleKind::Response(MqttMessageV3::Publish(content).to_vec().unwrap()))
                        // }
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
                LinkMessage::OutputMessage(data) => Some(ServerHandleKind::Response(data))
            },
            _ => None
        };
    }
}

impl Link {
    fn init_session(&mut self, v3_request: &Option<MqttMessageKind>) {
        if let Some(kind) = v3_request {
            match kind {
                RequestV3(v3) => {
                    match v3 {
                        Connect(connect_msg) => {
                            println!("{:?}", connect_msg);
                            self.session.init_protocol(
                                connect_msg.protocol_name.clone(),
                                connect_msg.protocol_level,
                            );
                            self.session.init(
                                connect_msg.payload.client_id.clone().into(),
                                connect_msg.will_flag,
                                connect_msg.will_qos,
                                connect_msg.will_retain,
                                connect_msg.payload.will_topic.clone().unwrap(),
                                connect_msg.payload.will_message.clone().unwrap(),
                            );
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    async fn handle_v3_request(&self, v3_request: &mut Option<MqttMessageKind>) {
        if let Some(kind) = v3_request {
            match kind {
                RequestV3(v3) => {
                    match v3 {
                        Unsubscribe(msg) => {
                            if SUBSCRIPT.contain(&msg.topic).await {
                                if SUBSCRIPT.is_subscript(&msg.topic, self.session.get_client_id()).await {
                                    SUBSCRIPT.unsubscript(&msg.topic, self.session.get_client_id()).await;
                                }
                            }
                            msg.protocol_level = self.session.protocol_level.clone();
                        }
                        Pubrel(msg) => {
                            MESSAGE_CONTAINER.complete(self.session.get_client_id(), msg.message_id).await;
                            msg.protocol_level = self.session.protocol_level;
                        }
                        Disconnect(msg) => {
                            if self.session.is_will_flag() {
                                if let Some(ref topic_msg) = self.session.get_will_message() {
                                    SUBSCRIPT.broadcast(self.session.get_will_topic(), topic_msg).await;
                                }
                            }
                            SUBSCRIPT.exit(self.session.get_client_id()).await;

                            if self.session.clean_session.is_some() && self.session.clean_session.unwrap() == MqttCleanSession::Enable {
                                MESSAGE_CONTAINER.remove(self.session.get_client_id()).await;
                            }
                            msg.protocol_level = self.session.protocol_level;
                        }
                        Connack(msg) => msg.protocol_level = self.session.protocol_level,
                        Publish(msg) => msg.protocol_level = self.session.protocol_level,
                        Puback(msg) => msg.protocol_level = self.session.protocol_level,
                        Pubrec(msg) => msg.protocol_level = self.session.protocol_level,
                        Pubcomp(msg) => msg.protocol_level = self.session.protocol_level,
                        Subscribe(msg) => msg.protocol_level = self.session.protocol_level,
                        Suback(msg) => msg.protocol_level = self.session.protocol_level,
                        Unsuback(msg) => msg.protocol_level = self.session.protocol_level,
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}



