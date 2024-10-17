use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;

async fn handler(stream: TcpStream) {
    let ws_stream = accept_async(stream)
        .await
        .expect("error during websocket handshake");
    let (mut sender, mut reader) = ws_stream.split();
    while let Some(msg) = reader.next().await {
        let msg = msg.expect("failed read message");
        if msg.is_text() {
            print!("{}", msg);
            sender.send(msg).await.expect("failed send message");
        }
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("localhost:8080")
        .await
        .expect("failed bind");
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handler(stream));
    }
}
