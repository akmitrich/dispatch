use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AuthRequest<'a> {
    pub username: &'a str,
    pub password: &'a str,
}
impl<'a> AuthRequest<'a> {
    pub fn is_empty(&self) -> bool {
        self.username.is_empty() || self.password.is_empty()
    }
}
