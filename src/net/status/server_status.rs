use uuid::Uuid;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerInfo {
    pub version: Version,
    pub players: Players,
    pub description: Value,
    #[serde(skip_serializing_if="Option::is_none")]
    pub favicon: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Version {
    pub name: String,
    pub protocol: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Players {
    pub max: u32,
    pub online: u32,
    #[serde(skip_serializing_if="Option::is_none")]
    pub sample: Option<Vec<Player>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    id: Uuid,
    name: String,
}
