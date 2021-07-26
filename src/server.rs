use tokio::runtime::Runtime;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncWrite};
use crate::{ClientID, TopicMessage};
use crate::protocol::{MqttProtocolLevel, MqttWillFlag, MqttQos, MqttRetain, MqttDup};
use tokio::sync::mpsc::{Sender, Receiver};
use crate::message::v3::{ConnackMessage, ConnectMessage, DisconnectMessage, MqttMessageV3, PingrespMessage, PubackMessage, PubcompMessage, PublishMessage, PubrecMessage, PubrelMessage, SubackMessage, SubscribeMessage, UnsubackMessage, UnsubscribeMessage};
use tokio::sync::mpsc;
use crate::message::{BaseMessage, MqttMessage, BaseConnect, MqttMessageKind, MqttBytesMessage};
use crate::types::TypeKind;

use crate::SUBSCRIPT;

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
        if self.run_time.is_none() { return; }
        self.run_time.as_ref().unwrap().block_on(async move {
            let listener = TcpListener::bind(format!("{}:{}", self.host, self.port)).await.expect("listener error");

            loop {
                let (mut socket, _) = listener.accept().await.expect("listener accept error");

                tokio::spawn(async move {
                    let mut buf = [0; 1024];
                    let mut line = Line2::new();
                    // In a loop, read data from the socket and write the data back.
                    'end_loop: loop {
                        let res = tokio::select! {
                                Ok(n) = socket.read(&mut buf) => {
                                    if n != 0 {
                                        // println!("length: {}",n);
                                        line.get_sender().send(LineMessage::SocketMessage(buf[0..n].to_vec())).await;
                                    }
                                    None
                                },
                                kind = line.recv() => kind,
                            };
                        if let Some(kind) = res {
                            match kind {
                                MqttMessageKind::Broadcast(data) => {
                                    println!("{:?}", line.client_id.as_ref());
                                    println!("{:?}", data);

                                    if let Err(e) = socket.write_all(data.as_slice()).await {
                                        println!("failed to write to socket; err = {:?}", e);
                                    }
                                    println!("send data: {:?}", PublishMessage::from(BaseMessage::from(data)))
                                }
                                MqttMessageKind::V3(ref msg) => {
                                    if let Some(res_msg) = handle_v3(&mut line, Some(msg)).await {
                                        println!("{:?}", line.client_id.as_ref());
                                        println!("{:?}", res_msg);
                                        match res_msg {
                                            MqttMessageV3::Connack(msg) => {
                                                if let Err(e) = socket.write_all(msg.as_bytes()).await {
                                                    println!("failed to write to socket; err = {:?}", e);
                                                }
                                            }
                                            MqttMessageV3::Suback(msg) => {
                                                if let Err(e) = socket.write_all(msg.as_bytes()).await {
                                                    println!("failed to write to socket; err = {:?}", e);
                                                }
                                            }
                                            MqttMessageV3::Puback(msg) => {
                                                if let Err(e) = socket.write_all(msg.as_bytes()).await {
                                                    println!("failed to write to socket; err = {:?}", e);
                                                }
                                            }
                                            MqttMessageV3::Unsuback(msg) => {
                                                if let Err(e) = socket.write_all(msg.as_bytes()).await {
                                                    println!("failed to write to socket; err = {:?}", e);
                                                }
                                            }
                                            MqttMessageV3::Pingresp(msg) => {
                                                if let Err(e) = socket.write_all(msg.as_bytes()).await {
                                                    println!("failed to write to socket; err = {:?}", e);
                                                }
                                            }
                                            MqttMessageV3::Disconnect(msg) => {
                                                if let Err(e) = socket.write_all(msg.as_bytes()).await {
                                                    println!("failed to write to socket; err = {:?}", e);
                                                }
                                                break 'end_loop;
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                MqttMessageKind::V5 => {}
                            }
                        }
                    }
                });
            }
        })
    }
}

#[derive(Debug, Clone)]
pub enum LineMessage {
    SocketMessage(Vec<u8>),
    SubscriptionMessage(TopicMessage),
}

pub struct Line2 {
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

impl Line2 {
    pub fn new() -> Line2 {
        let (sender, receiver) = mpsc::channel(128);
        Line2 {
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

    pub fn init(&mut self, connect_msg: &ConnectMessage) {
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
                        println!("socket msg");
                        let base_msg = BaseMessage::from(msg);
                        if base_msg.get_message_type() == TypeKind::CONNECT {
                            let connect = BaseConnect::from(&base_msg);
                            self.init_protocol(connect.get_protocol_name(), connect.get_protocol_level());
                        }
                        match self.protocol_level {
                            None => { None }
                            Some(level) => {
                                match level {
                                    MqttProtocolLevel::Level3_1_1 => {
                                        return MqttMessageKind::v3(base_msg);
                                    }
                                    MqttProtocolLevel::Level5 => {
                                        None
                                    }
                                    _ => { None }
                                }
                            }
                        }
                    }
                    LineMessage::SubscriptionMessage(msg) => {
                        println!("subscription msg");
                        return match msg {
                            TopicMessage::Content(from_id, content) => {
                                let to_client_id = self.client_id.as_ref().unwrap();
                                if to_client_id != &from_id {
                                    return Some(MqttMessageKind::Broadcast(content.as_bytes().to_vec()));
                                }
                                None
                            }
                            TopicMessage::Will(content) => {
                                Some(MqttMessageKind::Broadcast(content.as_bytes().to_vec()))
                            }
                        };
                    }
                }
            }
        }
    }
}

