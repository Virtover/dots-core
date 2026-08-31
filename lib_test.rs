#[cfg(test)]
mod tests {
    use crate::{
        DebugOptions, GameConfig, GameEngine, GameError, Move, Ownership, Point, from_json,
        to_json, debug_engine,
    };

    #[test]
    fn simple_surround() {
        let mut engine = GameEngine::new(GameConfig::new(10, 10, true, crate::ScoringMode::Territory));
        engine.apply_move(Move::new(0, Point::new(3, 5))).unwrap();
        engine.apply_move(Move::new(1, Point::new(3, 4))).unwrap();
        engine.apply_move(Move::new(0, Point::new(4, 6))).unwrap();
        engine.apply_move(Move::new(1, Point::new(4, 3))).unwrap();
        print!("{}", debug_engine(&engine, DebugOptions::default()));
        assert!(debug_engine(&engine, DebugOptions::default()).contains(
            concat!(
                "xx  xx  xx  xx  xx  xx  xx  xx  xx  xx\n",
                "xx  xx  xx  xx  xx  xx  xx  xx  xx  xx\n",
                "xx  xx  xx  xx  xx  xx  xx  xx  xx  xx\n",
                "xx  xx  xx  xx  0E  xx  xx  xx  xx  xx\n",
                "xx  xx  xx  0E  10  0E  xx  xx  xx  xx\n",
                "xx  xx  xx  1x  0E  1x  xx  xx  xx  xx\n",
                "xx  xx  xx  xx  1x  xx  xx  xx  xx  xx\n",
                "xx  xx  xx  xx  xx  xx  xx  xx  xx  xx\n",
                "xx  xx  xx  xx  xx  xx  xx  xx  xx  xx\n",
                "xx  xx  xx  xx  xx  xx  xx  xx  xx  xx",
        )));
    }

    #[test]
    fn basic_move_errors() {
        let mut engine = GameEngine::new(GameConfig::new(10, 10, true, crate::ScoringMode::Dots));

        assert_eq!(
            engine.apply_move(Move::new(1, Point::new(0, 0))),
            Err(GameError::NotCurrentPlayer {
                expected: 0,
                got: 1,
            })
        );
        assert_eq!(engine.board_state[0][0].ownership, Ownership::None);

        engine.apply_move(Move::new(0, Point::new(0, 0))).unwrap();
        assert_eq!(
            engine.apply_move(Move::new(1, Point::new(0, 0))),
            Err(GameError::PointOccupied)
        );

        let mut blocked = GameEngine::new(GameConfig::new(10, 10, true, crate::ScoringMode::Dots));
        blocked.board_state[2][2].blocked_by = Ownership::Player(1);
        assert_eq!(
            blocked.apply_move(Move::new(0, Point::new(2, 2))),
            Err(GameError::PointBlocked)
        );
        assert_eq!(blocked.board_state[2][2].blocked_by, Ownership::Player(1));
    }

    #[test]
    fn nested_surround1_undo_redo() {
        let mut engine = GameEngine::new(GameConfig::new(10, 10, true, crate::ScoringMode::Dots));
        engine.apply_move(Move::new(0, Point::new(3, 5))).unwrap();
        engine.apply_move(Move::new(1, Point::new(3, 4))).unwrap();
        engine.apply_move(Move::new(0, Point::new(4, 6))).unwrap();
        engine.apply_move(Move::new(1, Point::new(4, 3))).unwrap();
        engine.apply_move(Move::new(0, Point::new(0, 9))).unwrap();
        engine.apply_move(Move::new(1, Point::new(2, 5))).unwrap();
        engine.apply_move(Move::new(0, Point::new(4, 7))).unwrap();
        engine.apply_move(Move::new(1, Point::new(2, 6))).unwrap();
        engine.apply_move(Move::new(0, Point::new(0, 0))).unwrap();
        engine.apply_move(Move::new(1, Point::new(3, 7))).unwrap();
        engine.apply_move(Move::new(0, Point::new(0, 1))).unwrap();
        engine.apply_move(Move::new(1, Point::new(3, 8))).unwrap();
        engine.apply_move(Move::new(0, Point::new(0, 2))).unwrap();
        engine.apply_move(Move::new(1, Point::new(4, 9))).unwrap();
        engine.apply_move(Move::new(0, Point::new(0, 3))).unwrap();
        engine.apply_move(Move::new(1, Point::new(5, 8))).unwrap();
        engine.apply_move(Move::new(0, Point::new(0, 4))).unwrap();
        engine.apply_move(Move::new(1, Point::new(6, 7))).unwrap();
        engine.apply_move(Move::new(0, Point::new(0, 5))).unwrap();
        engine.apply_move(Move::new(1, Point::new(6, 6))).unwrap();
        engine.apply_move(Move::new(0, Point::new(0, 6))).unwrap();
        engine.apply_move(Move::new(1, Point::new(6, 5))).unwrap();
        engine.undo();
        engine.undo();

        print!("{}", debug_engine(&engine, DebugOptions::default()));
        assert!(debug_engine(&engine, DebugOptions::default()).contains(
            concat!(
                "0x  xx  xx  xx  1x  xx  xx  xx  xx  xx\n",
                "xx  xx  xx  1x  xx  1x  xx  xx  xx  xx\n",
                "xx  xx  xx  1x  0x  xx  1x  xx  xx  xx\n",
                "xx  xx  1x  xx  0E  xx  1x  xx  xx  xx\n",
                "0x  xx  1x  0E  10  0E  xx  xx  xx  xx\n",
                "0x  xx  xx  1x  0E  1x  xx  xx  xx  xx\n",
                "0x  xx  xx  xx  1x  xx  xx  xx  xx  xx\n",
                "0x  xx  xx  xx  xx  xx  xx  xx  xx  xx\n",
                "0x  xx  xx  xx  xx  xx  xx  xx  xx  xx\n",
                "0x  xx  xx  xx  xx  xx  xx  xx  xx  xx",
        )));
        engine.redo();
        engine.redo();

        print!("{}", debug_engine(&engine, DebugOptions::default()));
        assert!(debug_engine(&engine, DebugOptions::default()).contains(
            concat!(
                "0x  xx  xx  xx  1E  xx  xx  xx  xx  xx\n",
                "xx  xx  xx  1E  x1  1E  xx  xx  xx  xx\n",
                "xx  xx  xx  1E  01  x1  1E  xx  xx  xx\n",
                "0x  xx  1E  x1  01  x1  1E  xx  xx  xx\n",
                "0x  xx  1E  01  11  01  1E  xx  xx  xx\n",
                "0x  xx  xx  1E  01  1E  xx  xx  xx  xx\n",
                "0x  xx  xx  xx  1E  xx  xx  xx  xx  xx\n",
                "0x  xx  xx  xx  xx  xx  xx  xx  xx  xx\n",
                "0x  xx  xx  xx  xx  xx  xx  xx  xx  xx\n",
                "0x  xx  xx  xx  xx  xx  xx  xx  xx  xx",
        )));
    }
}
