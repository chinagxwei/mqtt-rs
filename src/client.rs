use std::net::SocketAddr;
use crate::tools::config::Config;

struct Client {
    config: Config,
    address: SocketAddr,
}

impl Client {
    pub fn new(config: Config, address: SocketAddr) -> Client {
        Client { config, address}
    }

    pub fn publish(&self) {}

    pub fn subscribe(&self) {}

    pub fn unsubscribe(&self) {}

    pub fn ping(&self) {}

    pub fn disconnect(&self) {}

    pub async fn connect(&mut self) {

    }
}
