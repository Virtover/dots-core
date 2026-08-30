use serde::{Deserialize, Serialize};

pub type PlayerId = u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

impl Point {
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Edge {
    pub a: Point,
    pub b: Point,
}

impl Edge {
    pub fn new(a: Point, b: Point) -> Self {
        if (a.x, a.y) <= (b.x, b.y) {
            Self { a, b }
        } else {
            Self { a: b, b: a }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Move {
    pub player_id: PlayerId,
    pub edge: Edge,
}

impl Move {
    pub fn new(player_id: PlayerId, a: Point, b: Point) -> Self {
        Self {
            player_id,
            edge: Edge::new(a, b),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameConfig {
    pub width: u16,
    pub height: u16,
    pub players: u8,
}

impl GameConfig {
    pub const fn new(width: u16, height: u16, players: u8) -> Self {
        Self {
            width,
            height,
            players,
        }
    }
}
