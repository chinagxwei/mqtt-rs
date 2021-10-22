use std::future::Future;
use std::marker::PhantomData;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str::FromStr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use crate::message::{BaseMessage, MqttMessageKind};
use crate::message::v3::{SubackMessage, DisconnectMessage, MqttMessageV3, PubackMessage, PublishMessage, SubscribeMessage, UnsubackMessage, UnsubscribeMessage};
use crate::server::ServerHandleKind;
use crate::session::v3_link::Link;
use crate::session::{LinkMessage, Session};
use crate::subscript::TopicMessage;
use crate::tools::protocol::MqttQos;
use crate::SUBSCRIPT;

pub struct MqttServer<F, Fut>
    where
        F: Fn(Session, BaseMessage) -> Fut + Copy + Clone + Send + Sync + 'static,
        Fut: Future<Output=Option<ServerHandleKind>> + Send,
{
    addr: SocketAddr,
    handle: Option<Box<F>>,
}

impl<F, Fut> MqttServer<F, Fut>
    where
        F: Fn(Session, BaseMessage) -> Fut + Copy + Clone + Send + Sync + 'static,
        Fut: Future<Output=Option<ServerHandleKind>> + Send,
{
    pub fn new(addr: SocketAddr) -> MqttServer<F, Fut> {
        MqttServer { addr, handle: None }
    }

    pub fn handle(mut self, f: F) -> MqttServer<F, Fut> {
        self.handle = Some(Box::new(f));
        self
    }

    pub async fn start(&self) {
        if self.handle.is_none() { return; }
        let listener: TcpListener = TcpListener::bind(self.addr).await.expect("listener error");
        while let Ok((mut stream, _)) = listener.accept().await {
            let handle_message = **self.handle.as_ref().unwrap();

            tokio::spawn(async move {
                let mut buf = [0; 1024];
                let mut link = Link::new();
                loop {
                    let res = tokio::select! {
                        Ok(n) = stream.read(&mut buf) => {
                            if n != 0 {
                                println!("length: {}",n);
                                link.send_message(LinkMessage::SocketMessage(buf[0..n].to_vec())).await;
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
                            ServerHandleKind::Exit(data) => {
                                if let Err(e) = stream.write_all(data.as_slice()).await {
                                    println!("failed to write to socket; err = {:?}", e);
                                }
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            });
        }
    }
}
