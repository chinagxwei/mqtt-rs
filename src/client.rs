use std::future::Future;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpSocket;
use crate::message::{BaseMessage, MqttBytesMessage};
use crate::server::ServerHandleKind;
use crate::session::Session;
use crate::tools::config::Config;
use crate::tools::protocol::MqttCleanSession;

struct MqttClient<F, Fut>
    where
        F: Fn(Session, BaseMessage) -> Fut + Copy + Clone + Send + Sync + 'static,
        Fut: Future<Output=Option<ServerHandleKind>> + Send,
{
    config: Config,
    address: SocketAddr,
    handle: Option<Box<F>>,
}

impl<F, Fut> MqttClient<F, Fut>
    where
        F: Fn(Session, BaseMessage) -> Fut + Copy + Clone + Send + Sync + 'static,
        Fut: Future<Output=Option<ServerHandleKind>> + Send,
{
    pub fn new(config: Config, address: SocketAddr) -> MqttClient<F, Fut> {
        MqttClient { config, address, handle: None }
    }

    pub fn handle(mut self, f: F) -> MqttClient<F, Fut> {
        self.handle = Some(Box::new(f));
        self
    }

    pub fn publish(&self) {}

    pub fn subscribe(&self) {}

    pub fn unsubscribe(&self) {}

    pub fn ping(&self) {}

    pub fn disconnect(&self) {}

    pub async fn connect(&mut self) {
        if self.handle.is_none() { return; }
        // let socket = TcpSocket::new_v4().unwrap();
        // let mut stream = socket.connect(self.address).await.unwrap();
        // let mut buffer = [0; 1024];
        // let msg = ConnectMessage::new(MqttCleanSession::Enable, self.config.clone());
        // let mut interval = tokio::time::interval(Duration::from_secs(30));
        // stream.write(msg.as_bytes());
        // stream.flush();
        //
        // match stream.read(&mut buffer).await {
        //     Ok(n) => {
        //         let data = buffer[0..n].to_vec();
        //     }
        //     Err(_) => {}
        // }
        // loop {
        //
        // }
    }
}
