use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiUserResponse {
    pub data: ApiUserObject
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiUserObject {
    pub uuid: String,
    pub name: Option<String>,
    pub steamId: u64,
    pub discordId: u64,
    pub gmodStoreId: Option<String>,
    pub avatar: Option<String>,
    created_at: String,
    updated_at: String,
}