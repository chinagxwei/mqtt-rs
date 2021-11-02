use std::future::Future;
use std::option::Option::Some;
use tokio::sync::mpsc;
use async_trait::async_trait;
use crate::executor::ReturnKind;
use crate::handle::{ClientExecute, HandleEvent};
use crate::message::{MqttMessageKind, BaseMessage};
use crate::message::v3::MqttMessageV3;
use crate::message::v5::MqttMessageV5;
use crate::session::{ClientSession, MqttSession};
use crate::tools::protocol::MqttProtocolLevel;

pub struct ClientHandleV3 {
    session: ClientSession,
    receiver: mpsc::Receiver<HandleEvent>,
}

impl ClientHandleV3 {
    pub fn new(session: ClientSession, receiver: mpsc::Receiver<HandleEvent>) -> ClientHandleV3 {
        ClientHandleV3 {
            session,
            receiver,
        }
    }

    pub fn session(&self) -> &ClientSession {
        &self.session
    }

    pub async fn send_message(&self, msg: HandleEvent) {
        self.session.send_event(msg).await
    }
}

#[async_trait]
impl ClientExecute for ClientHandleV3 {
    type Ses = ClientSession;

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
                        let request = self.request(base_msg);
                        if let Some(send) = sender {
                            if let Some(kind) = request.as_ref() {
                                match kind {
                                    MqttMessageKind::RequestV3(MqttMessageV3::Publish(msg)) => { send.send(msg.msg_body.clone()).await; }
                                    MqttMessageKind::RequestV5(MqttMessageV5::Publish(msg)) => { send.send(msg.msg_body.clone()).await; }
                                    _ => {}
                                }
                            }
                        };
                        f(self.session.clone(), request).await;
                        None
                    }
                    HandleEvent::OutputEvent(data) => {
                        Some(ReturnKind::Response(data.0))
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

impl ClientHandleV3 {
    fn request(&self, base_msg: BaseMessage) -> Option<MqttMessageKind> {
        match self.session.protocol_level {
            MqttProtocolLevel::Level3_1_1 => {
                MqttMessageKind::to_v3_request(base_msg)
            }
            MqttProtocolLevel::Level5 => {
                MqttMessageKind::to_v5_request(base_msg)
            }
            _ => None
        }
    }
}
