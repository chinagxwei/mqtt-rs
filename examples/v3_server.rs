use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str::FromStr;
use tokio::sync::mpsc::Sender;
use mqtt_demo::{SUBSCRIPT};
use mqtt_demo::message::{BaseMessage, MqttMessageKind};
use mqtt_demo::message::v3::{SubackMessage, DisconnectMessage, MqttMessageV3, PubackMessage, PublishMessage, SubscribeMessage, UnsubackMessage, UnsubscribeMessage};
use mqtt_demo::server::ServerHandleKind;
use mqtt_demo::server::v3_server::MqttServer;
use mqtt_demo::session::{LinkMessage, Session};
use mqtt_demo::subscript::TopicMessage;
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

pub async fn handle_v3_message(mut session: Session, v3_kind: Option<MqttMessageKind>) -> Option<ServerHandleKind> {
    if let Some(v3) = v3_kind {
        return match v3 {
            MqttMessageKind::RequestV3(ref msg) => {
                let res_msg = handle_v3(&mut session, Some(msg)).await.expect("handle v3 message error");
                if res_msg.is_disconnect() {
                    Some(ServerHandleKind::Exit(res_msg.as_bytes().to_vec()))
                } else {
                    Some(ServerHandleKind::Response(res_msg.as_bytes().to_vec()))
                }
            }
            MqttMessageKind::RequestV3Vec(ref items) => {
                let mut res = vec![];
                for x in items {
                    if let Some(res_msg) = handle_v3(&mut session, Some(x)).await {
                        res.push(res_msg.as_bytes().to_vec());
                    }
                }
                Some(ServerHandleKind::Response(res.concat()))
            }
            _ => None
        };
    }
    None
}

async fn handle_v3(session: &mut Session, kind_opt: Option<&MqttMessageV3>) -> Option<MqttMessageV3> {
    if let Some(kind) = kind_opt {
        return match kind {
            MqttMessageV3::Subscribe(msg) => handle_v3_subscribe(session, msg).await,
            MqttMessageV3::Unsubscribe(msg) => handle_v3_unsubscribe(session, msg).await,
            MqttMessageV3::Publish(msg) => handle_v3_publish(session, msg).await,
            MqttMessageV3::Pingresp(msg) => Some(MqttMessageV3::Pingresp(msg.clone())),
            MqttMessageV3::Disconnect(_) => handle_v3_disconnect(session).await,
            _ => None
        };
    }
    None
}

async fn handle_v3_publish(session: &mut Session, msg: &PublishMessage) -> Option<MqttMessageV3> {
    let topic_msg = TopicMessage::ContentV3(session.get_client_id().to_owned(), msg.clone());
    println!("topic: {:?}", topic_msg);
    SUBSCRIPT.broadcast(&msg.topic, &topic_msg).await;
    if msg.qos == MqttQos::Qos1 {
        return Some(MqttMessageV3::Puback(PubackMessage::new(msg.message_id)));
    }
    return None;
}

async fn handle_v3_subscribe(session: &mut Session, msg: &SubscribeMessage) -> Option<MqttMessageV3> {
    let sender: Sender<LinkMessage> = session.get_sender();
    println!("{:?}", msg);
    let topic = &msg.topic;
    if SUBSCRIPT.contain(topic).await {
        SUBSCRIPT.subscript(topic, session.get_client_id(), sender);
    } else {
        SUBSCRIPT.new_subscript(topic, session.get_client_id(), sender).await;
    }
    println!("broadcast topic len: {}", SUBSCRIPT.len().await);
    println!("broadcast topic list: {:?}", SUBSCRIPT.topics().await);
    println!("broadcast client len: {:?}", SUBSCRIPT.client_len(topic).await);
    println!("broadcast client list: {:?}", SUBSCRIPT.clients(topic).await);
    let sm = SubackMessage::from(msg.clone());
    println!("{:?}", sm);
    return Some(MqttMessageV3::Suback(sm));
}

async fn handle_v3_unsubscribe(session: &mut Session, msg: &UnsubscribeMessage) -> Option<MqttMessageV3> {
    println!("topic name: {}", &msg.topic);
    if SUBSCRIPT.contain(&msg.topic).await {
        if SUBSCRIPT.is_subscript(&msg.topic, session.get_client_id()).await {
            SUBSCRIPT.unsubscript(&msg.topic, session.get_client_id()).await;
            return Some(MqttMessageV3::Unsuback(UnsubackMessage::new(msg.message_id)));
        }
    }
    return None;
}

async fn handle_v3_disconnect(session: &mut Session) -> Option<MqttMessageV3> {
    println!("client disconnect");
    if session.is_will_flag() {
        if let Some(ref topic_msg) = session.get_will_message() {
            SUBSCRIPT.broadcast(session.get_will_topic(), topic_msg).await;
        }
    }
    SUBSCRIPT.exit(session.get_client_id()).await;
    return Some(MqttMessageV3::Disconnect(DisconnectMessage::default()));
}
