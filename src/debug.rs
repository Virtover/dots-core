use crate::game::GameState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebugSnapshot {
    pub turn: u32,
    pub current_player: u8,
    pub edges_count: usize,
    pub blocked_dots_count: usize,
    pub state_json: String,
}

pub fn debug_snapshot(state: &GameState) -> DebugSnapshot {
    DebugSnapshot {
        turn: state.turn,
        current_player: state.current_player,
        edges_count: state.edges.len(),
        blocked_dots_count: state.blocked_dots.len(),
        state_json: serde_json::to_string_pretty(state)
            .unwrap_or_else(|_| "{\"error\":\"failed to serialize game state\"}".to_string()),
    }
}

pub fn debug_state_json(state: &GameState) -> String {
    serde_json::to_string_pretty(state)
        .unwrap_or_else(|_| "{\"error\":\"failed to serialize game state\"}".to_string())
}
