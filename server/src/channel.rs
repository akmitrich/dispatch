use crate::context::ConnectionsPool;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{Connection, PgConnection};
use tokio::sync::mpsc;

pub type ChannelSender = mpsc::UnboundedSender<ChannelMessage>;

#[derive(Serialize, Deserialize, Clone)]
pub struct ChannelMessage {
    pub from: String,
    pub body: String,
}
impl ChannelMessage {
    pub fn to_string(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

#[derive(Clone)]
pub struct Channel {
    sender: ChannelSender,
}
impl Channel {
    pub async fn new(database_url: &str, connections: ConnectionsPool) -> Result<Self> {
        let mut pg_connection = PgConnection::connect(&database_url).await?;
        let (sender, mut receiver) = mpsc::unbounded_channel::<ChannelMessage>();
        tokio::spawn(async move {
            while let Some(msg) = receiver.recv().await {
                for (_, connection) in connections.read().await.iter() {
                    connection.send(msg.clone());
                }
                // sqlx::query("INSERT INTO chat_history (\"from\", \"body\") VALUES ($1, $2)")
                //     .bind(&msg.from)
                //     .bind(&msg.body)
                //     .execute(&mut pg_connection)
                //     .await;
            }
        });
        Ok(Self { sender: sender })
    }
    pub fn send(&self, msg: ChannelMessage) -> Result<()> {
        self.sender.send(msg)?;
        Ok(())
    }
}
