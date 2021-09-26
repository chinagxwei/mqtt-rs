use crate::message::v3::{ConnackMessage, ConnectMessage, DisconnectMessage, MqttMessageV3, PubackMessage, PubcompMessage, PublishMessage, PubrecMessage, PubrelMessage, SubackMessage, SubscribeMessage, UnsubackMessage, UnsubscribeMessage};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Sender, Receiver};
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use crate::tools::protocol::{MqttProtocolLevel, MqttWillFlag, MqttQos, MqttRetain, MqttDup};
use crate::message::{MqttMessageKind, MqttBytesMessage, BaseMessage, MqttMessage, BaseConnect};
use crate::tools::types::TypeKind;

use crate::{SUBSCRIPT, v3_handle};
use crate::message::v5::MqttMessageV5;

#[derive(Debug, Clone, Eq, Hash)]
pub struct ClientID(pub String);

impl AsRef<ClientID> for ClientID {
    fn as_ref(&self) -> &ClientID {
        &self
    }
}

impl From<String> for ClientID {
    fn from(s: String) -> Self {
        ClientID(s)
    }
}

impl From<&str> for ClientID {
    fn from(s: &str) -> Self {
        ClientID(s.to_owned())
    }
}

impl From<&ClientID> for ClientID {
    fn from(client: &ClientID) -> Self {
        client.to_owned()
    }
}

impl PartialEq for ClientID {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.0, &other.0)
    }

    fn ne(&self, other: &Self) -> bool {
        PartialEq::ne(&self.0, &other.0)
    }
}


#[derive(Debug, Clone)]
pub enum TopicMessage {
    ContentV3(ClientID, crate::message::v3::PublishMessage),
    ContentV5(ClientID, crate::message::v5::PublishMessage),
    Will(PublishMessage),
}

pub struct Subscript {
    container: Arc<Mutex<HashMap<String, Topic>>>,
}

impl Subscript {
    pub fn new() -> Subscript {
        Subscript { container: Arc::new(Mutex::new(HashMap::default())) }
    }

    pub async fn contain<S: AsRef<str>>(&self, topic_name: S) -> bool {
        self.container.lock().await.contains_key(topic_name.as_ref())
    }

    pub async fn len(&self) -> usize {
        self.container.lock().await.len()
    }

    pub async fn add<S: Into<String>>(&self, topic_name: S, topic: Topic) -> Option<Topic> {
        self.container.lock().await.insert(topic_name.into(), topic)
    }

    pub async fn remove<S: AsRef<str>>(&self, topic_name: S) -> Option<Topic> {
        self.container.lock().await.remove(topic_name.as_ref())
    }

    pub async fn is_subscript<S: AsRef<str>, SS: AsRef<ClientID>>(&self, topic_name: S, client_id: SS) -> bool {
        self.container.lock().await.get(topic_name.as_ref()).unwrap().contain(client_id)
    }

    pub async fn new_subscript<S: AsRef<str>, SS: AsRef<ClientID>>(&self, topic_name: S, client_id: SS, sender: Sender<LineMessage>) {
        let mut top = Topic::new(topic_name.as_ref());
        top.subscript(client_id.as_ref(), sender);
        self.add(topic_name.as_ref(), top).await;
    }

    pub fn subscript<S: AsRef<str>, SS: AsRef<ClientID>>(&self, topic_name: S, client_id: SS, sender: Sender<LineMessage>) {
        match self.container.try_lock() {
            Ok(mut container) => {
                if let Some(t) = container.get_mut(topic_name.as_ref()) {
                    t.subscript(client_id.as_ref(), sender);
                }
            }
            Err(e) => {
                println!("{:?}", e)
            }
        }
    }

    pub async fn unsubscript<S: AsRef<str>, SS: AsRef<ClientID>>(&self, topic_name: S, client_id: SS) {
        self.container.lock().await.get_mut(topic_name.as_ref()).unwrap().unsubscript(client_id);
    }