async fn handle_v3(line: &mut Line2, kind_opt: Option<&MqttMessageV3>) -> Option<MqttMessageV3> {
    if let Some(kind) = kind_opt {
        match kind {
            MqttMessageV3::Connect(msg) => {
                line.init(msg);
                // if let Err(e) = self.socket.lock().await.write_all(ConnackMessage::default().as_bytes()).await {
                //     println!("failed to write to socket; err = {:?}", e);
                // }
                return Some(MqttMessageV3::Connack(ConnackMessage::default()));
            }
            // MqttMessageV3::Puback(msg) => {
            //     if let Err(e) = self.socket.lock().await.write_all(msg.as_bytes()).await {
            //         println!("failed to write to socket; err = {:?}", e);
            //     }
            // }
            MqttMessageV3::Subscribe(msg) => {
                println!("{:?}", msg);
                if SUBSCRIPT.contain(&msg.topic).await {
                    SUBSCRIPT.subscript(&msg.topic, line.client_id.as_ref().unwrap(), line.get_sender());
                } else {
                    SUBSCRIPT.new_subscript(&msg.topic, line.client_id.as_ref().unwrap(), line.get_sender()).await;
                }
                println!("broadcast topic len: {}", SUBSCRIPT.len().await);
                println!("broadcast topic list: {:?}", SUBSCRIPT.topics().await);
                println!("broadcast client len: {:?}", SUBSCRIPT.client_len(&msg.topic).await);
                println!("broadcast client list: {:?}", SUBSCRIPT.clients(&msg.topic).await);
                let sm = SubackMessage::from(msg.clone());
                println!("{:?}", sm);
                return Some(MqttMessageV3::Suback(sm));
                // if let Err(e) = self.socket.lock().await.write_all(sm.as_bytes()).await {
                //     println!("failed to write to socket; err = {:?}", e);
                // }
            }
            MqttMessageV3::Unsubscribe(msg) => {
                println!("topic name: {}", &msg.topic);
                if SUBSCRIPT.contain(&msg.topic).await {
                    if SUBSCRIPT.is_subscript(&msg.topic, line.client_id.as_ref().unwrap().as_ref()).await {
                        SUBSCRIPT.unsubscript(&msg.topic, line.client_id.as_ref().unwrap().as_ref()).await;
                        // if let Err(e) = self.socket.lock().await.write_all(UnsubackMessage::new(msg.message_id).as_bytes()).await {
                        //     println!("failed to write to socket; err = {:?}", e);
                        // }
                        return Some(MqttMessageV3::Unsuback(UnsubackMessage::new(msg.message_id)));
                    }
                }
                return None;
            }
            MqttMessageV3::Publish(msg) => {
                // println!("{:?}", msg);
                let topic_msg = TopicMessage::Content(line.client_id.as_ref().unwrap().to_owned(), msg.refresh());
                println!("topic: {:?}", topic_msg);
                SUBSCRIPT.broadcast(&msg.topic, &topic_msg).await;
                if msg.qos == MqttQos::Qos1 {
                    return Some(MqttMessageV3::Puback(PubackMessage::new(msg.message_id)));
                }
                return None;
            }
            MqttMessageV3::Pingresp(msg) => {
                return Some(MqttMessageV3::Pingresp(msg.clone()));
            }
            MqttMessageV3::Disconnect(_) => {
                println!("client disconnect");
                if line.will_flag.unwrap() == MqttWillFlag::Enable {
                    let msg = PublishMessage::new(
                        line.will_qos.unwrap(),
                        MqttDup::Disable,
                        line.will_retain.unwrap(),
                        line.will_topic.as_ref().unwrap().to_owned(),
                        0,
                        line.will_message.as_ref().unwrap().to_owned(),
                    );
                    let topic_msg = TopicMessage::Content(line.client_id.as_ref().unwrap().clone(), msg);
                    SUBSCRIPT.broadcast(line.will_topic.as_ref().unwrap(), &topic_msg).await;
                }
                SUBSCRIPT.exit(line.client_id.as_ref().unwrap()).await;
                return Some(MqttMessageV3::Disconnect(DisconnectMessage::default()));
            }
            _ => { return None; }
        }
    }
    None
}


