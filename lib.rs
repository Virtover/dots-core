pub mod debug;
pub mod game;
pub mod persistence;
pub mod types;

// pub use debug::{DebugSnapshot, debug_snapshot, debug_state_json};
pub use game::{GameEngine, GameError, GameHistory};
pub use persistence::{from_bytes, from_json, to_bytes, to_json};
pub use types::{GameConfig, Move, PlayerId, Point, Change, Ownership, PointState, ScoringMode};

#[cfg(test)]
mod lib_test;