    pub async fn exit<S: AsRef<ClientID>>(&self, client_id: S) {
        for (_, topic) in self.container.lock().await.iter_mut() {
            topic.unsubscript(client_id.as_ref());
        }
    }

    pub async fn topics(&self) -> Vec<String> {
        self.container.lock().await.keys().cloned().collect::<Vec<String>>()
    }

    pub async fn clients<S: AsRef<str>>(&self, topic_name: S) -> Vec<ClientID> {
        self.container.lock().await.get(topic_name.as_ref()).unwrap().client_id_list()
    }

    pub async fn client_len<S: AsRef<str>>(&self, topic_name: S) -> usize {
        self.container.lock().await.get(topic_name.as_ref()).unwrap().client_len()
    }

    pub async fn broadcast<S: AsRef<str>>(&self, topic_name: S, msg: &TopicMessage) {
        if let Some(t) = self.container.lock().await.get(topic_name.as_ref()) {
            t.broadcast(msg).await
        }
    }

    pub async fn get_client<S: AsRef<str>, SS: AsRef<ClientID>>(&self, topic_name: S, client_id: SS) -> Sender<LineMessage> {
        self.container.lock().await.get(topic_name.as_ref()).unwrap().senders.get(client_id.as_ref()).unwrap().clone()
    }
}

#[derive(Debug)]
pub struct Topic {
    name: String,
    senders: HashMap<ClientID, Sender<LineMessage>>,
}

impl Topic {
    pub fn new<S: Into<String>>(name: S) -> Topic {
        Topic { name: name.into(), senders: HashMap::new() }
    }
}

impl Topic {
    pub fn subscript<S: Into<ClientID>>(&mut self, client_id: S, sender: Sender<LineMessage>) {
        let id = client_id.into();
        println!("subscript client id: {:?}", &id);
        self.senders.insert(id, sender);
    }

    pub fn unsubscript<S: AsRef<ClientID>>(&mut self, client_id: S) -> Option<Sender<LineMessage>> {
        if self.senders.contains_key(client_id.as_ref()) {
            return self.senders.remove(client_id.as_ref());
        }
        None
    }

    pub fn client_id_list(&self) -> Vec<ClientID> {
        self.senders.keys().cloned().collect::<Vec<ClientID>>()
    }

    pub fn client_len(&self) -> usize {
        self.senders.len()
    }

    pub async fn broadcast(&self, msg: &TopicMessage) {
        for (_, sender) in self.senders.iter() {
            sender.send(LineMessage::SubscriptionMessage(msg.clone())).await;
        }
    }

    pub fn contain<S: AsRef<ClientID>>(&self, client_id: S) -> bool {
        self.senders.contains_key(client_id.as_ref())
    }
}

#[derive(Debug, Clone)]
pub enum LineMessage {
    SocketMessage(Vec<u8>),
    SubscriptionMessage(TopicMessage),
}

pub struct Line {
    sender: Sender<LineMessage>,
    receiver: Receiver<LineMessage>,
    client_id: Option<ClientID>,
    protocol_name: Option<String>,
    protocol_level: Option<MqttProtocolLevel>,
    will_flag: Option<MqttWillFlag>,
    will_qos: Option<MqttQos>,
    will_retain: Option<MqttRetain>,
    will_topic: Option<String>,
    will_message: Option<String>,
}

impl Line {
    pub fn new() -> Line {
        let (sender, receiver) = mpsc::channel(128);
        Line {
            sender,
            receiver,
            client_id: None,
            protocol_name: None,
            protocol_level: None,
            will_flag: None,
            will_qos: None,
            will_retain: None,
            will_topic: None,
            will_message: None,
        }
    }

    pub fn get_client_id(&self) -> &ClientID {
        self.client_id.as_ref().unwrap()
    }

    pub fn init_protocol(&mut self, protocol_name: String, protocol_level: MqttProtocolLevel) {
        self.protocol_name = Some(protocol_name);
        self.protocol_level = Some(protocol_level);
    }

