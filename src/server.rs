use tokio::runtime::Runtime;
use tokio::net::{TcpListener, TcpStream};
use crate::types::TypeKind;
use crate::message::{MqttMessageKind, MqttBytesMessage, BaseMessage, MqttMessage, BaseConnect};
use crate::message::v3::{ConnackMessage, ConnectMessage, ConnectMessageBuild, DisconnectMessage, MqttMessageV3, PingrespMessage, PubackMessage, PubcompMessage, PublishMessage, PubrecMessage, PubrelMessage, SubackMessage, SubscribeMessage, UnsubackMessage, UnsubscribeMessage};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio::sync::mpsc::{Sender, Receiver};
use crate::TopicMessage;
use crate::protocol::{MqttProtocolLevel, MqttWillFlag, MqttQos, MqttRetain, MqttDup};
use tokio::io::{AsyncWriteExt, AsyncReadExt};

use crate::SUBSCRIPT;

fn handle_level3_1_1(base_msg: BaseMessage) -> Option<MqttMessageKind> {
    match base_msg.get_message_type() {
        TypeKind::CONNECT => {
            Some(MqttMessageKind::V3(MqttMessageV3::Connect(ConnectMessageBuild::from(base_msg).build())))
        }
        // TypeKind::CONNACK => {}
        TypeKind::PUBLISH => {
            Some(MqttMessageKind::V3(MqttMessageV3::Publish(PublishMessage::from(base_msg))))
        }
        TypeKind::PUBACK => {
            Some(MqttMessageKind::V3(MqttMessageV3::Puback(PubackMessage::from(base_msg))))
        }
        TypeKind::PUBREC => {
            Some(MqttMessageKind::V3(MqttMessageV3::Pubrec(PubrecMessage::from(base_msg))))
        }
        TypeKind::PUBREL => {
            Some(MqttMessageKind::V3(MqttMessageV3::Pubrel(PubrelMessage::from(base_msg))))
        }
        TypeKind::PUBCOMP => {
            Some(MqttMessageKind::V3(MqttMessageV3::Pubcomp(PubcompMessage::from(base_msg))))
        }
        TypeKind::SUBSCRIBE => {
            Some(MqttMessageKind::V3(MqttMessageV3::Subscribe(SubscribeMessage::from(base_msg))))
        }
        // TypeKind::SUBACK => {}
        TypeKind::UNSUBSCRIBE => {
            Some(MqttMessageKind::V3(MqttMessageV3::Unsubscribe(UnsubscribeMessage::from(base_msg))))
        }
        // TypeKind::UNSUBACK => {}
        TypeKind::PINGREQ => {
            Some(MqttMessageKind::V3(MqttMessageV3::Pingresp(PingrespMessage::default())))
        }
        // TypeKind::PINGRESP => {}
        TypeKind::DISCONNECT => {
            Some(MqttMessageKind::V3(MqttMessageV3::Disconnect(DisconnectMessage::default())))
        }
        // TypeKind::AUTH => {}
        _ => {
            return None;
        }
    }
}

pub struct MqttServer {
    host: String,
    port: u32,
    run_time: Option<Runtime>,
}

impl MqttServer {
    pub fn new<S: Into<String>>(host: S, port: u32) -> MqttServer {
        MqttServer { host: host.into(), port, run_time: None }
    }

    pub fn init(mut self) -> MqttServer {
        if self.run_time.is_none() {
            self.run_time = Option::from(Runtime::new().expect("create tokio is error"));
        }
        self
    }

    pub fn start(&self) {
        if self.run_time.is_none() {}
        self.run_time.as_ref().unwrap().block_on(async move {
            let listener = TcpListener::bind(format!("{}:{}", self.host, self.port)).await.expect("listener error");

            loop {
                let (mut socket, _) = listener.accept().await.expect("listener accept error");
                println!("new line");
                tokio::spawn(async move {
                    let mut line = Line::new(socket);

                    loop {
                        if !line.accept(|level, bmsg| {
                            // println!("{:?}", x.as_ref().unwrap().get_message_type());
                            println!("{:?}", level);
                            println!("{:?}", bmsg);

                            match level {
                                MqttProtocolLevel::Level3_1_1 => {
                                    if let Some(base_msg) = bmsg {
                                        let res = handle_level3_1_1(base_msg);
                                        // println!("res: {:?}", res);
                                        return res;
                                    }
                                    return None;
                                }
                                MqttProtocolLevel::Level5 => {
                                    return None;
                                }
                                _ => { return None; }
                            }

                            // println!("{:?}", x);
                            // None
                        }).await {
                            break;
                        }
                    }

                    println!("line end");
                });
            }
        })
    }
}

