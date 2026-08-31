use crate::{Change, GameEngine, Ownership, PointState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DebugOptions {
    pub include_history: bool,
}

pub fn debug_engine(engine: &GameEngine, options: DebugOptions) -> String {
    let mut lines = vec![
        "GameEngine debug view".to_string(),
        format!("size={}x{}", engine.config.width, engine.config.height),
        format!("scoring_mode={:?}", engine.config.scoring_mode),
        format!("turn={}", engine.turn),
        format!("current_player={}", engine.current_player),
        format!("scores=[{}, {}]", engine.scores[0], engine.scores[1]),
        format!("view_only={}", engine.view_only),
        "legend: ab where a=owner(x or player_id), b=state(x normal, E edge, player_id blocked_by)".to_string(),
        "board_state:".to_string(),
    ];

    lines.extend(format_board_state(&engine.board_state));

    lines.push("edges:".to_string());
    for (point, edges) in &engine.edges {
        if !edges.is_empty() {
            lines.push(format!("{:?}: ({:?})", point, edges));
        }
    }

    if options.include_history {
        lines.push(format!("past_len={}", engine.past.len()));
        lines.extend(format_changes("past", &engine.past));
        lines.push(format!("future_len={}", engine.future.len()));
        lines.extend(format_changes("future", &engine.future));
    }

    lines.join("\n") + "\n"
}

pub fn debug_engine_basic(engine: &GameEngine) -> String {
    debug_engine(engine, DebugOptions::default())
}

fn format_board_state(board_state: &[Vec<PointState>]) -> Vec<String> {
    board_state
        .iter()
        .enumerate()
        .rev()
        .map(|(_, row)| {
            row.iter()
                .enumerate()
                .map(|(_, point_state)| format!("{}", board_token(point_state)))
                .collect::<Vec<_>>()
                .join("  ")
        })
        .collect()
}

fn board_token(point_state: &PointState) -> String {
    let a = owner_token(point_state.ownership);
    let b = if point_state.is_edge {
        "E".to_string()
    } else if point_state.blocked_by != Ownership::None {
        owner_token(point_state.blocked_by)
    } else {
        "x".to_string()
    };
    format!("{a}{b}")
}

fn owner_token(ownership: Ownership) -> String {
    match ownership {
        Ownership::None => "x".to_string(),
        Ownership::Player(player_id) => player_id.to_string(),
    }
}

fn format_changes(label: &str, changes: &[Change]) -> Vec<String> {
    if changes.is_empty() {
        return vec![format!("{label}: []")];
    }
    changes
        .iter()
        .enumerate()
        .map(|(idx, change)| {
            format!(
                "{label}[{idx}]: p{} @ ({}, {}) surrounded={} +edge={} -edge={} +score=[{}, {}]",
                change.mv.player_id,
                change.mv.point.x,
                change.mv.point.y,
                change.surrounded_points.len(),
                change.edge_points_added.len(),
                change.edge_points_removed.len(),
                change.score_changes[0],
                change.score_changes[1]
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{GameConfig, Move, Point, ScoringMode};

    use super::{DebugOptions, debug_engine};

    #[test]
    fn debug_engine_skips_history_by_default() {
        let mut engine = crate::GameEngine::new(GameConfig::new(5, 5, false, ScoringMode::Dots));
        engine.apply_move(Move::new(0, Point::new(2, 2))).unwrap();

        let output = debug_engine(&engine, DebugOptions::default());
        assert!(!output.contains("past_len="));
        assert!(!output.contains("future_len="));
    }

    #[test]
    fn debug_engine_prints_board_tokens() {
        let mut engine = crate::GameEngine::new(GameConfig::new(5, 5, false, ScoringMode::Dots));
        engine.board_state[0][0].ownership = crate::Ownership::Player(1);
        engine.board_state[0][0].is_edge = true;
        engine.board_state[1][1].blocked_by = crate::Ownership::Player(0);
        let output = debug_engine(
            &engine,
            DebugOptions {
                include_history: true,
            },
        );
        assert!(output.contains("xx  xx  xx  xx  xx\nxx  xx  xx  xx  xx\nxx  xx  xx  xx  xx\nxx  x0  xx  xx  xx\n1E  xx  xx  xx  xx\n"));
        assert!(output.contains("past_len=0"));
        assert!(output.contains("future_len=0"));
    }
}
