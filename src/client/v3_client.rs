use std::future::Future;
use std::net::SocketAddr;
use std::option::Option::Some;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpSocket, TcpStream};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use crate::message::{BaseMessage, MqttBytesMessage, MqttMessageKind, PingreqMessage};
use crate::message::v3::{ConnectMessage, DisconnectMessage, PublishMessage, SubscribeMessage, UnsubscribeMessage};
use crate::server::ServerHandleKind;
use crate::session::{LinkHandle, LinkMessage, Session};
use crate::session::v3_client_link::Link;

use crate::tools::config::Config;
use crate::tools::protocol::{MqttCleanSession, MqttQos};

struct MqttClient<F, Fut>
    where
        F: Fn(Session, Option<MqttMessageKind>) -> Fut + Copy + Clone + Send + Sync + 'static,
        Fut: Future<Output=Option<ServerHandleKind>> + Send,
{
    config: Config,
    address: SocketAddr,
    handle: Option<Box<F>>,
    sender: Option<Sender<LinkMessage>>,
}

impl<F, Fut> MqttClient<F, Fut>
    where
        F: Fn(Session, Option<MqttMessageKind>) -> Fut + Copy + Clone + Send + Sync + 'static,
        Fut: Future<Output=Option<ServerHandleKind>> + Send,
{
    pub fn new(config: Config, address: SocketAddr) -> MqttClient<F, Fut> {
        MqttClient { config, address, handle: None, sender: None }
    }

    pub fn handle(mut self, f: F) -> MqttClient<F, Fut> {
        self.handle = Some(Box::new(f));
        self
    }

    // pub fn publish(&self, topic: String, message: String, qos: MqttQos) {
    //     PublishMessage::new()
    // }
    //
    // pub async fn subscribe(&self, topic: String) {
    //     SubscribeMessage::new()
    // }
    //
    // pub fn unsubscribe(&self, topic: String) {
    //     UnsubscribeMessage::new()
    // }

    pub async fn disconnect(&self) {
        if let Some(sender) = self.sender.as_ref() {
            sender.send(LinkMessage::OutputMessage(DisconnectMessage::default().bytes)).await;
        }
    }

    async fn init(&self) -> TcpStream {
        let socket = TcpSocket::new_v4().unwrap();
        let mut stream = socket.connect(self.address).await.unwrap();
        let msg = ConnectMessage::new(MqttCleanSession::Enable, self.config.clone());
        stream.write_all(msg.as_bytes()).await;
        stream.flush().await;
        stream
    }

    fn init_link(&mut self) -> Link {
        let (sender, receiver) = mpsc::channel(512);
        self.sender = Some(sender.clone());
        let link = Link::new(Session::new(sender), receiver);
        link
    }

    pub async fn connect(&mut self) {
        if self.handle.is_none() { return; }
        let handle_message = **self.handle.as_ref().unwrap();
        let mut stream = self.init().await;
        let mut link = self.init_link();
        tokio::spawn(async move {
            let mut buffer = [0; 1024];
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                let res = tokio::select! {
                    _ = interval.tick() => {
                        link.send_message(LinkMessage::OutputMessage(PingreqMessage::default().bytes)).await;
                        None
                    },
                    Ok(n) = stream.read(&mut buffer) => {
                        if n != 0 {
                            println!("length: {}",n);
                            link.send_message(LinkMessage::InputMessage(buffer[0..n].to_vec())).await;
                        }
                        None
                    },
                    kind = link.handle(handle_message) => kind
                };
                if let Some(kind) = res {
                    match kind {
                        ServerHandleKind::Response(data) => {
                            println!("data: {:?}", data);
                            if let Err(e) = stream.write_all(data.as_slice()).await {
                                println!("failed to write to socket; err = {:?}", e);
                            }
                        }
                        _ => {}
                    }
                }
            }
        });
    }
}
