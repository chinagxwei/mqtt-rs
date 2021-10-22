use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

use tokio::sync::Mutex;
use crate::message::v3;
use crate::message::v3::PublishMessage;
use crate::session::LinkMessage;
use crate::tools::protocol::{MqttDup, MqttQos, MqttRetain};

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
    WillV3(crate::message::v3::PublishMessage),
    WillV5(crate::message::v5::PublishMessage),
}

impl TopicMessage {
    pub fn generate_v3_topic_message(client_id: ClientID, will_qos: MqttQos, will_retain: MqttRetain, will_topic: String, will_message: String) -> TopicMessage {
        let msg = v3::PublishMessage::new(
            will_qos,
            MqttDup::Disable,
            will_retain,
            will_topic,
            0,
            will_message,
        );
        TopicMessage::ContentV3(client_id, msg)
    }
}

#[derive(Debug)]
pub struct Topic {
    name: String,
    senders: HashMap<ClientID, Sender<LinkMessage>>,
}

impl Topic {
    pub fn new<S: Into<String>>(name: S) -> Topic {
        Topic { name: name.into(), senders: HashMap::new() }
    }
}

impl Topic {
    pub fn subscript<S: Into<ClientID>>(&mut self, client_id: S, sender: Sender<LinkMessage>) {
        let id = client_id.into();
        println!("subscript client id: {:?}", &id);
        self.senders.insert(id, sender);
    }

    pub fn unsubscript<S: AsRef<ClientID>>(&mut self, client_id: S) -> Option<Sender<LinkMessage>> {
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
            sender.send(LinkMessage::HandleMessage(msg.clone())).await;
        }
    }

    pub fn contain<S: AsRef<ClientID>>(&self, client_id: S) -> bool {
        self.senders.contains_key(client_id.as_ref())
    }
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

    pub async fn new_subscript<S: AsRef<str>, SS: AsRef<ClientID>>(&self, topic_name: S, client_id: SS, sender: Sender<LinkMessage>) {
        let mut top = Topic::new(topic_name.as_ref());
        top.subscript(client_id.as_ref(), sender);
        self.add(topic_name.as_ref(), top).await;
    }

    pub fn subscript<S: AsRef<str>, SS: AsRef<ClientID>>(&self, topic_name: S, client_id: SS, sender: Sender<LinkMessage>) {
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

    pub async fn get_client<S: AsRef<str>, SS: AsRef<ClientID>>(&self, topic_name: S, client_id: SS) -> Sender<LinkMessage> {
        self.container.lock().await.get(topic_name.as_ref()).unwrap().senders.get(client_id.as_ref()).unwrap().clone()
    }
}
