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
    pub point: Point,
}

impl Move {
    pub fn new(player_id: PlayerId, point: Point) -> Self {
        Self { player_id, point }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScoringMode {
    Dots,
    Territory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameConfig {
    pub width: u16,
    pub height: u16,
    pub initial_central_dots: bool,
    pub scoring_mode: ScoringMode,
}

impl GameConfig {
    pub const fn new(
        width: u16,
        height: u16,
        initial_central_dots: bool,
        scoring_mode: ScoringMode,
    ) -> Self {
        assert!(
            width > 4 && height > 4,
            "GameConfig dimensions must be greater than 4x4"
        );
        Self {
            width,
            height,
            initial_central_dots,
            scoring_mode,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ownership {
    None,
    Player(PlayerId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PointState {
    pub ownership: Ownership,
    pub blocked_by: Ownership,
    pub is_edge: bool,
}

impl PointState {
    pub fn new() -> Self {
        Self {
            ownership: Ownership::None,
            blocked_by: Ownership::None,
            is_edge: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Change {
    pub mv: Move,
    pub who_surrounded: Ownership,
    pub surrounded_points: Vec<Point>,
    pub edge_points_added: Vec<Point>,
    pub unsurrounded_points: Vec<Point>,
    pub edge_points_removed: Vec<Point>,
    pub score_changes: [i32; 2], // (player0_score_change, player1_score_change)
}

impl Change {
    pub fn new(
        mv: Move,
        who_surrounded: Ownership,
        surrounded_points: Vec<Point>,
        edge_points_added: Vec<Point>,
        unsurrounded_points: Vec<Point>,
        edge_points_removed: Vec<Point>,
        score_changes: [i32; 2],
    ) -> Self {
        Self {
            mv,
            who_surrounded,
            surrounded_points,
            edge_points_added,
            unsurrounded_points,
            edge_points_removed,
            score_changes,
        }
    }
}