    pub fn is_will_flag(&self) -> bool {
        self.will_flag.unwrap() == MqttWillFlag::Enable
    }

    pub fn get_v3_topic_message(&self) -> TopicMessage {
        let msg = PublishMessage::new(
            self.will_qos.unwrap(),
            MqttDup::Disable,
            self.will_retain.unwrap(),
            self.will_topic.as_ref().unwrap().to_owned(),
            0,
            self.will_message.as_ref().unwrap().to_owned(),
        );
        TopicMessage::ContentV3(self.get_client_id().clone(), msg)
    }

    pub fn get_will_topic(&self) -> &String {
        self.will_topic.as_ref().unwrap()
    }

    pub fn init_v3(&mut self, connect_msg: &ConnectMessage) {
        self.client_id = Some(ClientID(connect_msg.payload.client_id.to_owned()));
        self.will_flag = Some(connect_msg.will_flag);
        self.will_qos = Some(connect_msg.will_qos);
        self.will_retain = Some(connect_msg.will_retain);
        self.will_topic = connect_msg.payload.will_topic.clone();
        self.will_message = connect_msg.payload.will_message.clone();
    }

    pub fn init_v5(&mut self, connect_msg: &crate::message::v5::ConnectMessage) {
        self.client_id = Some(ClientID(connect_msg.payload.client_id.to_owned()));
        self.will_flag = Some(connect_msg.will_flag);
        self.will_qos = Some(connect_msg.will_qos);
        self.will_retain = Some(connect_msg.will_retain);
        self.will_topic = connect_msg.payload.will_topic.clone();
        self.will_message = connect_msg.payload.will_message.clone();
    }

    pub fn get_sender(&self) -> Sender<LineMessage> {
        self.sender.clone()
    }

