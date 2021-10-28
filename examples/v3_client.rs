use std::net::{Ipv4Addr, SocketAddrV4, SocketAddr};
use std::str::FromStr;
use mqtt_demo::executor::v3_client::MqttClient;
use mqtt_demo::message::MqttMessageKind;
use mqtt_demo::session::ClientSessionV3;
use mqtt_demo::tools::config::ConfigBuilder;

#[tokio::main]
async fn main() {
    // tokio::join!(start());
    start().await;
}

async fn start(){
    let socket = SocketAddrV4::new(
        Ipv4Addr::from_str("127.0.0.1").unwrap(),
        22222,
    );
    let builder = ConfigBuilder::default();
    let mut client = MqttClient::new(builder.build().unwrap(), SocketAddr::from(socket));
    // let mut client = client.handle(handle_v3_message);
    // client.disconnect()
    let rc = client.handle(handle_v3_message).connect().await;
}


pub async fn handle_v3_message(session: ClientSessionV3, v3_kind: Option<MqttMessageKind>) {

}