pub struct Line {
    socket: Arc<Mutex<TcpStream>>,
    buff: [u8; 1024],
    sender: Sender<TopicMessage>,
    receiver: Arc<Mutex<Receiver<TopicMessage>>>,
    client_id: Option<String>,
    protocol_name: Option<String>,
    protocol_level: Option<MqttProtocolLevel>,
    will_flag: Option<MqttWillFlag>,
    will_qos: Option<MqttQos>,
    will_retain: Option<MqttRetain>,
    will_topic: Option<String>,
    will_message: Option<String>,
}

impl Line {
    pub fn new(socket: TcpStream) -> Line {
        let socket = Arc::new(Mutex::new(socket));
        let (sender, receiver) = mpsc::channel(1);
        let receiver = Arc::new(Mutex::new(receiver));
        Line {
            socket,
            buff: [0_u8; 1024],
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

    fn action_receiver(&self) {
        let rec = Arc::clone(&self.receiver);
        let soc = Arc::clone(&self.socket);
        let to_client_id = self.client_id.as_ref().unwrap().to_owned();
        tokio::spawn(async move {
            // println!("new thread");
            // let id = client_id.to_owned();
            loop {
                if let Some(msg) = rec.clone().lock().await.recv().await {
                    // println!("id = {:?}", &client_id);
                    // println!("receiver = {:?}", msg);
                    match msg {
                        TopicMessage::Content(from_id, content) => {
                            if &to_client_id != &from_id {
                                println!("publish msg to [{}]: {:?}", &to_client_id, content);
                                if let Err(e) = soc.clone().lock().await.write_all(content.as_bytes()).await {
                                    println!("failed to write to socket; err = {:?}", e);
                                }
                            }
                            // soc.clone();
                            // println!("got = {}", str);
                        }
                        TopicMessage::Will(content) => {
                            println!("will msg to [{}]: {:?}", &to_client_id, content);
                            if let Err(e) = soc.clone().lock().await.write_all(content.as_bytes()).await {
                                println!("failed to write to socket; err = {:?}", e);
                            }
                        }
                        TopicMessage::Exit => {
                            break;
                        }
                    }
                }
            }
            // println!("thread end");
        });
    }

    pub fn init_protocol(&mut self, protocol_name: String, protocol_level: MqttProtocolLevel) {
        self.protocol_name = Some(protocol_name);
        self.protocol_level = Some(protocol_level);
    }

    pub fn init(&mut self, connect_msg: &ConnectMessage) {
        self.client_id = Some(connect_msg.payload.client_id.to_owned());
        self.will_flag = Some(connect_msg.will_flag);
        self.will_qos = Some(connect_msg.will_qos);
        self.will_retain = Some(connect_msg.will_retain);
        self.will_topic = connect_msg.payload.will_topic.clone();
        self.will_message = connect_msg.payload.will_message.clone();
        self.action_receiver()
    }

    pub fn get_sender(&self) -> Sender<TopicMessage> {
        self.sender.clone()
    }

    async fn get_message(&mut self) -> Option<BaseMessage> {
        let n = match self.socket.lock().await.read(self.buff.as_mut()).await {
            // socket closed
            Ok(n) if n == 0 => return None,
            Ok(n) => n,
            Err(e) => {
                println!("failed to read from socket; err = {:?}", e);
                return None;
            }
        };
        Some(BaseMessage::from(&self.buff[0..n]))
    }

    pub async fn accept<F>(&mut self, callback: F) -> bool
        where F: Fn(&MqttProtocolLevel, Option<BaseMessage>) -> Option<MqttMessageKind> {
        let msg = self.get_message().await;

        if msg.is_some() && msg.as_ref().unwrap().get_message_type() == TypeKind::CONNECT {
            let connect = BaseConnect::from(msg.as_ref().unwrap());
            self.init_protocol(connect.get_protocol_name(), connect.get_protocol_level());
        }

        if let Some(kind) = callback(self.protocol_level.as_ref().unwrap(), msg) {
            println!("accept: {:?}", kind);

            if let Some(kind) = kind.get_v3() {
                match kind {
                    MqttMessageV3::Connect(msg) => {
                        self.init(msg);
                        if let Err(e) = self.socket.lock().await.write_all(ConnackMessage::default().as_bytes()).await {
                            println!("failed to write to socket; err = {:?}", e);
                        }
                    }
                    MqttMessageV3::Puback(msg) => {
                        if let Err(e) = self.socket.lock().await.write_all(msg.as_bytes()).await {
                            println!("failed to write to socket; err = {:?}", e);
                        }
                    }
                    MqttMessageV3::Subscribe(msg) => {
                        println!("{:?}", msg);
                        if SUBSCRIPT.contain(&msg.topic).await {
                            SUBSCRIPT.subscript(&msg.topic, self.client_id.as_ref().unwrap(), self.get_sender());
                        } else {
                            SUBSCRIPT.new_subscript(&msg.topic, self.client_id.as_ref().unwrap(), self.get_sender()).await;
                        }
                        println!("broadcast topic len: {}", SUBSCRIPT.len().await);
                        println!("broadcast topic list: {:?}", SUBSCRIPT.topics().await);
                        println!("broadcast client len: {:?}", SUBSCRIPT.client_len(&msg.topic).await);
                        println!("broadcast client list: {:?}", SUBSCRIPT.clients(&msg.topic).await);
                        let sm = SubackMessage::from(msg.clone());
                        println!("{:?}", sm);
                        if let Err(e) = self.socket.lock().await.write_all(sm.as_bytes()).await {
                            println!("failed to write to socket; err = {:?}", e);
                        }
                    }
                    MqttMessageV3::Unsubscribe(msg) => {
                        println!("topic name: {}", &msg.topic);
                        if SUBSCRIPT.contain(&msg.topic).await {
                            if SUBSCRIPT.is_subscript(&msg.topic, self.client_id.as_ref().unwrap()).await {
                                SUBSCRIPT.unsubscript(&msg.topic, self.client_id.as_ref().unwrap()).await;
                                if let Err(e) = self.socket.lock().await.write_all(UnsubackMessage::new(msg.message_id).as_bytes()).await {
                                    println!("failed to write to socket; err = {:?}", e);
                                }
                            }
                        }
                    }
                    MqttMessageV3::Publish(msg) => {
                        println!("{:?}", msg);
                        let topic_msg = TopicMessage::Content(self.client_id.as_ref().unwrap().to_owned(), msg.clone());
                        SUBSCRIPT.broadcast(&msg.topic, &topic_msg).await;
                        if msg.qos == MqttQos::Qos1 {
                            // let p = PubackMessage::new(pmsg.message_id);
                            if let Err(e) = self.socket.lock().await.write_all(PubackMessage::new(msg.message_id).as_bytes()).await {
                                println!("failed to write to socket; err = {:?}", e);
                            }
                        }
                    }
                    MqttMessageV3::Pingresp(msg) => {
                        if let Err(e) = self.socket.lock().await.write_all(msg.as_bytes()).await {
                            println!("failed to write to socket; err = {:?}", e);
                        }
                    }
                    MqttMessageV3::Disconnect(_) => {
                        println!("client disconnect");
                        if self.will_flag.unwrap() == MqttWillFlag::Enable {
                            let msg = PublishMessage::new(
                                self.will_qos.unwrap(),
                                MqttDup::Disable,
                                self.will_retain.unwrap(),
                                self.will_topic.as_ref().unwrap().to_owned(),
                                0,
                                self.will_message.as_ref().unwrap().to_owned(),
                            );
                            let topic_msg = TopicMessage::Will(msg);
                            SUBSCRIPT.broadcast(self.will_topic.as_ref().unwrap(), &topic_msg).await;
                        }
                    }
                    _ => {}
                }
            }
            true
        } else {
            false
        }
    }

    pub fn close(&self) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            sender.send(TopicMessage::Exit).await;
        });
    }
}

impl Drop for Line {
    fn drop(&mut self) {
        self.close();
    }
}
