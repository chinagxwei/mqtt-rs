use std::path::PathBuf;

pub mod v3_server;

pub enum ServerHandleKind {
    Response(Vec<u8>),
    Exit(Vec<u8>),
}

pub struct MqttServerOption {
    cert: PathBuf,
    key: PathBuf,
}

impl MqttServerOption {
    pub fn new(cert: String, key: String) -> MqttServerOption {
        MqttServerOption { cert: PathBuf::from(cert), key: PathBuf::from(key) }
    }
}
