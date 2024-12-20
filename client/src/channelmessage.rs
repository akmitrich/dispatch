use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChannelMessage {
    pub from: String,
    pub body: String,
    pub timestamp: i64,
}
