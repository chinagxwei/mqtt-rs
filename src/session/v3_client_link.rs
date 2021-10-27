use std::future::Future;
use async_trait::async_trait;
use tokio::sync::mpsc::Receiver;
use crate::message::{BaseMessage, MqttMessageKind};
use crate::server::ServerHandleKind;
use crate::session::{LinkHandle, LinkMessage, MqttSession, ClientSessionV3};

pub struct Link {
    session: ClientSessionV3,
    receiver: Receiver<LinkMessage>,
}

impl Link {
    pub fn new(session: ClientSessionV3, receiver: Receiver<LinkMessage>) -> Link {
        Link {
            session,
            receiver,
        }
    }

    pub fn session(&self) -> &ClientSessionV3 {
        &self.session
    }

    pub async fn send_message(&self, msg: LinkMessage) {
        // self.session.sender.send(msg).await;
        if let Err(e) = self.session.sender.send(msg).await {
            println!("failed to send message; err = {:?}", e);
        }
    }
}

#[async_trait]
impl LinkHandle for Link {
    type Ses = ClientSessionV3;

    async fn handle<F, Fut>(&mut self, f: F) -> Option<ServerHandleKind>
        where
            F: Fn(Self::Ses, Option<MqttMessageKind>) -> Fut + Copy + Clone + Send + Sync + 'static,
            Fut: Future<Output=()> + Send
    {
        return match self.receiver.recv().await {
            Some(msg) => {
                match msg {
                    LinkMessage::InputMessage(data) => {
                        let base_msg = BaseMessage::from(data);
                        let v3_request = MqttMessageKind::to_v3_request(base_msg);
                        f(self.session.clone(), v3_request).await;
                        None
                    }
                    LinkMessage::OutputMessage(data) => {
                        Some(ServerHandleKind::Response(data))
                    }
                    LinkMessage::ExitMessage(_) => {
                        Some(ServerHandleKind::Exit)
                    }
                    _ => None
                }
            }
            _ => None,
        };
    }
}
