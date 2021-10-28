use std::future::Future;
use tokio::sync::mpsc;
use async_trait::async_trait;
use crate::handle::{HandleEvent, LinkHandle};
use crate::message::{MqttMessageKind, BaseMessage};
use crate::session::ServerSessionV3;
use crate::{SUBSCRIPT, MESSAGE_CONTAINER};
use crate::container::MessageFrame;
use crate::executor::ReturnKind;
use crate::message::MqttMessageKind::RequestV3;
use crate::message::v3::MqttMessageV3;
use crate::message::v3::MqttMessageV3::*;
use crate::subscript::TopicMessage::Content;
use crate::tools::protocol::{MqttCleanSession, MqttQos};

pub struct ServerHandler {
    session: ServerSessionV3,
    receiver: mpsc::Receiver<HandleEvent>,
}

impl ServerHandler {
    pub fn new() -> ServerHandler {
        let (sender, receiver) = mpsc::channel(512);
        ServerHandler {
            session: ServerSessionV3::new(sender),
            receiver,
        }
    }

    pub fn session(&self) -> &ServerSessionV3 {
        &self.session
    }

    pub async fn send_message(&self, msg: HandleEvent) {
        if let Err(e) = self.session.send_event(msg).await {
            println!("failed to send message; err = {:?}", e);
        }
    }
}

#[async_trait]
impl LinkHandle for ServerHandler {
    type Ses = ServerSessionV3;

    async fn execute<F, Fut>(&mut self, f: F) -> Option<ReturnKind>
        where
            F: Fn(Self::Ses, Option<MqttMessageKind>) -> Fut + Copy + Clone + Send + Sync + 'static,
            Fut: Future<Output=()> + Send,
    {
        return match self.receiver.recv().await {
            Some(msg) => return match msg {
                HandleEvent::InputEvent(msg) => {
                    let base_msg = BaseMessage::from(msg);
                    let mut v3_request = MqttMessageKind::to_v3_request(base_msg);
                    self.init_session(&v3_request);
                    self.handle_v3_request(&mut v3_request).await;
                    f(self.session.clone(), v3_request).await;
                    None
                }
                HandleEvent::BroadcastEvent(Content(from_id, content)) => {
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
                        return Some(ReturnKind::Response(
                            MqttMessageV3::Publish(content).to_vec().unwrap()
                        ));
                    }
                    None
                }
                HandleEvent::ExitEvent(will) => {
                    if will && self.session.is_will_flag() {
                        if let Some(ref topic_msg) = self.session.get_will_message() {
                            SUBSCRIPT.broadcast(self.session.get_will_topic(), topic_msg).await;
                        }
                    }
                    SUBSCRIPT.exit(self.session.get_client_id()).await;
                    Some(ReturnKind::Exit)
                }
                HandleEvent::OutputEvent(data) => Some(ReturnKind::Response(data))
            },
            _ => None
        };
    }
}

impl ServerHandler {
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
                            MESSAGE_CONTAINER.init(connect_msg.payload.client_id.clone().into());
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
