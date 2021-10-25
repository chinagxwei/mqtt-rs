use std::path::PathBuf;

mod v3_client;

pub struct MqttClientOption {
    cert: PathBuf,
}

impl MqttClientOption {
    pub fn new(cert: String) -> MqttClientOption {
        MqttClientOption { cert: PathBuf::from(cert) }
    }
}
