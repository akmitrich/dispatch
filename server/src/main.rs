mod pool;

use futures_util::{SinkExt, StreamExt};
use pool::Pool;
use std::collections::HashMap;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("localhost:8080")
        .await
        .expect("failed bind");
    let pool = Pool::new(RwLock::new(HashMap::new()));
    while let Ok((socket, _)) = listener.accept().await {
        tokio::spawn(handler(pool.clone(), socket));
    }
}

async fn handler(pool: Pool, socket: TcpStream) {
    let ws_stream = accept_async(socket).await.expect("failed accept");
    let (mut ws_tx, mut ws_rx) = ws_stream.split();
    let (tx, mut rx) = mpsc::unbounded_channel();
    let id = Uuid::new_v4();
    pool.write()
        .await
        .insert(id, pool::Connection { sender: tx });
    let ws_sender = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            ws_tx
                .send(Message::from(msg))
                .await
                .expect("falied send to client");
        }
    }).abort_handle();
    while let Some(Ok(Message::Text(msg))) = ws_rx.next().await {
        let msg = format!("{}: {}", id, msg);
        pool::send_all(pool.clone(), msg).await;
    }
    pool.write().await.remove(&id);
    ws_sender.abort();
    let msg = format!("{} disconnect", id);
    pool::send_all(pool.clone(), msg).await;
}
