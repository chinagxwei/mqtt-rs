use std::path::PathBuf;

pub mod v3_client;
pub mod v3_server;

pub enum ReturnKind {
    Response(Vec<u8>),
    Exit,
}

pub struct MqttClientOption {
    cert: PathBuf,
}

impl MqttClientOption {
    pub fn new(cert: String) -> MqttClientOption {
        MqttClientOption { cert: PathBuf::from(cert) }
    }
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
