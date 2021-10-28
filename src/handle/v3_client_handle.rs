use std::future::Future;
use tokio::sync::mpsc;
use async_trait::async_trait;
use crate::executor::ReturnKind;
use crate::handle::{HandleEvent, LinkHandle};
use crate::message::{MqttMessageKind, BaseMessage};
use crate::session::ClientSessionV3;

pub struct ClientHandle {
    session: ClientSessionV3,
    receiver: mpsc::Receiver<HandleEvent>,
}

impl ClientHandle {
    pub fn new(session: ClientSessionV3, receiver: mpsc::Receiver<HandleEvent>) -> ClientHandle {
        ClientHandle {
            session,
            receiver,
        }
    }

    pub fn session(&self) -> &ClientSessionV3 {
        &self.session
    }

    pub async fn send_message(&self, msg: HandleEvent) {
        if let Err(e) = self.session.send_event(msg).await {
            println!("failed to send message; err = {:?}", e);
        }
    }
}

#[async_trait]
impl LinkHandle for ClientHandle {
    type Ses = ClientSessionV3;

    async fn execute<F, Fut>(&mut self, f: F) -> Option<ReturnKind>
        where
            F: Fn(Self::Ses, Option<MqttMessageKind>) -> Fut + Copy + Clone + Send + Sync + 'static,
            Fut: Future<Output=()> + Send
    {
        return match self.receiver.recv().await {
            Some(msg) => {
                match msg {
                    HandleEvent::InputEvent(data) => {
                        println!("client input: {:?}", data);
                        let base_msg = BaseMessage::from(data);
                        let v3_request = MqttMessageKind::to_v3_request(base_msg);
                        f(self.session.clone(), v3_request).await;
                        None
                    }
                    HandleEvent::OutputEvent(data) => {
                        Some(ReturnKind::Response(data))
                    }
                    HandleEvent::ExitEvent(_) => {
                        Some(ReturnKind::Exit)
                    }
                    _ => None
                }
            }
            _ => None,
        };
    }
}
