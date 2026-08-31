# dots-core

Rust library scaffold for a reusable **Dots game backend** (server/client) with:

- move validation
- support for blocked/surrounded dots
- undo/redo move history
- state debugging helpers
- JSON/binary persistence for database save/load scenarios
- structure prepared for future Python and JavaScript/React packaging

## Core modules

- `types` – base data structures (`Point`, `Move`, `GameConfig`)
- `game` – game state, move application, engine with undo/redo integration
- `debug` – debug snapshots and JSON state dump
- `persistence` – serialize/deserialize game history (JSON + bincode)

## Quick start

```rust
use dots_core::{GameConfig, GameEngine, Move, Point};

let mut engine = GameEngine::new(GameConfig::new(4, 4, 2));
engine.apply_move(Move::new(0, Point::new(0, 0), Point::new(1, 0)))?;
engine.undo();
engine.redo();
# Ok::<(), dots_core::GameError>(())
```

## Cross-language packaging path

This crate exports `cdylib` in `Cargo.toml` and defines optional features (`python`, `javascript`) as extension points for wrapper crates (for example `pyo3` and `wasm-bindgen`/N-API).
