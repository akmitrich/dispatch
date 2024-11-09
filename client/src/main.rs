use futures_util::{SinkExt, StreamExt};
use std::io::Write;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() {
    let (ws_stream, _) = connect_async("ws://localhost:8080")
        .await
        .expect("failed connect");
    let (mut tx, mut rx) = ws_stream.split();
    tokio::spawn(async move {
        loop {
            tx.send(Message::from(input()))
                .await
                .expect("failed send");
        }
    });
    while let Some(Ok(Message::Text(msg))) = rx.next().await {
        println!("{}", msg);
    }
}

fn input() -> String {
    std::io::stdout().flush().expect("failed flush");
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("falied read line");
    input.trim().to_string()
}
