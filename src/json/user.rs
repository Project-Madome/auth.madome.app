use serde::Deserialize;

#[cfg_attr(test, derive(Clone))]
#[derive(Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub role: u8,
}
