use futures_util::stream::SplitStream;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

async fn handler(mut reader: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>) {
    while let Some(msg) = reader.next().await {
        let msg = msg.expect("failed read message");
        if msg.is_text() {
            print!("{}", msg);
        }
    }
}

#[tokio::main]
async fn main() {
    let (ws_stream, _) = connect_async("ws://localhost:8080")
        .await
        .expect("failed during websocket handshaket");
    let (mut sender, reader) = ws_stream.split();
    tokio::spawn(handler(reader));
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("failed read");
        sender
            .send(Message::from(input))
            .await
            .expect("failed send message");
    }
}
