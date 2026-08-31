use crate::types::{Change, GameConfig, Move, Ownership, PlayerId, Point, PointState, ScoringMode};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SChange {
    pub surrounded_points: BTreeSet<Point>,
    pub edge_points_added: BTreeSet<Point>,
    pub unsurrounded_points: BTreeSet<Point>,
    pub edge_points_removed: BTreeSet<Point>,
    pub score_changes: [i32; 2], // (player0_score_change, player1_score_change)
}

impl SChange {
    pub fn empty() -> Self {
        Self {
            surrounded_points: BTreeSet::new(),
            edge_points_added: BTreeSet::new(),
            unsurrounded_points: BTreeSet::new(),
            edge_points_removed: BTreeSet::new(),
            score_changes: [0, 0],
        }
    }

    pub fn merge(changes: impl IntoIterator<Item = SChange>) -> Self {
        let mut result = Self {
            surrounded_points: BTreeSet::new(),
            edge_points_added: BTreeSet::new(),
            unsurrounded_points: BTreeSet::new(),
            edge_points_removed: BTreeSet::new(),
            score_changes: [0, 0],
        };

        for change in changes {
            result.surrounded_points.extend(change.surrounded_points);
            result.edge_points_added.extend(change.edge_points_added);
            result
                .unsurrounded_points
                .extend(change.unsurrounded_points);
            result
                .edge_points_removed
                .extend(change.edge_points_removed);
            result.score_changes[0] += change.score_changes[0];
            result.score_changes[1] += change.score_changes[1];
        }

        result
    }

