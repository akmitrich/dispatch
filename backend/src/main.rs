mod authrequest;
mod channel;
mod context;
mod endpoints;
mod response;
mod userconnection;
mod channelmessage;

use std::env;
use anyhow::Result;
use context::Context;
use endpoints::{connect, signin, signup};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::from_path("backend/.env").ok();
    let context = Context::create().await?;
    let listener = TcpListener::bind(env::var("ADDRESS")?).await?;
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
