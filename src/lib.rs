pub mod debug;
pub mod game;
pub mod history;
pub mod persistence;
pub mod types;

pub use debug::{DebugSnapshot, debug_snapshot, debug_state_json};
pub use game::{GameEngine, GameError, GameState, MoveOutcome};
pub use history::History;
pub use persistence::{from_bytes, from_json, to_bytes, to_json};
pub use types::{Edge, GameConfig, Move, PlayerId, Point};

#[cfg(test)]
mod lib_test;
