use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub type Pool = Arc<RwLock<HashMap<Uuid, Connection>>>;

pub struct Connection {
    pub sender: mpsc::UnboundedSender<String>,
}

pub async fn send_all(pool: Pool, msg: String) {
    for connection in pool.read().await.values() {
        connection
            .sender
            .send(msg.clone())
            .expect("failed send to channel");
    }
}
