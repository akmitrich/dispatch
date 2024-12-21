use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Serialize, Deserialize, Clone, FromRow, Debug)]
pub struct ChannelMessage {
    pub from: String,
    pub body: String,
    pub timestamp: i64,
}
impl ChannelMessage {
    pub fn to_string(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}
