use futures_util::{SinkExt, StreamExt};
use tokio::{net::TcpListener, sync::broadcast};
use tokio_tungstenite::accept_async;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("localhost:8080")
        .await
        .expect("failed bind");
    let (tx, _) = broadcast::channel(100);
    while let Ok((socket, _)) = listener.accept().await {
        let tx = tx.clone();
        let mut rx = tx.subscribe();
        tokio::spawn(async move {
            let ws_stream = accept_async(socket).await.expect("failed accept");
            let (mut ws_tx, mut ws_rx) = ws_stream.split();
            let transmiter = tokio::spawn(async move {
                while let Ok(msg) = rx.recv().await {
                    ws_tx.send(msg).await.expect("falied send to client");
                }
            });
            while let Some(Ok(msg)) = ws_rx.next().await {
                tx.send(msg).expect("failed send to channel");
            }
            transmiter.abort();
        });
    }
}
