use crate::{
    authrequest::AuthRequest, channel::ChannelMessage, context::Context, response::Response,
    userconnection::UserConnection,
};
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use jwt_simple::{
    claims::{Claims, NoCustomClaims},
    prelude::{Duration, MACLike},
};
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};

pub async fn signin(context: Context, mut socket: TcpStream, body: &str) -> Result<()> {
    let userdata: AuthRequest = serde_json::from_str(body)?;
    let query = sqlx::query("SELECT * FROM users WHERE (username = $1) AND (password = $2)")
        .bind(userdata.username)
        .bind(userdata.password)
        .execute(&context.pg_pool)
        .await?;
    if query.rows_affected() != 0 {
        if !context.contains(userdata.username).await {
            let claims = Claims::create(Duration::from_mins(1)).with_subject(userdata.username);
            let token = context.key().authenticate(claims)?;
            socket.write(&Response::new(200, "OK", &token)).await?;
        } else {
            socket
                .write(&Response::new(409, "Conflict", "Already in"))
                .await?;
        }
    } else {
        socket
            .write(&Response::new(
                401,
                "Unauthorized",
                "Wrong username/password",
            ))
            .await?;
    }
    Ok(())
}

pub async fn signup(context: Context, mut socket: TcpStream, body: &str) -> Result<()> {
    let userdata: AuthRequest = serde_json::from_str(body)?;
    let query = sqlx::query("SELECT * FROM users WHERE username = $1")
        .bind(userdata.username)
        .execute(&context.pg_pool)
        .await?;
    if query.rows_affected() == 0 {
        socket.write(&Response::new(200, "OK", "Success")).await?;
        sqlx::query("INSERT INTO users (username, password) VALUES ($1, $2)")
            .bind(userdata.username)
            .bind(userdata.password)
            .execute(&context.pg_pool)
            .await?;
    } else {
        socket
            .write(&Response::new(409, "Conflict", "Already exists"))
            .await?;
    }
    Ok(())
}

pub async fn connect(mut context: Context, socket: TcpStream, headers: &str) -> Result<()> {
    if headers.contains("Upgrade: websocket") {
        let mut ws_stream = accept_async(socket).await?;
        if let Some(Ok(Message::Text(token))) = ws_stream.next().await {
            match context.key().verify_token::<NoCustomClaims>(&token, None) {
                Ok(claims) => {
                    let username = claims.subject.unwrap();
                    let (ws_tx, mut ws_rx) = ws_stream.split();
                    let connection = UserConnection::new(ws_tx).await;
                    let preload = sqlx::query_as::<_, ChannelMessage>(
                        "SELECT \"from\", \"body\", gate_timestamp FROM messages",
                    )
                    .fetch_all(&context.pg_pool)
                    .await?;
                    preload.iter().for_each(|msg| connection.send(msg.clone()));
                    context.insert(&username, connection).await;
                    while let Some(Ok(Message::Text(msg))) = ws_rx.next().await {
                        context.send(&username, msg)?
                    }
                    context.remove(&username).await;
                }
                Err(_) => ws_stream.close(None).await?,
            }
        }
    }
    Ok(())
}
