use crate::history::History;
use crate::types::{Edge, GameConfig, Move, PlayerId, Point};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameState {
    pub config: GameConfig,
    pub current_player: PlayerId,
    pub turn: u32,
    pub edges: BTreeSet<(Point, Point)>,
    pub blocked_dots: BTreeSet<Point>,
}

impl GameState {
    pub fn new(config: GameConfig) -> Self {
        Self {
            config,
            current_player: 0,
            turn: 0,
            edges: BTreeSet::new(),
            blocked_dots: BTreeSet::new(),
        }
    }

    pub fn legal_moves(&self) -> Vec<Edge> {
        let mut result = Vec::new();

        for y in 0..self.config.height {
            for x in 0..self.config.width {
                let p = Point::new(x, y);

                if x + 1 < self.config.width {
                    self.push_if_legal(p, Point::new(x + 1, y), &mut result);
                }

                if y + 1 < self.config.height {
                    self.push_if_legal(p, Point::new(x, y + 1), &mut result);
                }
            }
        }

        result
    }

    pub fn apply_move(&mut self, mv: Move) -> Result<MoveOutcome, GameError> {
        if mv.player_id != self.current_player {
            return Err(GameError::NotCurrentPlayer {
                expected: self.current_player,
                got: mv.player_id,
            });
        }

        self.validate_edge(mv.edge)?;

        let key = normalize_edge(mv.edge);
        if self.edges.contains(&key) {
            return Err(GameError::EdgeAlreadyUsed);
        }

        self.edges.insert(key);
        self.turn += 1;
        self.current_player = (self.current_player + 1) % self.config.players.max(1);

        Ok(MoveOutcome {
            turn: self.turn,
            next_player: self.current_player,
        })
    }

    pub fn is_dot_playable(&self, dot: Point) -> bool {
        !self.blocked_dots.contains(&dot)
    }

    pub fn mark_surrounded_dots<I>(&mut self, dots: I)
    where
        I: IntoIterator<Item = Point>,
    {
        self.blocked_dots.extend(dots);
    }

    pub fn clear_surrounded_dots(&mut self) {
        self.blocked_dots.clear();
    }

    fn validate_edge(&self, edge: Edge) -> Result<(), GameError> {
        let (a, b) = (edge.a, edge.b);

        if !self.in_bounds(a) || !self.in_bounds(b) {
            return Err(GameError::OutOfBounds);
        }

        if self.blocked_dots.contains(&a) || self.blocked_dots.contains(&b) {
            return Err(GameError::BlockedDotUsed);
        }

        let dx = a.x.abs_diff(b.x);
        let dy = a.y.abs_diff(b.y);
        if (dx == 1 && dy == 0) || (dx == 0 && dy == 1) {
            Ok(())
        } else {
            Err(GameError::NonAdjacentDots)
        }
    }

    fn in_bounds(&self, p: Point) -> bool {
        p.x < self.config.width && p.y < self.config.height
    }

    fn push_if_legal(&self, a: Point, b: Point, out: &mut Vec<Edge>) {
        if self.blocked_dots.contains(&a) || self.blocked_dots.contains(&b) {
            return;
        }

        let edge = Edge::new(a, b);
        if !self.edges.contains(&normalize_edge(edge)) {
            out.push(edge);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveOutcome {
    pub turn: u32,
    pub next_player: PlayerId,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum GameError {
    #[error("the move belongs to another player (expected {expected}, got {got})")]
    NotCurrentPlayer { expected: PlayerId, got: PlayerId },
    #[error("edge is already used")]
    EdgeAlreadyUsed,
    #[error("edge contains a blocked (surrounded) dot")]
    BlockedDotUsed,
    #[error("edge points must be adjacent")]
    NonAdjacentDots,
    #[error("edge is out of board bounds")]
    OutOfBounds,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameEngine {
    pub history: History<GameState>,
}

impl GameEngine {
    pub fn new(config: GameConfig) -> Self {
        Self {
            history: History::new(GameState::new(config)),
        }
    }

    pub fn state(&self) -> &GameState {
        &self.history.present
    }

    pub fn apply_move(&mut self, mv: Move) -> Result<MoveOutcome, GameError> {
        let mut next = self.history.present.clone();
        let outcome = next.apply_move(mv)?;
        self.history.apply(next);
        Ok(outcome)
    }

    pub fn undo(&mut self) -> bool {
        self.history.undo()
    }

    pub fn redo(&mut self) -> bool {
        self.history.redo()
    }

    pub fn mark_surrounded_dots<I>(&mut self, dots: I)
    where
        I: IntoIterator<Item = Point>,
    {
        let mut next = self.history.present.clone();
        next.mark_surrounded_dots(dots);
        self.history.apply(next);
    }
}

fn normalize_edge(edge: Edge) -> (Point, Point) {
    if (edge.a.x, edge.a.y) <= (edge.b.x, edge.b.y) {
        (edge.a, edge.b)
    } else {
        (edge.b, edge.a)
    }
}
