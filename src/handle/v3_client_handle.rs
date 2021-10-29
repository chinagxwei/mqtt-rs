use std::future::Future;
use std::option::Option::Some;
use tokio::sync::mpsc;
use async_trait::async_trait;
use crate::executor::ReturnKind;
use crate::handle::{ClientExecute, HandleEvent, ServerExecute};
use crate::message::{MqttMessageKind, BaseMessage};
use crate::message::v3::MqttMessageV3;
use crate::session::{ClientSessionV3, MqttSession};

pub struct ClientHandleV3 {
    session: ClientSessionV3,
    receiver: mpsc::Receiver<HandleEvent>,
}

impl ClientHandleV3 {
    pub fn new(session: ClientSessionV3, receiver: mpsc::Receiver<HandleEvent>) -> ClientHandleV3 {
        ClientHandleV3 {
            session,
            receiver,
        }
    }

    pub fn session(&self) -> &ClientSessionV3 {
        &self.session
    }

    pub async fn send_message(&self, msg: HandleEvent) {
        self.session.send_event(msg).await
    }
}

#[async_trait]
impl ClientExecute for ClientHandleV3 {
    type Ses = ClientSessionV3;

    async fn execute<F, Fut>(&mut self, f: F, sender: Option<mpsc::Sender<String>>) -> Option<ReturnKind>
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
                        if let Some(send) = sender{
                            if let Some(MqttMessageKind::RequestV3(MqttMessageV3::Publish(msg))) = v3_request.as_ref() {
                                send.send(msg.msg_body.clone());
                            }
                        }
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
