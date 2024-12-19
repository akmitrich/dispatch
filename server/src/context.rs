use crate::{channel::Channel, channelmessage::ChannelMessage, userconnection::UserConnection};
use anyhow::Result;
use chrono::Utc;
use jwt_simple::prelude::HS256Key;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{collections::HashMap, env, sync::Arc};
use tokio::sync::RwLock;

pub type ConnectionsPool = Arc<RwLock<HashMap<String, UserConnection>>>;

#[derive(Clone)]
pub struct Context {
    key: HS256Key,
    pub pg_pool: PgPool,
    pub channel: Channel,
    pub connections: ConnectionsPool,
}
impl Context {
    pub async fn create() -> Result<Self> {
        let key = HS256Key::from_bytes(env::var("SECRET_KEY")?.as_bytes());
        let database_url = env::var("DATABASE_URL")?;
        let pg_pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(&database_url)
            .await?;
        let connections = Arc::new(RwLock::new(HashMap::new()));
        let channel = Channel::new(pg_pool.clone(), connections.clone()).await?;
        Ok(Self {
            key: key,
            pg_pool: pg_pool,
            channel: channel,
            connections: connections,
        })
    }
    pub fn key(&self) -> &HS256Key {
        &self.key
    }
    pub fn send(&self, from: &str, body: String) -> Result<()> {
        self.channel.send(ChannelMessage {
            from: from.to_string(),
            body: body,
            timestamp: Utc::now().timestamp_millis(),
        })?;
        Ok(())
    }
    pub async fn contains(&self, username: &str) -> bool {
        self.connections.read().await.contains_key(username)
    }
    pub async fn insert(&mut self, username: &str, connection: UserConnection) {
        self.connections
            .write()
            .await
            .insert(username.to_string(), connection);
    }
    pub async fn remove(&mut self, username: &str) {
        self.connections
            .write()
            .await
            .remove(username)
            .unwrap()
            .close();
    }
}
