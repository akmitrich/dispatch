use tokio::sync::mpsc;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub type ChannelSender = mpsc::UnboundedSender<String>;
pub type ChannelReceiver = mpsc::UnboundedReceiver<String>;

pub async fn connect(url: &str) -> (ChannelSender, ChannelReceiver) {
    let (ws_stream, _) = connect_async(url).await.expect("failed connect");
    let (mut ws_tx, mut ws_rx) = ws_stream.split();
    let (client_tx, mut client_rx) = mpsc::unbounded_channel::<String>();
    tokio::spawn(async move {
        while let Some(msg) = client_rx.recv().await {
            ws_tx.send(Message::from(msg)).await.expect("failed send");
        }
    });
    let (socket_tx, socket_rx) = mpsc::unbounded_channel::<String>();
    tokio::spawn(async move {
        while let Some(Ok(Message::Text(msg))) = ws_rx.next().await {
            socket_tx.send(msg).expect("failed send");
        }
    });
    (client_tx, socket_rx)
}
