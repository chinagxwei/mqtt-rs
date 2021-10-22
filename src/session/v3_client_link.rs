use std::future::Future;
use async_trait::async_trait;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use crate::message::{BaseMessage, PingreqMessage};
use crate::message::v3::DisconnectMessage;
use crate::server::ServerHandleKind;
use crate::session::{LinkHandle, LinkMessage, Session};

pub struct Link {
    session: Session,
    receiver: Receiver<LinkMessage>,
}

impl Link {
    pub fn new(session: Session, receiver: Receiver<LinkMessage>) -> Link {
        Link {
            session,
            receiver,
        }
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub async fn send_message(&self, msg: LinkMessage) {
        self.session().sender.send(msg).await;
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
            Some(msg) => {
                match msg {
                    LinkMessage::InputMessage(data) => {
                        let base_msg = BaseMessage::from(data);
                        f(self.session.clone(), base_msg).await
                    }
                    LinkMessage::OutputMessage(data) => {
                        Some(ServerHandleKind::Response(data))
                    },
                    LinkMessage::ExitMessage(data) => {
                        Some(ServerHandleKind::Exit(data))
                    }
                    _ => None
                }
            }
            _ => None,
        };

    }
}
