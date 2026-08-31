use crate::game::GameHistory;

pub fn to_json(history: &GameHistory) -> Result<String, serde_json::Error> {
    serde_json::to_string(history)
}

pub fn from_json(payload: &str) -> Result<GameHistory, serde_json::Error> {
    serde_json::from_str(payload)
}

pub fn to_bytes(history: &GameHistory) -> Result<Vec<u8>, bincode::Error> {
    bincode::serialize(history)
}

pub fn from_bytes(payload: &[u8]) -> Result<GameHistory, bincode::Error> {
    bincode::deserialize(payload)
}
