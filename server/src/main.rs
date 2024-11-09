mod pool;

use futures_util::{SinkExt, StreamExt};
use pool::{Connection, Pool, SocketReciver, SocketSender};
use std::collections::HashMap;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio::task::AbortHandle;
use tokio_tungstenite::{accept_async, tungstenite::Message};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("localhost:8080")
        .await
        .expect("server: failed bind");
    let pool = Pool::new(RwLock::new(HashMap::new()));
    while let Ok((socket, _)) = listener.accept().await {
        tokio::spawn(handler(pool.clone(), socket));
    }
}

async fn handler(pool: Pool, socket: TcpStream) {
    let ws_stream = accept_async(socket).await.expect("socket: failed accept");
    let (ws_tx, mut ws_rx) = ws_stream.split();
    let (connection, sender) = authorization(
        ws_tx,
        &mut ws_rx
    ).await.expect("handler: failed authorization");
    pool.write().await.insert(connection.id(), connection.clone());
    let msg = format!("{} joined", connection.username());
    pool::send_all(pool.clone(), msg).await;
    while let Some(Ok(Message::Text(msg))) = ws_rx.next().await {
        let msg = format!("{}: {}", connection.username(), msg);
        pool::send_all(pool.clone(), msg).await;
    }
    pool.write().await.remove(&connection.id());
    sender.abort();
    let msg = format!("{} logout", connection.username());
    pool::send_all(pool.clone(), msg).await;
}

async fn authorization(
    mut ws_tx: SocketSender,
    ws_rx: &mut SocketReciver,
) -> Option<(Connection, AbortHandle)> {
    ws_tx
        .send(Message::from("username?"))
        .await
        .expect("authorization: failed send");
    if let Some(Ok(Message::Text(msg))) = ws_rx.next().await {
        let (channel_tx, mut channel_rx) = mpsc::unbounded_channel();
        let sender = tokio::spawn(async move {
            while let Some(msg) = channel_rx.recv().await {
                ws_tx
                    .send(Message::from(msg))
                    .await
                    .expect("failed send to client");
            }
        }).abort_handle();
        return Some((Connection::new(msg, channel_tx), sender));
    }
    None
}
