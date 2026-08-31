use crate::types::{GameConfig, Edge, Ownership, PointState, Change, Move, PlayerId, Point, ScoringMode};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SChange {
    pub surrounded_points: BTreeSet<Point>,
    pub edges_added: BTreeSet<Edge>,
    pub unsurrounded_points: BTreeSet<Point>,
    pub edges_removed: BTreeSet<Edge>,
    pub score_changes: (i32, i32), // (player0_score_change, player1_score_change)
}

impl SChange {
    pub fn merge(changes: impl IntoIterator<Item = SChange>) -> Self {
        let mut result = Self {
            surrounded_points: BTreeSet::new(),
            edges_added: BTreeSet::new(),
            unsurrounded_points: BTreeSet::new(),
            edges_removed: BTreeSet::new(),
            score_changes: (0, 0),
        };

        for change in changes {
            result.surrounded_points.extend(change.surrounded_points);
            result.edges_added.extend(change.edges_added);
            result.unsurrounded_points.extend(change.unsurrounded_points);
            result.edges_removed.extend(change.edges_removed);
            result.score_changes.0 += change.score_changes.0;
            result.score_changes.1 += change.score_changes.1;
        }

        result
    }

    pub fn to_change(self, mv: Move, who_surrounded: Ownership) -> Change {
        Change {
            mv,
            who_surrounded,
            surrounded_points: self.surrounded_points.into_iter().collect(),
            edges_added: self.edges_added.into_iter().collect(),
            unsurrounded_points: self.unsurrounded_points.into_iter().collect(),
            edges_removed: self.edges_removed.into_iter().collect(),
            score_changes: self.score_changes,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.surrounded_points.is_empty()
            && self.edges_added.is_empty()
            && self.unsurrounded_points.is_empty()
            && self.edges_removed.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameHistory {
    pub config: GameConfig,
    pub moves: Vec<Change>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum GameError {
    #[error("the move belongs to another player (expected {expected}, got {got})")]
    NotCurrentPlayer { expected: PlayerId, got: PlayerId },
    #[error("point is already occupied")]
    PointOccupied,
    #[error("point is blocked (surrounded)")]
    PointBlocked,
    #[error("edge is out of board bounds")]
    OutOfBounds,
    #[error("game is in view-only mode")]
    GameIsViewOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameEngine {
    pub config: GameConfig,
    pub turn: u32,
    pub current_player: PlayerId,
    pub scores: (i32, i32), // (player0_score, player1_score)
    pub edges: BTreeSet<Edge>,
    pub board_state: Vec<Vec<PointState>>,
    pub past: Vec<Change>,
    pub future: Vec<Change>,
    pub view_only: bool,
}

impl GameEngine {
    pub fn new(config: GameConfig) -> Self {
        let board_state = vec![vec![PointState::new(); config.width as usize]; config.height as usize];
        if config.initial_central_dots {
            let center_x = config.width / 2;
            let center_y = config.height / 2;
            for y in (center_y - 1)..=(center_y) {
                for x in (center_x - 1)..=(center_x) {
                    board_state[y as usize][x as usize].ownership = Ownership::Player((x + y) as u8 % 2); 
                }
            }
        }
        Self {
            config,
            turn: 0,
            current_player: 0,
            player0_score: 0,
            player1_score: 0,
            edges: BTreeSet::new(),
            board_state,
            past: Vec::new(),
            future: Vec::new(),
            view_only: false,
        }
    }

    pub fn from_history(history: GameHistory, view_only: bool) -> Self {
        let mut engine = GameEngine::new(history.config.clone());
        engine.future = history.moves.clone();
        engine.view_only = view_only;
        engine
    }

    pub fn history(&self) -> GameHistory {
        GameHistory {
            config: self.config.clone(),
            moves: [self.past.clone(), self.future.clone()].concat(),
        }
    }

    fn is_surrounding_possible(&self, last_placed_dot: &Point) -> bool {
        assert!(last_placed_dot.x < self.config.width && last_placed_dot.y < self.config.height, "Point is out of bounds");
        let who_may_surround = &self.board_state[last_placed_dot.y as usize][last_placed_dot.x as usize].ownership;
        if who_may_surround == &Ownership::None { return false; }
        // todo: explore point states in proper directions
        true
    }

    fn get_neighbours(&self, point: &Point) -> Vec<Point> {
        let mut neighbours = Vec::new();
        let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        for (dx, dy) in directions.iter() {
            let new_x = point.x as i32 + dx;
            let new_y = point.y as i32 + dy;
            if new_x >= 0 && new_x < self.config.width as i32 && new_y >= 0 && new_y < self.config.height as i32 {
                neighbours.push(Point::new(new_x as u16, new_y as u16));
            }
        }
        neighbours
    }

    fn get_schange(&self, point: &Point, who_may_surround: Ownership) -> SChange {
        // bfs: who_may_surround dots are border
        //      store border points in BTreeSet and possibly surrounded points in another BTreeSet
        //      if reached map boundaries - not surrounded
        //      else, after bfs:
        //        if there were no point of opponent to who_may_surround - not surrounded
        //        if there were point of opponent to who_may_surround - surrounded by who_may_surround
        //          in that case, start bfs: starting set is all boundary points of board
        //             previously stored border points reached in seocond bfs are points that form edges of surrounded area
        //             store outer-points, edge-points in separate BTreeSets
        //             remaining points are surrounded points (surrounded_points)
        //             store all edges & points surrounded by another player than who_may_surround in surrounded area in separate unsurrounded_points, edges_removed BTreeSets
        //             form edges from points stored in edge-points
        //             return SChange with all these sets; remember to store score changes in score_changes field, depending on scoring mode (dots or territory)
        // todo
        SChange {
            surrounded_points: BTreeSet::new(),
            edges_added: BTreeSet::new(),
            unsurrounded_points: BTreeSet::new(),
            edges_removed: BTreeSet::new(),
            score_changes: (0, 0),
        }
    }

    fn apply_change(&mut self, change: &Change) {
        let mv = &change.mv;
        self.board_state[mv.point.y as usize][mv.point.x as usize].ownership = Ownership::Player(mv.player_id);

        for point in &change.surrounded_points {
            self.board_state[point.y as usize][point.x as usize].blocked_by = Ownership::Player(mv.player_id);
        }
        for point in &change.unsurrounded_points {
            self.board_state[point.y as usize][point.x as usize].blocked_by = Ownership::None;
        }
        for edge in &change.edges_added { self.edges.insert(edge.clone()); }
        for edge in &change.edges_removed { self.edges.remove(edge);}
        for edge in &change.edges_added { self.edges.insert(edge.clone()); }
        for edge in &change.edges_removed { self.edges.remove(edge); }
    }

    pub fn apply_move(&mut self, mv: Move) -> Result<Change, GameError> {
        if self.view_only {
            return Err(GameError::GameIsViewOnly);
        }

        if mv.player_id != self.current_player {
            return Err(GameError::NotCurrentPlayer {
                expected: self.current_player,
                got: mv.player_id,
            });
        }

        let point_state = &self.board_state[mv.point.y as usize][mv.point.x as usize];
        if point_state.ownership != Ownership::None {
            return Err(GameError::PointOccupied);
        }
        if point_state.blocked_by != Ownership::None {
            return Err(GameError::PointBlocked);
        }

        let mut change = Change {
            mv,
            who_surrounded: Ownership::None,
            surrounded_points: Vec::new(),
            edges_added: Vec::new(),
            unsurrounded_points: Vec::new(),
            edges_removed: Vec::new(),
            score_changes: (0, 0),
        };
        if !self.is_surrounding_possible(&mv.point) { return Ok(change); } 

        self.board_state[mv.point.y as usize][mv.point.x as usize].ownership = Ownership::Player(mv.player_id);
        let neighbours = self.get_neighbours(&mv.point);
        let schange = SChange::merge(neighbours.iter().map(|n| self.get_schange(n, Ownership::Player(mv.player_id))));
        if !schange.is_empty() {
            change = schange.to_change(mv, Ownership::Player(mv.player_id));
        } else {
            change = self.get_schange(&mv.point, Ownership::Player((mv.player_id + 1) % 2));
        }

        apply_change(self, &change);

        self.past.push(change.clone());
        self.future.clear();
        self.turn += 1;
        self.current_player = (self.current_player + 1) % 2; // Assuming 2 players

        Ok(change)
    }

    pub fn undo(&mut self) -> bool {
        if let Some(change) = self.past.pop() {
            self.apply_change(&change); // Apply the change in reverse
            self.future.push(change);
            self.turn -= 1;
            self.current_player = (self.current_player + 1) % 2; // Switch back to previous player
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(change) = self.future.pop() {
            self.apply_change(&change); // Apply the change
            self.past.push(change);
            self.turn += 1;
            self.current_player = (self.current_player + 1) % 2; // Switch to next player
            true
        } else {
            false
        }
    }
}
