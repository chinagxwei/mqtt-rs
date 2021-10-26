use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::subscript::ClientID;

pub struct ClientMessageFrames {
    frames: HashMap<u16, MessageFrame>,
}

impl ClientMessageFrames {
    pub fn new() -> ClientMessageFrames {
        ClientMessageFrames { frames: HashMap::default() }
    }

    pub fn append(&mut self, message_id: u16, frame: MessageFrame) -> Option<MessageFrame> {
        self.frames.insert(message_id, frame)
    }

    fn remove(&mut self, message_id: u16) -> Option<MessageFrame> {
        self.frames.remove(&message_id)
    }

    fn complete(&mut self, message_id: u16) {
        if let Some(frame) = self.frames.get_mut(&message_id) {
            frame.complete();
        }
    }
}

pub struct MessageFrame {
    from: ClientID,
    to: ClientID,
    message_id: u16,
    bytes: Vec<u8>,
    send: bool,
}

impl MessageFrame {
    pub fn new(from: ClientID, to: ClientID, bytes: Vec<u8>, message_id: u16) -> Self {
        MessageFrame {
            from,
            to,
            message_id,
            bytes,
            send: false,
        }
    }

    pub fn complete(&mut self) {
        self.send = true
    }
}

pub struct MessageContainer {
    inner: Arc<Mutex<HashMap<ClientID, ClientMessageFrames>>>,
}

impl MessageContainer {
    pub fn new() -> MessageContainer {
        MessageContainer { inner: Arc::new(Mutex::new(HashMap::default())) }
    }

    pub async fn append(&self, client_id: ClientID, message_id: u16, frame: MessageFrame) {
        if self.inner.lock().await.contains_key(&client_id) {
            if let Some(frames) = self.inner.lock().await.get_mut(&client_id) {
                frames.append(message_id, frame);
            }
        } else {
            let mut frames = ClientMessageFrames::new();
            frames.append(message_id, frame);
            self.inner.lock().await.insert(client_id, frames);
        }
    }

    pub async fn complete(&self, client_id: &ClientID, message_id: u16) {
        if let Some(frames) = self.inner.lock().await.get_mut(client_id) {
            frames.complete(message_id)
        }
    }
}
