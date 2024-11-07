use futures_util::stream::SplitStream;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

type Receiver = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

#[tokio::main]
async fn main() {
    let (ws_stream, _) = connect_async("ws://localhost:8080")
        .await
        .expect("failed connect");
    let (mut tx, rx) = ws_stream.split();
    tokio::spawn(handler(rx));
    loop {
        tx.send(Message::from(readline()))
            .await
            .expect("failed send");
    }
}

fn readline() -> String {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("falied readline");
    input
        .strip_suffix("\r\n")
        .or(input.strip_suffix("\n"))
        .expect("failed readline").to_string()
}

async fn handler(mut rx: Receiver) {
    while let Some(msg) = rx.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("{}", text);
            }
            _ => return
        }
    }
}
