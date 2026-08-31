pub mod debug;
pub mod game;
pub mod persistence;
pub mod types;

pub use debug::{DebugOptions, debug_engine, debug_engine_basic};
pub use game::{GameEngine, GameError, GameHistory};
pub use persistence::{from_bytes, from_json, to_bytes, to_json};
pub use types::{
    Change, GameConfig, Move, Ownership, PlayerId, Point, PointState, ScoringMode,
};

#[cfg(test)]
mod lib_test;
