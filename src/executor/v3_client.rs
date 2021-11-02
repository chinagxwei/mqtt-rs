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
use crate::handle::{HandleEvent, ClientExecute, Response};
use crate::handle::v3_client_handle::ClientHandleV3;
use crate::message::MqttMessageKind;
use crate::message::entity::{ConnectMessage, PublishMessage, SubscribeMessage, UnsubscribeMessage};
use crate::message::v3::MqttMessageV3;
use crate::session::ClientSession;
use crate::tools::config::Config;
use crate::tools::protocol::{MqttCleanSession, MqttDup, MqttQos, MqttRetain};

pub struct MqttClient<F, Fut>
    where
        F: Fn(ClientSession, Option<MqttMessageKind>) -> Fut + Copy + Clone + Send + Sync + 'static,
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
        F: Fn(ClientSession, Option<MqttMessageKind>) -> Fut + Copy + Clone + Send + Sync + 'static,
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
            let msg = MqttMessageV3::Publish(PublishMessage::new(qos, dup, retain, topic, 0, message, None))
                .to_vec()
                .unwrap();
            sender.send(
                HandleEvent::OutputEvent(
                    Response(msg, self.config.protocol_level())
                )
            ).await;
        }
    }

    pub async fn subscribe(&self, topic: String, qos: MqttQos) {
        if let Some(sender) = self.sender.as_ref() {
            let msg = MqttMessageV3::Subscribe(SubscribeMessage::new(0, topic, qos))
                .to_vec()
                .unwrap();
            sender.send(
                HandleEvent::OutputEvent(
                    Response(msg, self.config.protocol_level())
                )
            ).await;
        }
    }

    pub async fn unsubscribe(&self, topic: String) {
        if let Some(sender) = self.sender.as_ref() {
            let msg = MqttMessageV3::Unsubscribe(UnsubscribeMessage::new(0, topic))
                .to_vec()
                .unwrap();
            sender.send(
                HandleEvent::OutputEvent(
                    Response(msg, self.config.protocol_level())
                )
            ).await;
        }
    }

    pub async fn disconnect(&self) {
        if let Some(sender) = self.sender.as_ref() {
            sender.send(HandleEvent::ExitEvent(true)).await;
        }
    }

    async fn init(&self) -> TcpStream {
        let socket = TcpSocket::new_v4().unwrap();
        socket.connect(self.address).await.unwrap()
    }

    fn init_handle(&mut self) -> ClientHandleV3 {
        let (sender, receiver) = mpsc::channel(512);
        self.sender = Some(sender.clone());
        ClientHandleV3::new(
            ClientSession::new(
                self.config.client_id().clone(),
                self.config.protocol_level(),
                sender,
            ),
            receiver,
        )
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

    pub async fn connect(&mut self) -> Option<mpsc::Receiver<String>> {
        if self.handle.is_none() { return None; }
        let callback = **self.handle.as_ref().unwrap();
        let stream = self.init().await;
        let handle = self.init_handle();
        let config = self.config.clone();
        let (tx, rx) = mpsc::channel(512);
        tokio::spawn(async move {
            run(stream, callback, Some(tx), handle, config).await;
        });
        Some(rx)
    }
}

impl<F, Fut> Drop for MqttClient<F, Fut>
    where
        F: Fn(ClientSession, Option<MqttMessageKind>) -> Fut + Copy + Clone + Send + Sync + 'static,
        Fut: Future<Output=()> + Send,
{
    fn drop(&mut self) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            if let Some(sender) = sender.as_ref() {
                if let Err(e) = sender.send(HandleEvent::ExitEvent(true)).await {
                    println!("failed to send subscribe message; err = {:?}", e);
                }
            }
        });
    }
}

async fn run<S, F, Fut>(mut stream: S, callback: F, sender: Option<mpsc::Sender<String>>, mut handle: ClientHandleV3, config: Config)
    where
        F: Fn(ClientSession, Option<MqttMessageKind>) -> Fut + Copy + Clone + Send + Sync + 'static,
        Fut: Future<Output=()> + Send,
        S: AsyncReadExt + AsyncWriteExt + Unpin
{
    let mut interval = tokio::time::interval(Duration::from_secs(config.keep_alive() as u64));

    let level = config.protocol_level();

    let msg = MqttMessageV3::Connect(ConnectMessage::new(MqttCleanSession::Enable, config))
        .to_vec()
        .unwrap();
    handle.send_message(
        HandleEvent::OutputEvent(
            Response(msg, level)
        )
    ).await;

    let mut buffer = [0; 1024];

    loop {
        let cp_sender = sender.clone();
        let res = tokio::select! {
            _ = interval.tick() => {
                handle.send_message(HandleEvent::OutputEvent(Response(MqttMessageV3::ping().unwrap(),level))).await;
                None
            },
            Ok(n) = stream.read(&mut buffer) => {
                if n != 0 {handle.send_message(HandleEvent::InputEvent(buffer[0..n].to_vec())).await;}
                None
            },
            kind = handle.execute(callback, cp_sender) => kind
        };
        if let Some(kind) = res {
            match kind {
                ReturnKind::Response(data) => {
                    println!("client output: {:?}", data);
                    if let Err(e) = stream.write_all(data.as_slice()).await {
                        println!("failed to write to socket; err = {:?}", e);
                    }
                }
                ReturnKind::Exit => {
                    let msg = MqttMessageV3::disconnect().unwrap();
                    if let Err(e) = stream.write_all(msg.as_slice()).await {
                        println!("failed to write to socket; err = {:?}", e);
                    }
                    break;
                }
            }
        }
    }
    println!("client service stop!")
}
