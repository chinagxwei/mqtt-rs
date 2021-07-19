#[macro_use]
extern crate lazy_static;

use crate::message::v3::PublishMessage;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;

pub mod types;
pub mod hex;
pub mod tools;
mod config;
pub mod packet;
pub mod message;
pub mod protocol;
pub mod server;


lazy_static! {
    static ref SUBSCRIPT:Subscript = Subscript::new();
}

#[derive(Debug, Clone)]
pub enum TopicMessage {
    Content(String, PublishMessage),
    Will(PublishMessage),
    Exit,
}

struct Subscript {
    container: Arc<Mutex<HashMap<String, Topic>>>,
}

impl Subscript {
    pub fn new() -> Subscript {
        Subscript { container: Arc::new(Mutex::new(HashMap::default())) }
    }

    pub async fn contain(&self, topic_name: &String) -> bool {
        self.container.lock().await.contains_key(topic_name)
    }

    pub async fn len(&self) -> usize {
        self.container.lock().await.len()
    }

    pub async fn add<S: Into<String>>(&self, topic_name: S, topic: Topic) -> Option<Topic> {
        self.container.lock().await.insert(topic_name.into(), topic)
    }

    pub async fn delete(&self, topic_name: &String) -> Option<Topic> {
        self.container.lock().await.remove(topic_name)
    }

    pub async fn is_subscript(&self, topic_name: &String, client_id: &String) -> bool {
        self.container.lock().await.get(topic_name).unwrap().contain(client_id)
    }

    pub async fn new_subscript(&self, topic_name: &String, client_id: &String, sender: Sender<TopicMessage>) {
        let mut top = Topic::new(topic_name.to_owned());
        top.subscript(client_id, sender);
        self.add(topic_name, top).await;
    }

    pub fn subscript(&self, topic_name: &String, client_id: &String, sender: Sender<TopicMessage>) {
        match self.container.try_lock() {
            Ok(mut container) => {
                if let Some(t) = container.get_mut(topic_name) {
                    t.subscript(client_id, sender);
                }
            }
            Err(e) => {
                println!("{:?}", e)
            }
        }
    }

    pub async fn unsubscript(&self, topic_name: &String, client_id: &String) {
        self.container.lock().await.get_mut(topic_name).unwrap().unsubscript(client_id)
    }

    pub async fn topics(&self) -> Vec<String> {
        self.container.lock().await.keys().cloned().collect::<Vec<String>>()
    }

    pub async fn clients(&self, topic_name: &String) -> Vec<String> {
        self.container.lock().await.get(topic_name).unwrap().client_id_list()
    }

    pub async fn client_len(&self, topic_name: &String) -> usize {
        self.container.lock().await.get(topic_name).unwrap().client_len()
    }

    pub async fn broadcast(&self, topic_name: &String, msg: &TopicMessage) {
        if let Some(t) = self.container.lock().await.get(topic_name) {
            t.broadcast(msg).await
        }
    }
}

#[derive(Debug)]
pub struct Topic {
    name: String,
    senders: HashMap<String, Sender<TopicMessage>>,
}

impl Topic {
    pub fn new(name: String) -> Topic {
        Topic { name, senders: HashMap::new() }
    }
}

impl Topic {
    pub fn subscript<S: Into<String>>(&mut self, client_id: S, sender: Sender<TopicMessage>) {
        let id = client_id.into();
        println!("subscript client id: {}", &id);
        self.senders.insert(id, sender);
    }

    pub fn unsubscript(&mut self, client_id: &String) {
        self.senders.remove(client_id);
    }

    pub fn client_id_list(&self) -> Vec<String> {
        self.senders.keys().cloned().collect::<Vec<String>>()
    }

    pub fn client_len(&self) -> usize {
        self.senders.len()
    }

    pub async fn broadcast(&self, msg: &TopicMessage) {
        for (_, sender) in self.senders.iter() {
            sender.send(msg.clone()).await;
        }
    }

    pub fn contain(&self, client_id: &String) -> bool {
        self.senders.contains_key(client_id)
    }
}
