#[cfg(test)]
mod tests {
    use crate::{GameConfig, GameEngine, GameError, Move, Point, from_json, to_json};

    #[test]
    fn apply_move_switches_player_and_turn() {
        let mut engine = GameEngine::new(GameConfig::new(3, 3, 2));

        let outcome = engine
            .apply_move(Move::new(0, Point::new(0, 0), Point::new(1, 0)))
            .expect("first move should be valid");

        assert_eq!(outcome.turn, 1);
        assert_eq!(outcome.next_player, 1);
        assert_eq!(engine.state().current_player, 1);
    }

    #[test]
    fn undo_redo_restores_state() {
        let mut engine = GameEngine::new(GameConfig::new(3, 3, 2));
        let first = Move::new(0, Point::new(0, 0), Point::new(1, 0));
        let second = Move::new(1, Point::new(0, 1), Point::new(1, 1));

        engine.apply_move(first).expect("first move should pass");
        engine.apply_move(second).expect("second move should pass");

        assert!(engine.undo());
        assert_eq!(engine.state().turn, 1);

        assert!(engine.redo());
        assert_eq!(engine.state().turn, 2);
    }

    #[test]
    fn surrounded_dot_cannot_be_used() {
        let mut engine = GameEngine::new(GameConfig::new(3, 3, 2));
        engine.mark_surrounded_dots([Point::new(0, 0)]);

        let err = engine
            .apply_move(Move::new(0, Point::new(0, 0), Point::new(1, 0)))
            .expect_err("move touching blocked dot must fail");

        assert_eq!(err, GameError::BlockedDotUsed);
    }

    #[test]
    fn state_serialization_round_trip() {
        let mut engine = GameEngine::new(GameConfig::new(2, 2, 2));
        engine
            .apply_move(Move::new(0, Point::new(0, 0), Point::new(1, 0)))
            .expect("move should pass");

        let json = to_json(engine.state()).expect("state should serialize");
        let state = from_json(&json).expect("state should deserialize");

        assert_eq!(state, engine.state().clone());
    }
}
