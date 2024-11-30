mod connection;

use connection::{Connection, Pool, SocketReciver, SocketSender};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::collections::HashMap;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio::task::AbortHandle;
use tokio_tungstenite::{accept_async, tungstenite::Message};

#[tokio::main]
async fn main() {
    let addr = "localhost:8080";
    let listener = TcpListener::bind(addr).await.expect("server: failed bind");
    let pool = Pool::new(RwLock::new(HashMap::new()));
    while let Ok((socket, _)) = listener.accept().await {
        tokio::spawn(handler(pool.clone(), socket));
    }
}

async fn handler(pool: Pool, socket: TcpStream) {
    let ws_stream = accept_async(socket).await.expect("socket: failed accept");
    let (ws_tx, mut ws_rx) = ws_stream.split();
    match authorization(ws_tx, &mut ws_rx).await {
        Some((connection, sender_aborthandle)) => {
            pool.write()
                .await
                .insert(connection.id(), connection.clone());
            let msg = format!("{} joined", connection.username());
            Connection::send_all(pool.clone(), msg).await;
            while let Some(Ok(Message::Text(msg))) = ws_rx.next().await {
                let msg = format!("{}: {}", connection.username(), msg);
                Connection::send_all(pool.clone(), msg).await;
            }
            pool.write().await.remove(&connection.id());
            sender_aborthandle.abort();
            let msg = format!("{} logout", connection.username());
            Connection::send_all(pool.clone(), msg).await;
        }
        None => return,
    }
}

#[derive(Deserialize)]
struct Userdata {
    username: String,
    password: String,
}

impl From<String> for Userdata {
    fn from(value: String) -> Self {
        serde_json::from_str::<Self>(&value).expect("failed userdata")
    }
}

async fn authorization(
    mut ws_tx: SocketSender,
    ws_rx: &mut SocketReciver,
) -> Option<(Connection, AbortHandle)> {
    match ws_rx.next().await {
        Some(Ok(Message::Text(msg))) => {
            println!("{}", msg);
            let userdata = Userdata::from(msg);
            let (channel_tx, mut channel_rx) = mpsc::unbounded_channel();
            ws_tx
                .send(Message::from("@connected"))
                .await
                .expect("authorization: failed send");
            let sender_aborthandle = tokio::spawn(async move {
                while let Some(msg) = channel_rx.recv().await {
                    ws_tx
                        .send(Message::from(msg))
                        .await
                        .expect("failed send to client");
                }
            })
            .abort_handle();
            Some((Connection::new(userdata.username, channel_tx), sender_aborthandle))
        }
        _ => None,
    }
}
