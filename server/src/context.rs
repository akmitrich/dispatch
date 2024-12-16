use crate::{
    channel::{Channel, ChannelMessage},
    userconnection::UserConnection,
};
use anyhow::Result;
use jwt_simple::prelude::HS256Key;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{collections::HashMap, env, sync::Arc};
use tokio::sync::RwLock;

pub type ConnectionsPool = Arc<RwLock<HashMap<String, UserConnection>>>;

#[derive(Clone)]
pub struct Context {
    key: HS256Key,
    pub db_pool: PgPool,
    pub channel: Channel,
    pub conn_pool: ConnectionsPool,
}
impl Context {
    pub async fn create() -> Result<Self> {
        let key = HS256Key::from_bytes(env::var("SECRET_KEY")?.as_bytes());
        let database_url = env::var("DATABASE_URL")?;
        let db_pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(&database_url)
            .await?;
        let conn_pool = Arc::new(RwLock::new(HashMap::new()));
        let channel = Channel::new(&database_url, conn_pool.clone()).await?;
        Ok(Self {
            key: key,
            db_pool: db_pool,
            channel: channel,
            conn_pool: conn_pool,
        })
    }
    pub fn key(&self) -> &HS256Key { &self.key }
    pub fn send(&self, from: &str, body: String) -> Result<()> {
        self.channel.send(ChannelMessage {
            from: from.to_string(),
            body: body,
        })?;
        Ok(())
    }
    pub async fn contains(&self, username: &str) -> bool {
        self.conn_pool.read().await.contains_key(username)
    }
    pub async fn insert(&mut self, username: &str, connection: UserConnection) {
        self.conn_pool
            .write()
            .await
            .insert(username.to_string(), connection);
    }
    pub async fn remove(&mut self, username: &str) {
        self.conn_pool
            .write()
            .await
            .remove(username)
            .unwrap()
            .close();
    }
}
