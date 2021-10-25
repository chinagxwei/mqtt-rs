use std::convert::TryFrom;
use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::option::Option::Some;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpSocket, TcpStream};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio_rustls::rustls::OwnedTrustAnchor;
use tokio_rustls::{rustls, webpki, TlsConnector};
use crate::client::MqttClientOption;
use crate::message::{BaseMessage, MqttBytesMessage, MqttMessageKind, PingreqMessage};
use crate::message::v3::{ConnectMessage, DisconnectMessage, PublishMessage, SubscribeMessage, UnsubscribeMessage};
use crate::server::ServerHandleKind;
use crate::session::{LinkHandle, LinkMessage, Session};
use crate::session::v3_client_link::Link;

use crate::tools::config::Config;
use crate::tools::protocol::{MqttCleanSession, MqttQos};
use crate::tools::tls::{client_load_certs, load_certs};

struct MqttClient<F, Fut>
    where
        F: Fn(Session, Option<MqttMessageKind>) -> Fut + Copy + Clone + Send + Sync + 'static,
        Fut: Future<Output=Option<ServerHandleKind>> + Send,
{
    config: Config,
    address: SocketAddr,
    handle: Option<Box<F>>,
    sender: Option<Sender<LinkMessage>>,
    option: Option<MqttClientOption>,
}

impl<F, Fut> MqttClient<F, Fut>
    where
        F: Fn(Session, Option<MqttMessageKind>) -> Fut + Copy + Clone + Send + Sync + 'static,
        Fut: Future<Output=Option<ServerHandleKind>> + Send,
{
    pub fn new(config: Config, address: SocketAddr) -> MqttClient<F, Fut> {
        MqttClient { config, address, handle: None, sender: None, option: None }
    }

    pub fn option(mut self, option: MqttClientOption) -> MqttClient<F, Fut> {
        self.option = Some(option);
        self
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
        socket.connect(self.address).await.unwrap()
    }

    fn init_link(&mut self) -> Link {
        let (sender, receiver) = mpsc::channel(512);
        self.sender = Some(sender.clone());
        let link = Link::new(Session::new(sender), receiver);
        link
    }

    fn tls_connector(&self) -> Option<TlsConnector> {
        if self.option.is_none() { return None; }
        let mut root_cert_store = rustls::RootCertStore::empty();
        let certs = client_load_certs(&self.option.as_ref().unwrap().cert).expect("load certs error");
        let trust_anchors = certs.iter().map(|cert| {
            let ta = webpki::TrustAnchor::try_from_cert_der(&cert[..]).unwrap();
            OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        });
        root_cert_store.add_server_trust_anchors(trust_anchors);
        let config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();
        Some(TlsConnector::from(Arc::new(config)))
    }

    pub async fn connect_with_tls(&mut self) {
        if self.handle.is_none() { return; }
        if let Some(connector) = self.tls_connector() {
            let handle_message = **self.handle.as_ref().unwrap();
            let mut stream = self.init().await;
            let domain = rustls::ServerName::try_from(self.address.to_string().as_str())
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid dnsname"))
                .expect("into server name error");
            let mut stream = connector.connect(domain, stream).await.expect("");
            let link = self.init_link();
            let config = self.config.clone();
            tokio::spawn(async move {
                run(stream, handle_message, link, config).await;
            });
        }
    }

    pub async fn connect(&mut self) {
        if self.handle.is_none() { return; }
        let handle_message = **self.handle.as_ref().unwrap();
        let mut stream = self.init().await;
        let link = self.init_link();
        let config = self.config.clone();
        tokio::spawn(async move {
            run(stream, handle_message, link, config).await;
        });
    }
}

async fn run<S, F, Fut>(mut stream: S, handle: F, mut link: Link, config: Config)
    where
        F: Fn(Session, Option<MqttMessageKind>) -> Fut + Copy + Clone + Send + Sync + 'static,
        Fut: Future<Output=Option<ServerHandleKind>> + Send,
        S: AsyncReadExt + AsyncWriteExt + Unpin
{
    let msg = ConnectMessage::new(MqttCleanSession::Enable, config);
    link.send_message(LinkMessage::OutputMessage(msg.bytes.unwrap())).await;
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
            kind = link.handle(handle) => kind
        };
        if let Some(kind) = res {
            match kind {
                ServerHandleKind::Response(data) => {
                    println!("data: {:?}", data);
                    if let Err(e) = stream.write_all(data.as_slice()).await {
                        println!("failed to write to socket; err = {:?}", e);
                    }
                }
                ServerHandleKind::Exit(_) => break
            }
        }
    }
    println!("client service stop!")
}
