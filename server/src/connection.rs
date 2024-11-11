use futures_util::stream::{SplitSink, SplitStream};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::{
    net::TcpStream,
    sync::{mpsc, RwLock},
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use uuid::Uuid;

pub type Pool = Arc<RwLock<HashMap<Uuid, Connection>>>;
pub type SocketSender = SplitSink<WebSocketStream<TcpStream>, Message>;
pub type SocketReciver = SplitStream<WebSocketStream<TcpStream>>;
pub type ChannelSender = mpsc::UnboundedSender<String>;

#[derive(Clone)]
pub struct Connection {
    id: Uuid,
    username: String,
    channel_sender: ChannelSender,
}

impl Connection {
    pub fn new(username: String, sender: ChannelSender) -> Self {
        Connection {
            id: Uuid::new_v4(),
            username: username,
            channel_sender: sender
        }
    }
    pub fn id(&self) -> Uuid { self.id }
    pub fn username(&self) -> String { self.username.clone() }
    pub fn channel_sender(&self) -> ChannelSender { self.channel_sender.clone()  } 
    pub async fn send_all(pool: Pool, msg: String) {
        for connection in pool.read().await.values() {
            connection
                .channel_sender()
                .send(msg.clone())
                .expect("failed send to channel");
        }
    }
}