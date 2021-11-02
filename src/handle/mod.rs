use std::future::Future;
use async_trait::async_trait;
use tokio::sync::mpsc;
use crate::executor::ReturnKind;
use crate::message::{MqttMessageKind, VariableHeader};
use crate::session::{MqttSession, ServerSession};
use crate::subscript::TopicMessage;
use crate::tools::protocol::MqttProtocolLevel;
pub mod server_handle;
pub mod v3_client_handle;

#[derive(Debug)]
pub struct Response(pub Vec<u8>, pub MqttProtocolLevel);

#[derive(Debug)]
pub enum HandleEvent {
    InputEvent(Vec<u8>),
    BroadcastEvent(TopicMessage),
    OutputEvent(Response),
    ExitEvent(bool),
}

#[async_trait]
pub trait ServerExecute {
    type Ses: MqttSession;
    async fn execute<F, Fut>(&mut self, f: F) -> Option<ReturnKind>
        where
            F: Fn(Self::Ses, Option<MqttMessageKind>) -> Fut + Copy + Clone + Send + Sync + 'static,
            Fut: Future<Output=()> + Send;
}

#[async_trait]
pub trait ClientExecute {
    type Ses: MqttSession;
    async fn execute<F, Fut>(&mut self, f: F, sender: Option<mpsc::Sender<String>>) -> Option<ReturnKind>
        where
            F: Fn(Self::Ses, Option<MqttMessageKind>) -> Fut + Copy + Clone + Send + Sync + 'static,
            Fut: Future<Output=()> + Send;
}

pub struct ServerHandler {
    session: ServerSession,
    receiver: mpsc::Receiver<HandleEvent>,
}

impl ServerHandler {
    pub fn new() -> ServerHandler {
        let (sender, receiver) = mpsc::channel(512);
        ServerHandler {
            session: ServerSession::new(sender),
            receiver,
        }
    }

    pub async fn send_message(&self, msg: HandleEvent) {
        self.session.send_event(msg).await;
    }
}

