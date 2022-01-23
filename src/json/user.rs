use serde::Deserialize;

#[derive(Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub role: u8,
}
