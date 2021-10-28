use std::future::Future;
use async_trait::async_trait;
use crate::executor::ReturnKind;
use crate::message::MqttMessageKind;
use crate::session::MqttSession;
use crate::subscript::TopicMessage;

pub mod v3_server_handle;
pub mod v3_client_handle;

#[derive(Debug)]
pub enum HandleEvent{
    InputEvent(Vec<u8>),
    BroadcastEvent(TopicMessage),
    OutputEvent(Vec<u8>),
    ExitEvent(bool)
}

#[async_trait]
pub trait LinkHandle {
    type Ses: MqttSession;
    async fn execute<F, Fut>(&mut self, f: F) -> Option<ReturnKind>
        where
            F: Fn(Self::Ses, Option<MqttMessageKind>) -> Fut + Copy + Clone + Send + Sync + 'static,
            Fut: Future<Output=()> + Send;
}
