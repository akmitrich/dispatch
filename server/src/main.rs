mod authrequest;
mod channel;
mod context;
mod response;
mod userconnection;

use anyhow::Result;
use authrequest::AuthRequest;
use context::Context;
use futures_util::{stream::SplitSink, StreamExt};
use jwt_simple::{
    claims::{Claims, NoCustomClaims},
    prelude::{Duration, HS256Key, MACLike},
};
use response::Response;
use std::env;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};
use userconnection::UserConnection;

pub type WebSocketSender = SplitSink<WebSocketStream<TcpStream>, Message>;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let context = Context::create().await?;
    let listener = TcpListener::bind("localhost:3000").await?;
    while let Ok((socket, _)) = listener.accept().await {
        tokio::spawn(handle(context.clone(), socket));
    }
    Ok(())
}

async fn handle(context: Context, socket: TcpStream) -> Result<()> {
    let mut buf = [0; 1024];
    let n = socket.peek(&mut buf).await?;
    let request = String::from_utf8_lossy(&buf[..n]);
    let (headers, body) = request.split_once("\r\n\r\n").unwrap();
    if let Some(line) = headers.lines().next() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let method = parts[0];
        let path = parts[1];
        match (method, path) {
            ("POST", "/signin") => signin(context, socket, body).await?,
            ("POST", "/signup") => signup(context, socket, body).await?,
            ("GET", "/connect") => connect(context, socket, headers).await?,
            _ => {}
        }
    }
    Ok(())
}

async fn signin(context: Context, mut socket: TcpStream, body: &str) -> Result<()> {
    let request: AuthRequest = serde_json::from_str(body)?;
    let query = sqlx::query("SELECT * FROM users WHERE (username = $1) AND (password = $2)")
        .bind(request.username)
        .bind(request.password)
        .execute(&context.db_pool)
        .await?;
    if query.rows_affected() != 0 {
        if !context.contains(request.username).await {
            let key = HS256Key::from_bytes(env::var("SECRET")?.as_bytes());
            let claims = Claims::create(Duration::from_mins(1)).with_subject(request.username);
            let token = key.authenticate(claims)?;
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

async fn signup(context: Context, mut socket: TcpStream, body: &str) -> Result<()> {
    let request: AuthRequest = serde_json::from_str(body)?;
    let query = sqlx::query("SELECT * FROM users WHERE username = $1")
        .bind(request.username)
        .execute(&context.db_pool)
        .await?;
    if query.rows_affected() == 0 {
        socket.write(&Response::new(200, "OK", "Success")).await?;
        sqlx::query("INSERT INTO users (username, password) VALUES ($1, $2)")
            .bind(request.username)
            .bind(request.password)
            .execute(&context.db_pool)
            .await?;
    } else {
        socket
            .write(&Response::new(409, "Conflict", "Already exists"))
            .await?;
    }
    Ok(())
}

async fn connect(mut context: Context, socket: TcpStream, headers: &str) -> Result<()> {
    if headers.contains("Upgrade: websocket") {
        let mut ws_stream = accept_async(socket).await?;
        if let Some(Ok(Message::Text(token))) = ws_stream.next().await {
            let key = HS256Key::from_bytes(env::var("SECRET")?.as_bytes());
            match key.verify_token::<NoCustomClaims>(&token, None) {
                Ok(claims) => {
                    let username = claims.subject.unwrap();
                    let (ws_tx, mut ws_rx) = ws_stream.split();
                    let connection = UserConnection::new(ws_tx).await;
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
