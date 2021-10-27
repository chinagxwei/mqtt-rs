use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str::FromStr;
use mqtt_demo::message::entity::{ConnackMessage, DisconnectMessage, PubackMessage, PubrecMessage, UnsubackMessage, SubackMessage, PubrelMessage, PubcompMessage};
use mqtt_demo::message::MqttMessageKind;
use mqtt_demo::message::v3::MqttMessageV3;
use mqtt_demo::server::ServerHandleKind;
use mqtt_demo::server::v3_server::MqttServer;
use mqtt_demo::session::{MqttSession, ServerSessionV3};
use mqtt_demo::tools::protocol::MqttQos;

#[tokio::main]
async fn main() {
    let socket = SocketAddrV4::new(
        Ipv4Addr::from_str("127.0.0.1").unwrap(),
        22222,
    );
    let server = MqttServer::new(SocketAddr::from(socket));
    server
        .handle(handle_v3_message)
        .start()
        .await;
}

pub async fn handle_v3_message(session: ServerSessionV3, v3_kind: Option<MqttMessageKind>) {
    if let Some(v3) = v3_kind {
        match v3 {
            MqttMessageKind::RequestV3(ref msg) => {
                let res_msg = handle_v3(&session, Some(msg)).await.expect("handle v3 message error");
                if res_msg.is_disconnect() {
                    session.exit().await;
                } else {
                    session.send(res_msg.to_vec().unwrap()).await
                }
            }
            MqttMessageKind::RequestV3Vec(ref items) => {
                let mut res = vec![];
                for x in items {
                    if let Some(res_msg) = handle_v3(&session, Some(x)).await {
                        res.push(res_msg.to_vec().unwrap());
                    }
                }
                session.send(res.concat()).await
            }
            _ => {}
        };
    }
}

async fn handle_v3(session: &ServerSessionV3, kind_opt: Option<&MqttMessageV3>) -> Option<MqttMessageV3> {
    if let Some(kind) = kind_opt {
        return match kind {
            MqttMessageV3::Connect(_) => {
                return Some(MqttMessageV3::Connack(ConnackMessage::default()));
            }
            MqttMessageV3::Subscribe(msg) => {
                session.subscribe(&msg.topic).await;
                let sm = SubackMessage::from(msg.clone());
                println!("{:?}", sm);
                return Some(MqttMessageV3::Suback(sm));
            }
            MqttMessageV3::Unsubscribe(msg) => Some(MqttMessageV3::Unsuback(UnsubackMessage::new(msg.message_id, None))),
            MqttMessageV3::Publish(msg) => {
                session.publish(msg).await;
                match msg.qos {
                    MqttQos::Qos1 => return Some(MqttMessageV3::Puback(PubackMessage::new(msg.message_id))),
                    MqttQos::Qos2 => return Some(MqttMessageV3::Pubrec(PubrecMessage::new(msg.message_id))),
                    _ => return None
                }
            }
            MqttMessageV3::Pubrec(msg) => {
                Some(MqttMessageV3::Pubrel(PubrelMessage::from(msg.clone())))
            }
            MqttMessageV3::Pubrel(msg) => {
                Some(MqttMessageV3::Pubcomp(PubcompMessage::from(msg.clone())))
            }
            MqttMessageV3::Pingresp(msg) => Some(MqttMessageV3::Pingresp(msg.clone())),
            MqttMessageV3::Disconnect(_) => Some(MqttMessageV3::Disconnect(DisconnectMessage::default())),
            _ => None
        };
    }
    None
}
