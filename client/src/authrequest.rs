use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AuthRequest<'a> {
    pub username: &'a str,
    pub password: &'a str,
}
impl<'a> AuthRequest<'a> {
    pub fn is_valid(&self) -> bool {
        let is_empty = self.username.is_empty() || self.password.is_empty();
        let correct_length = self.username.len() <= 8 && self.password.len() <= 8;
        let is_ascii = self.username.is_ascii() && self.password.is_ascii();
        !is_empty && correct_length && is_ascii
    }
}
