use std::future::Future;
use tokio::sync::mpsc;
use async_trait::async_trait;
use crate::handle::{HandleEvent, ServerExecute, ServerHandler};
use crate::message::{MqttMessageKind, BaseMessage, VariableHeader, MqttProtocolLevelInfo};
use crate::session::{MqttSession, ServerSession};
use crate::{SUBSCRIPT, MESSAGE_CONTAINER};
use crate::container::MessageFrame;
use crate::executor::ReturnKind;
use crate::message::entity::PublishMessage;
use crate::message::v3::MqttMessageV3;
use crate::message::v5::MqttMessageV5;
use crate::subscript::TopicMessage::Content;
use crate::tools::protocol::{MqttCleanSession, MqttProtocolLevel, MqttQos};
use crate::tools::types::TypeKind;
use crate::tools::un_pack_tool::get_connect_variable_header;

#[async_trait]
impl ServerExecute for ServerHandler {
    type Ses = ServerSession;
    async fn execute<F, Fut>(&mut self, f: F) -> Option<ReturnKind>
        where
            F: Fn(Self::Ses, Option<MqttMessageKind>) -> Fut + Copy + Clone + Send + Sync + 'static,
            Fut: Future<Output=()> + Send,
    {
        return match self.receiver.recv().await {
            Some(msg) => return match msg {
                HandleEvent::InputEvent(data) => {
                    println!("server input: {:?}", data);
                    let base_msg = BaseMessage::from(data);
                    let type_kind = base_msg.msg_type;
                    if type_kind == TypeKind::CONNECT {
                        let (header, _) = get_connect_variable_header(base_msg.bytes.as_slice());
                        self.init_session_protocol(&header);
                    }
                    let mut request = self.request(base_msg);
                    self.init_session(&request);
                    self.handle_request(&mut request).await;
                    f(self.session.clone(), request).await;
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
                        let bytes = self.publish(content);
                        return Some(ReturnKind::Response(bytes));
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
                HandleEvent::OutputEvent(data) => Some(ReturnKind::Response(data.0))
            },
            _ => None
        };
    }
}

impl ServerHandler {
    fn request(&self, base_msg: BaseMessage) -> Option<MqttMessageKind> {
        if let Some(level) = self.protocol_level() {
            match level {
                MqttProtocolLevel::Level3_1_1 => {
                    MqttMessageKind::to_v3_request(base_msg)
                }
                MqttProtocolLevel::Level5 => {
                    MqttMessageKind::to_v5_request(base_msg)
                }
                _ => None
            }
        } else {
            None
        }
    }

    fn publish(&self, content: PublishMessage) -> Vec<u8> {
        match self.session().protocol_level.unwrap() {
            MqttProtocolLevel::Level3_1_1 => {
                MqttMessageV3::Publish(content).to_vec().unwrap()
            }
            MqttProtocolLevel::Level5 => {
                MqttMessageV5::Publish(content).to_vec().unwrap()
            }
            _ => vec![]
        }
    }
}

#[async_trait]
pub trait HandleSession {
    fn session(&self) -> &ServerSession;

    fn session_mut(&mut self) -> &mut ServerSession;

    fn protocol_level(&self) -> Option<MqttProtocolLevel>;

    fn init_session_protocol(&mut self, header: &VariableHeader) {
        self.session_mut().init_protocol(
            header.protocol_name.clone(),
            header.protocol_level,
        );
    }

    fn init_session(&mut self, request: &Option<MqttMessageKind>) {
        if let Some(kind) = request {
            let connect_msg = match kind {
                MqttMessageKind::RequestV3(MqttMessageV3::Connect(connect_msg)) => Some(connect_msg),
                MqttMessageKind::RequestV5(MqttMessageV5::Connect(connect_msg)) => Some(connect_msg),
                _ => None
            };
            if let Some(connect) = connect_msg {
                self.session_mut().init(
                    connect.payload.client_id.clone().into(),
                    connect.will_flag,
                    connect.will_qos,
                    connect.will_retain,
                    connect.payload.will_topic.clone().unwrap(),
                    connect.payload.will_message.clone().unwrap(),
                );
                MESSAGE_CONTAINER.init(connect.payload.client_id.clone().into());
            }
        }
    }

    async fn handle_request(&self, request: &mut Option<MqttMessageKind>) {
        if let Some(kind) = request {
            match kind {
                MqttMessageKind::RequestV3(v3) => {
                    v3.set_protocol_level(self.protocol_level().unwrap());
                    match v3 {
                        MqttMessageV3::Unsubscribe(msg) => {
                            SUBSCRIPT.unsubscript(&msg.topic, self.session().get_client_id()).await;
                        }
                        MqttMessageV3::Pubrel(msg) => {
                            MESSAGE_CONTAINER.complete(self.session().get_client_id(), msg.message_id).await;
                        }
                        MqttMessageV3::Disconnect(_) => {
                            if self.session().is_will_flag() {
                                if let Some(ref topic_msg) = self.session().get_will_message() {
                                    SUBSCRIPT.broadcast(self.session().get_will_topic(), topic_msg).await;
                                }
                            }
                            SUBSCRIPT.exit(self.session().get_client_id()).await;

                            if self.session().clean_session.is_some() && self.session().clean_session.unwrap() == MqttCleanSession::Enable {
                                MESSAGE_CONTAINER.remove(self.session().get_client_id()).await;
                            }
                        }
                        _ => {}
                    }
                }
                MqttMessageKind::RequestV5(v5) => {
                    v5.set_protocol_level(self.protocol_level().unwrap());
                    match v5 {
                        MqttMessageV5::Unsubscribe(msg) => {
                            SUBSCRIPT.unsubscript(&msg.topic, self.session().get_client_id()).await;
                        }
                        MqttMessageV5::Pubrel(msg) => {
                            MESSAGE_CONTAINER.complete(self.session().get_client_id(), msg.message_id).await;
                        }
                        MqttMessageV5::Disconnect(_) => {
                            if self.session().is_will_flag() {
                                if let Some(ref topic_msg) = self.session().get_will_message() {
                                    SUBSCRIPT.broadcast(self.session().get_will_topic(), topic_msg).await;
                                }
                            }
                            SUBSCRIPT.exit(self.session().get_client_id()).await;

                            if self.session().clean_session.is_some() && self.session().clean_session.unwrap() == MqttCleanSession::Enable {
                                MESSAGE_CONTAINER.remove(self.session().get_client_id()).await;
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }

        }
    }
}

#[async_trait]
impl HandleSession for ServerHandler {
    fn session(&self) -> &ServerSession {
        &self.session
    }

    fn session_mut(&mut self) -> &mut ServerSession {
        &mut self.session
    }

    fn protocol_level(&self) -> Option<MqttProtocolLevel> {
        self.session.protocol_level
    }
}
