#[cfg(test)]
mod tests {
    use crate::{DebugOptions, GameConfig, GameEngine, GameError, Move, Point, from_json, to_json, debug_engine};

    #[test]
    fn simple_surround() {
        let mut engine = GameEngine::new(GameConfig::new(10, 10, true, crate::ScoringMode::Dots));
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
}