    pub async fn recv(&mut self) -> Option<MqttMessageKind> {
        match self.receiver.recv().await {
            None => { None }
            Some(msg) => {
                match msg {
                    LineMessage::SocketMessage(msg) => {
                        // println!("socket msg");
                        let base_msg = BaseMessage::from(msg);
                        if base_msg.get_message_type() == TypeKind::CONNECT {
                            let connect = BaseConnect::from(&base_msg);
                            self.init_protocol(connect.get_protocol_name(), connect.get_protocol_level());
                        }

                        if let Some(level) = self.protocol_level {
                            return match level {
                                MqttProtocolLevel::Level3_1_1 => v3_handle::match_v3_data(self, base_msg).await,
                                MqttProtocolLevel::Level5 => {
                                    if let Some(v5) = MqttMessageKind::v5(base_msg) {
                                        if let MqttMessageKind::RequestV5(v5_type) = v5 {
                                            match v5_type {
                                                MqttMessageV5::Connect(msg) => {
                                                    println!("{:?}", msg);
                                                    self.init_v5(&msg);
                                                    let mut connack = crate::message::v5::ConnackMessage::default();
                                                    println!("{:?}", connack);
                                                    return Some(MqttMessageKind::Response(connack.bytes));
                                                }
                                                // MqttMessageV5::Connack(_) => {}
                                                MqttMessageV5::Publish(msg) => {
                                                    let topic_msg = TopicMessage::ContentV5(self.get_client_id().to_owned(), msg.clone());
                                                    println!("topic: {:?}", &msg.topic);
                                                    println!("topic: {:?}", topic_msg);
                                                    SUBSCRIPT.broadcast(&msg.topic, &topic_msg).await;
                                                    if msg.qos == MqttQos::Qos1 {
                                                        let mut puback = PubackMessage::new(msg.message_id);
                                                        return Some(MqttMessageKind::Response(puback.bytes.unwrap()));
                                                    }
                                                }
                                                // MqttMessageV5::Subscribe(msg) => {}
                                                // MqttMessageV5::Suback(_) => {}
                                                MqttMessageV5::Unsubscribe(msg) => {
                                                    let unsub = crate::message::v5::UnsubackMessage::from(msg);
                                                    return Some(MqttMessageKind::Response(unsub.as_bytes().to_vec()));
                                                }
                                                // MqttMessageV5::Unsuback(_) => {}
                                                // MqttMessageV5::Pingreq(_) => {}
                                                MqttMessageV5::Pingresp(msg) => {
                                                    println!("{:?}", msg);
                                                    return Some(MqttMessageKind::Response(msg.as_bytes().to_vec()));
                                                }
                                                MqttMessageV5::Disconnect(msg) => {
                                                    println!("client disconnect");
                                                    if self.is_will_flag() {
                                                        let topic_msg = self.get_v3_topic_message();
                                                        SUBSCRIPT.broadcast(self.get_will_topic(), &topic_msg).await;
                                                    }
                                                    SUBSCRIPT.exit(self.get_client_id()).await;
                                                    return Some(MqttMessageKind::Exit(msg.as_bytes().to_vec()));
                                                }
                                                _ => { return None; }
                                            }
                                        } else if v5.is_v5s() {
                                            if let Some(items) = v5.get_v5s() {
                                                let mut res = vec![];
                                                for x in items {
                                                    match x {
                                                        MqttMessageV5::Subscribe(msg) => {
                                                            println!("{:?}", msg);
                                                            let topic = &msg.topic;
                                                            if SUBSCRIPT.contain(topic).await {
                                                                SUBSCRIPT.subscript(topic, self.get_client_id(), self.get_sender());
                                                            } else {
                                                                SUBSCRIPT.new_subscript(topic, self.get_client_id(), self.get_sender()).await;
                                                            }
                                                            println!("broadcast topic len: {}", SUBSCRIPT.len().await);
                                                            println!("broadcast topic list: {:?}", SUBSCRIPT.topics().await);
                                                            println!("broadcast client len: {:?}", SUBSCRIPT.client_len(topic).await);
                                                            println!("broadcast client list: {:?}", SUBSCRIPT.clients(topic).await);
                                                            let mut suback = crate::message::v5::SubackMessage::from(msg.clone());
                                                            println!("{:?}", suback);
                                                            res.push(suback.bytes.unwrap());
                                                        }
                                                        MqttMessageV5::Unsubscribe(msg) => {
                                                            println!("topic name: {}", &msg.topic);
                                                            if SUBSCRIPT.contain(&msg.topic).await {
                                                                if SUBSCRIPT.is_subscript(&msg.topic, self.get_client_id()).await {
                                                                    SUBSCRIPT.unsubscript(&msg.topic, self.get_client_id()).await;
                                                                    let mut unsuback = crate::message::v5::UnsubackMessage::new(msg.message_id);
                                                                    res.push(unsuback.bytes.unwrap())
                                                                }
                                                            }
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                                return Some(MqttMessageKind::Response(res.concat()));
                                            }
                                            return None;
                                        }
                                        return None;
                                    }

                                    // let msg = crate::packet::v5_unpacket::connect(base_msg);
                                    // println!("{:?}", msg);
                                    return None;
                                }
                                _ => { return None; }
                            };
                        }
                        None
                    }
                    LineMessage::SubscriptionMessage(msg) => {
                        // println!("subscription msg");
                        return match msg {
                            TopicMessage::ContentV3(from_id, content) => {
                                println!("from: {:?}", from_id);
                                println!("to: {:?}", self.get_client_id());
                                if self.get_client_id() != &from_id {
                                    return Some(MqttMessageKind::Response(content.as_bytes().to_vec()));
                                }
                                None
                            }
                            TopicMessage::ContentV5(from_id, content) => {
                                println!("from: {:?}", from_id);
                                println!("to: {:?}", self.get_client_id());
                                if self.get_client_id() != &from_id {
                                    return Some(MqttMessageKind::Response(content.as_bytes().to_vec()));
                                }
                                None
                            }
                            TopicMessage::Will(content) => {
                                Some(MqttMessageKind::Response(content.as_bytes().to_vec()))
                            }
                        };
                    }
                }
            }
        }
    }
}


