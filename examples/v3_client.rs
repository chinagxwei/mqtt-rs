use std::net::{Ipv4Addr, SocketAddrV4, SocketAddr};
use std::str::FromStr;
use mqtt_rs::executor::v3_client::MqttClient;
use mqtt_rs::message::MqttMessageKind;
use mqtt_rs::message::v3::MqttMessageV3;
use mqtt_rs::session::{ClientSession, MqttSession};
use mqtt_rs::tools::config::ConfigBuilder;

#[tokio::main]
async fn main() {
    // tokio::join!(start());
    start().await;
}

async fn start() {
    let socket = SocketAddrV4::new(
        Ipv4Addr::from_str("127.0.0.1").unwrap(),
        22222,
    );
    let builder = ConfigBuilder::default();
    let mut client = MqttClient::new(builder.build().unwrap(), SocketAddr::from(socket));
    let mut client = client.handle(handle_v3_message);
    let rc = client.connect().await;
}


pub async fn handle_v3_message(session: ClientSession, v3_kind: Option<MqttMessageKind>) {
    if let Some(MqttMessageKind::RequestV3(v3)) = v3_kind {
        match v3 {
            MqttMessageV3::Connack(_) => {
                session.subscribe(session.session_id()).await;
            }
            _ => {}
        }
    }
}
