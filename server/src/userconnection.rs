use crate::{
    channel::{ChannelMessage, ChannelSender},
    WebSocketSender,
};
use futures_util::SinkExt;
use tokio::{sync::mpsc, task::AbortHandle};
use tokio_tungstenite::tungstenite::Message;

pub struct UserConnection {
    sender: ChannelSender,
    aborthandle: AbortHandle,
}

impl UserConnection {
    pub async fn new(mut ws_tx: WebSocketSender) -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel::<ChannelMessage>();
        let aborthandle = tokio::spawn(async move {
            while let Some(msg) = receiver.recv().await {
                let msg = msg.to_string().unwrap();
                ws_tx.send(Message::from(msg)).await.unwrap();
            }
        }).abort_handle();
        Self {
            sender: sender,
            aborthandle: aborthandle,
        }
    }
    pub fn send(&self, msg: ChannelMessage) {
        let err = format!("failed to send to {}", msg.from);
        self.sender.send(msg).expect(&err);
    }
    pub fn abort(&self) {
        self.aborthandle.abort();
    }
}