    pub fn to_change(self, mv: Move, who_surrounded: Ownership) -> Change {
        Change {
            mv,
            who_surrounded,
            surrounded_points: self.surrounded_points.into_iter().collect(),
            edge_points_added: self.edge_points_added.into_iter().collect(),
            unsurrounded_points: self.unsurrounded_points.into_iter().collect(),
            edge_points_removed: self.edge_points_removed.into_iter().collect(),
            score_changes: self.score_changes,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.surrounded_points.is_empty()
            && self.edge_points_added.is_empty()
            && self.unsurrounded_points.is_empty()
            && self.edge_points_removed.is_empty()
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
    pub scores: [i32; 2], // (player0_score, player1_score)
    pub board_state: Vec<Vec<PointState>>,
    pub past: Vec<Change>,
    pub future: Vec<Change>,
    pub view_only: bool,
}

impl GameEngine {
    pub fn new(config: GameConfig) -> Self {
        let mut board_state =
            vec![vec![PointState::new(); config.width as usize]; config.height as usize];
        if config.initial_central_dots {
            let center_x = config.width / 2;
            let center_y = config.height / 2;
            for y in (center_y - 1)..=(center_y) {
                for x in (center_x - 1)..=(center_x) {
                    board_state[y as usize][x as usize].ownership =
                        Ownership::Player((x + y) as u8 % 2);
                }
            }
        }
        Self {
            config,
            turn: 0,
            current_player: 0,
            scores: [0, 0],
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

    fn is_boundary_point(&self, point: &Point) -> bool {
        point.x == 0
            || point.x == self.config.width - 1
            || point.y == 0
            || point.y == self.config.height - 1
    }

    fn is_surrounding_possible(&self, last_placed_dot: &Point) -> bool {
        assert!(
            last_placed_dot.x < self.config.width && last_placed_dot.y < self.config.height,
            "Point is out of bounds"
        );
        let who_may_surround =
            self.board_state[last_placed_dot.y as usize][last_placed_dot.x as usize].ownership;
        if who_may_surround == Ownership::None {
            return false;
        }
        // todo: explore point states in proper directions
        true
    }

    fn get_neighbours(&self, point: &Point) -> Vec<Point> {
        let mut neighbours = Vec::new();
        let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        for (dx, dy) in directions.iter() {
            let new_x = point.x as i32 + dx;
            let new_y = point.y as i32 + dy;
            if new_x >= 0
                && new_x < self.config.width as i32
                && new_y >= 0
                && new_y < self.config.height as i32
            {
                neighbours.push(Point::new(new_x as u16, new_y as u16));
            }
        }
        neighbours
    }

    fn get_schange(&self, point: &Point, who_may_surround: Ownership) -> SChange {
        assert!(
            point.x < self.config.width && point.y < self.config.height,
            "Point is out of bounds"
        );
        let who_may_surround_player_id = match who_may_surround {
            Ownership::Player(id) => id,
            _ => panic!("who_may_surround must be a player"),
        };
        // bfs: who_may_surround dots are border
        //      store border points in BTreeSet and possibly surrounded points in another BTreeSet
        //      if reached map boundaries - not surrounded
        //      else, after bfs:
        //        if there were no point of opponent to who_may_surround - not surrounded
        //        if there were point of opponent to who_may_surround - surrounded by who_may_surround
        //          in that case, start bfs: starting set is all boundary points of board
        //             previously stored border points reached in second bfs are points that form edges of surrounded area
        //             store outer-points, edge-points in separate BTreeSets
        //             remaining points are surrounded points (surrounded_points)
        //             store all edges & points surrounded by another player than who_may_surround in surrounded area in separate unsurrounded_points, edges_removed BTreeSets
        //             form edges from points stored in edge-points
        //             return SChange with all these sets; remember to store score changes in score_changes field, depending on scoring mode (dots or territory)
        // todo
        let mut border = BTreeSet::new();
        let mut possibly_surrounded = BTreeSet::new();
        let mut to_check = vec![*point];
        let mut any_opponent_dots_found = false;

        while let Some(current) = to_check.pop() {
            if border.contains(&current) || possibly_surrounded.contains(&current) {
                continue;
            }

            let state = &self.board_state[current.y as usize][current.x as usize];
            if let Ownership::Player(player_id) = state.ownership {
                if player_id == who_may_surround_player_id {
                    if state.blocked_by == Ownership::None {
                        border.insert(current);
                        continue;
                    }
                } else {
                    any_opponent_dots_found = true;
                }
            }

            if self.is_boundary_point(&current) {
                return SChange::empty();
            }

            possibly_surrounded.insert(current);
            to_check.extend(self.get_neighbours(&current));
        }

        if !any_opponent_dots_found {
            return SChange::empty();
        }

        let mut edge_points_added = BTreeSet::new();
        let mut outer_points = BTreeSet::new();
        let mut to_check: Vec<Point> = (0..self.config.width)
            .flat_map(|x| [Point::new(x, 0), Point::new(x, self.config.height - 1)])
            .chain(
                (0..self.config.height)
                    .flat_map(|y| [Point::new(0, y), Point::new(self.config.width - 1, y)]),
            )
            .collect();

        while let Some(current) = to_check.pop() {
            if edge_points_added.contains(&current) || outer_points.contains(&current) {
                continue;
            }

            if border.contains(&current) {
                edge_points_added.insert(current);
                continue;
            }

            outer_points.insert(current);
            to_check.extend(self.get_neighbours(&current));
        }

        let mut surrounded_points = BTreeSet::new();
        let mut unsurrounded_points = BTreeSet::new();
        let mut edge_points_removed = BTreeSet::new();
        let mut score_changes = [0, 0];

        for x in 0..self.config.width {
            for y in 0..self.config.height {
                let p = Point::new(x, y);
                if edge_points_added.contains(&p) || outer_points.contains(&p) {
                    continue;
                }

                let state = &self.board_state[y as usize][x as usize];

                if state.blocked_by == who_may_surround
                    || (state.is_edge && state.ownership == who_may_surround)
                {
                    continue;
                }

                surrounded_points.insert(p);
                if self.config.scoring_mode == ScoringMode::Territory
                    || state.ownership == Ownership::Player((who_may_surround_player_id + 1) % 2)
                {
                    score_changes[who_may_surround_player_id as usize] += 1;
                }

                if let Ownership::Player(other_player_id) = state.blocked_by
                    && other_player_id != who_may_surround_player_id
                {
                    unsurrounded_points.insert(p);
                }

                if state.is_edge && state.ownership != who_may_surround {
                    edge_points_removed.insert(p);
                }
            }
        }

        SChange {
            surrounded_points: surrounded_points,
            edge_points_added: edge_points_added,
            unsurrounded_points: unsurrounded_points,
            edge_points_removed: edge_points_removed,
            score_changes: score_changes,
        }
    }

    fn apply_change(&mut self, change: &Change, undo: bool) {
        let mv = &change.mv;
        self.board_state[mv.point.y as usize][mv.point.x as usize].ownership =
            Ownership::Player(mv.player_id);
        for point in &change.unsurrounded_points {
            self.board_state[point.y as usize][point.x as usize].blocked_by = Ownership::None;
        }
        for point in &change.surrounded_points {
            self.board_state[point.y as usize][point.x as usize].blocked_by =
                Ownership::Player(mv.player_id);
        }
        for point in &change.edge_points_removed {
            self.board_state[point.y as usize][point.x as usize].is_edge = false;
        }
        for point in &change.edge_points_added {
            self.board_state[point.y as usize][point.x as usize].is_edge = true;
        }

        let mult = if undo { -1 } else { 1 };
        self.scores[0] += mult * change.score_changes[0];
        self.scores[1] += mult * change.score_changes[1];
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
            edge_points_added: Vec::new(),
            unsurrounded_points: Vec::new(),
            edge_points_removed: Vec::new(),
            score_changes: [0, 0],
        };
        if !self.is_surrounding_possible(&mv.point) {
            return Ok(change);
        }

        self.board_state[mv.point.y as usize][mv.point.x as usize].ownership =
            Ownership::Player(mv.player_id);
        let neighbours = self.get_neighbours(&mv.point);
        let schange = SChange::merge(
            neighbours
                .iter()
                .map(|n| self.get_schange(n, Ownership::Player(mv.player_id))),
        );
        if !schange.is_empty() {
            change = schange.to_change(mv, Ownership::Player(mv.player_id));
        } else {
            let who_surrounded = Ownership::Player((mv.player_id + 1) % 2);
            change = self
                .get_schange(&mv.point, who_surrounded)
                .to_change(mv, who_surrounded);
        }

        self.apply_change(&change, false);

        self.past.push(change.clone());
        self.future.clear();
        self.turn += 1;
        self.current_player = (self.current_player + 1) % 2; // Assuming 2 players

        Ok(change)
    }

    pub fn undo(&mut self) -> bool {
        if let Some(change) = self.past.pop() {
            self.apply_change(&change, true); // Apply the change in reverse
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
            self.apply_change(&change, false); // Apply the change
            self.past.push(change);
            self.turn += 1;
            self.current_player = (self.current_player + 1) % 2; // Switch to next player
            true
        } else {
            false
        }
    }
}
