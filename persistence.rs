use crate::game::{GameHistory, GameEngine};

pub fn to_json(game: &GameEngine) -> Result<String, serde_json::Error> {
    serde_json::to_string(&game.history())
}

pub fn from_json(payload: &str, view_only: bool) -> Result<GameEngine, serde_json::Error> {
    let history: GameHistory = serde_json::from_str(payload)?;
    Ok(GameEngine::from_history(history, view_only))
}

pub fn to_bytes(game: &GameEngine) -> Result<Vec<u8>, bincode::Error> {
    bincode::serialize(&game.history())
}

pub fn from_bytes(payload: &[u8], view_only: bool) -> Result<GameEngine, bincode::Error> {
    let history: GameHistory = bincode::deserialize(payload)?;
    Ok(GameEngine::from_history(history, view_only))
}
