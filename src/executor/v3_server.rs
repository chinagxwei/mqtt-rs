use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_rustls::rustls;
use crate::message::MqttMessageKind;
use crate::session::ServerSessionV3;
use crate::tools::tls::{load_certs, load_keys};
use tokio_rustls::TlsAcceptor;
use crate::executor::{MqttServerOption, ReturnKind};
use crate::handle::{HandleEvent, LinkHandle};
use crate::handle::v3_server_handle::ServerHandler;

pub struct MqttServer<F, Fut>
    where
        F: Fn(ServerSessionV3, Option<MqttMessageKind>) -> Fut + Copy + Clone + Send + Sync + 'static,
        Fut: Future<Output=()> + Send,
{
    addr: SocketAddr,
    handle: Option<Box<F>>,
    option: Option<MqttServerOption>,
}

impl<F, Fut> MqttServer<F, Fut>
    where
        F: Fn(ServerSessionV3, Option<MqttMessageKind>) -> Fut + Copy + Clone + Send + Sync + 'static,
        Fut: Future<Output=()> + Send,
{
    pub fn new(addr: SocketAddr) -> MqttServer<F, Fut> {
        MqttServer { addr, handle: None, option: None }
    }

    pub fn option(mut self, option: MqttServerOption) -> MqttServer<F, Fut> {
        self.option = Some(option);
        self
    }

    pub fn handle(mut self, f: F) -> MqttServer<F, Fut> {
        self.handle = Some(Box::new(f));
        self
    }

    fn acceptor(&self) -> Option<TlsAcceptor> {
        if self.option.is_none() { return None; }
        let certs = load_certs(&self.option.as_ref().unwrap().cert).expect("load certs error");
        let mut keys = load_keys(&self.option.as_ref().unwrap().key).expect("load keys error");
        let config = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, keys.remove(0)).expect("build server tls config error");
        Some(TlsAcceptor::from(Arc::new(config)))
    }

    pub async fn start_with_tls(&self) {
        if self.handle.is_none() { return; }
        if let Some(acceptor) = self.acceptor() {
            let listener: TcpListener = TcpListener::bind(self.addr).await.expect("listener error");
            while let Ok((stream, _)) = listener.accept().await {
                let handle_message = **self.handle.as_ref().unwrap();
                let acceptor = acceptor.clone();
                let stream = acceptor.accept(stream).await.expect("");
                tokio::spawn(async move {
                    run(stream, handle_message).await;
                });
            }
        }
    }

    pub async fn start(&self) {
        if self.handle.is_none() { return; }
        let listener: TcpListener = TcpListener::bind(self.addr).await.expect("listener error");
        while let Ok((stream, _)) = listener.accept().await {
            let handle_message = **self.handle.as_ref().unwrap();
            tokio::spawn(async move {
                run(stream, handle_message).await;
            });
        }
    }
}

async fn run<S, F, Fut>(mut stream: S, callback: F)
    where
        F: Fn(ServerSessionV3, Option<MqttMessageKind>) -> Fut + Copy + Clone + Send + Sync + 'static,
        Fut: Future<Output=()> + Send,
        S: AsyncReadExt + AsyncWriteExt + Unpin
{
    let mut buf = [0; 1024];
    let mut handle = ServerHandler::new();
    loop {
        let res = tokio::select! {
            Ok(n) = stream.read(&mut buf) => {
                if n != 0 {handle.send_message(HandleEvent::InputEvent(buf[0..n].to_vec())).await;}
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
}
