use std::convert::TryFrom;
use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpSocket, TcpStream};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio_rustls::rustls::OwnedTrustAnchor;
use tokio_rustls::{rustls, webpki, TlsConnector};
use crate::executor::{MqttClientOption, ReturnKind};
use crate::handle::{HandleEvent, LinkHandle};
use crate::handle::v3_client_handle::ClientHandle;
use crate::message::MqttMessageKind;
use crate::message::entity::{ConnectMessage, DisconnectMessage, PingreqMessage, PublishMessage, SubscribeMessage, UnsubscribeMessage};
use crate::message::v3::MqttMessageV3;
use crate::session::ClientSessionV3;
use crate::tools::config::Config;
use crate::tools::protocol::{MqttCleanSession, MqttDup, MqttQos, MqttRetain};

struct MqttClient<F, Fut>
    where
        F: Fn(ClientSessionV3, Option<MqttMessageKind>) -> Fut + Copy + Clone + Send + Sync + 'static,
        Fut: Future<Output=()> + Send,
{
    config: Config,
    address: SocketAddr,
    handle: Option<Box<F>>,
    sender: Option<Sender<HandleEvent>>,
    option: Option<MqttClientOption>,
}

impl<F, Fut> MqttClient<F, Fut>
    where
        F: Fn(ClientSessionV3, Option<MqttMessageKind>) -> Fut + Copy + Clone + Send + Sync + 'static,
        Fut: Future<Output=()> + Send,
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

    pub async fn publish(&self, topic: String, message: String, qos: MqttQos, dup: MqttDup, retain: MqttRetain) {
        if let Some(sender) = self.sender.as_ref() {
            let msg: MqttMessageV3 = PublishMessage::new(qos, dup, retain, topic, 0, message, None).into();
            sender.send(HandleEvent::OutputEvent(msg.to_vec().unwrap())).await;
        }
    }

    pub async fn subscribe(&self, topic: String, qos: MqttQos) {
        if let Some(sender) = self.sender.as_ref() {
            let msg: MqttMessageV3 = SubscribeMessage::new(0, topic, qos).into();
            sender.send(HandleEvent::OutputEvent(msg.to_vec().unwrap())).await;
        }
    }

    pub async fn unsubscribe(&self, topic: String) {
        if let Some(sender) = self.sender.as_ref() {
            let msg: MqttMessageV3 = UnsubscribeMessage::new(0, topic).into();
            sender.send(HandleEvent::OutputEvent(msg.to_vec().unwrap())).await;
        }
    }

    pub async fn disconnect(&self) {
        if let Some(sender) = self.sender.as_ref() {
            let msg: MqttMessageV3 = DisconnectMessage::default().into();
            sender.send(HandleEvent::OutputEvent(msg.to_vec().unwrap())).await;
        }
    }

    async fn init(&self) -> TcpStream {
        let socket = TcpSocket::new_v4().unwrap();
        socket.connect(self.address).await.unwrap()
    }

    fn init_handle(&mut self) -> ClientHandle {
        let (sender, receiver) = mpsc::channel(512);
        self.sender = Some(sender.clone());
        ClientHandle::new(ClientSessionV3::new(sender), receiver)
    }

    // fn tls_connector(&self) -> Option<TlsConnector> {
    //     if self.option.is_none() { return None; }
    //     let mut root_cert_store = rustls::RootCertStore::empty();
    //     let certs = client_load_certs(&self.option.as_ref().unwrap().cert).expect("load certs error");
    //     let trust_anchors = certs.iter().map(|cert| {
    //         let ta = webpki::TrustAnchor::try_from_cert_der(&cert[..]).unwrap();
    //         OwnedTrustAnchor::from_subject_spki_name_constraints(
    //             ta.subject,
    //             ta.spki,
    //             ta.name_constraints,
    //         )
    //     });
    //     root_cert_store.add_server_trust_anchors(trust_anchors);
    //     let config = rustls::ClientConfig::builder()
    //         .with_safe_defaults()
    //         .with_root_certificates(root_cert_store)
    //         .with_no_client_auth();
    //     Some(TlsConnector::from(Arc::new(config)))
    // }
    //
    // pub async fn connect_with_tls(&mut self) {
    //     if self.handle.is_none() { return; }
    //     if let Some(connector) = self.tls_connector() {
    //         let handle_message = **self.handle.as_ref().unwrap();
    //         let mut stream = self.init().await;
    //         let domain = rustls::ServerName::try_from(self.address.to_string().as_str())
    //             .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid dnsname"))
    //             .expect("into server name error");
    //         let mut stream = connector.connect(domain, stream).await.expect("");
    //         let link = self.init_link();
    //         let config = self.config.clone();
    //         tokio::spawn(async move {
    //             run(stream, handle_message, link, config).await;
    //         });
    //     }
    // }

    pub async fn connect(&mut self) {
        if self.handle.is_none() { return; }
        let callback = **self.handle.as_ref().unwrap();
        let stream = self.init().await;
        let handle = self.init_handle();
        let config = self.config.clone();
        tokio::spawn(async move {
            run(stream, callback, handle, config).await;
        });
    }
}

async fn run<S, F, Fut>(mut stream: S, callback: F, mut handle: ClientHandle, config: Config)
    where
        F: Fn(ClientSessionV3, Option<MqttMessageKind>) -> Fut + Copy + Clone + Send + Sync + 'static,
        Fut: Future<Output=()> + Send,
        S: AsyncReadExt + AsyncWriteExt + Unpin
{
    let msg = ConnectMessage::new(MqttCleanSession::Enable, config);
    handle.send_message(HandleEvent::OutputEvent(msg.bytes.unwrap())).await;
    let mut buffer = [0; 1024];
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    loop {
        let res = tokio::select! {
            _ = interval.tick() => {
                handle.send_message(HandleEvent::OutputEvent(PingreqMessage::default().bytes)).await;
                None
            },
            Ok(n) = stream.read(&mut buffer) => {
                if n != 0 {
                    println!("length: {}",n);
                    handle.send_message(HandleEvent::OutputEvent(buffer[0..n].to_vec())).await;
                }
                None
            },
            kind = handle.execute(callback) => kind
        };
        if let Some(kind) = res {
            match kind {
                ReturnKind::Response(data) => {
                    println!("data: {:?}", data);
                    if let Err(e) = stream.write_all(data.as_slice()).await {
                        println!("failed to write to socket; err = {:?}", e);
                    }
                }
                ReturnKind::Exit => break
            }
        }
    }
    println!("client service stop!")
}
