# dots-core

A reusable backend for the Dots game with move validation, surrounding logic, undo/redo history, and serializer helpers for Rust, Python, and JavaScript/WASM builds.

## Features

- strict move validation and game-state checks
- ownership / blocked-dot tracking
- territory and dot-scoring modes
- undo and redo support
- state snapshots for debugging and persistence
- JSON + binary serialization helpers
- Rust core library with optional Python and JavaScript exports

## Rust usage

```rust
use dots_core::{GameConfig, GameEngine, Move, Point, ScoringMode};

let mut engine = GameEngine::new(GameConfig::new(10, 10, true, ScoringMode::Dots));
engine.apply_move(Move::new(0, Point::new(3, 5)))?;
engine.undo();
engine.redo();
# Ok::<(), dots_core::GameError>(())
```

## Python package

Build the wheel with maturin:

```bash
maturin build --release --features python --interpreter "C:/path/to/python.exe"
```

Install the generated wheel:

```bash
pip install target/wheels/dots_core-0.1.0-cp312-cp312-win_amd64.whl
```

Then use it as:

```python
import dots_core

engine = dots_core.PyGameEngine(10, 10, True, "dots")
print(engine.current_player)
print(engine.turn)
print(engine.config)
print(engine.edges)

engine.apply_move(0, 3, 5)
engine.undo()
engine.redo()
```

## JavaScript / WebAssembly package

Build the WASM bundle for the web or bundler target:

```bash
wasm-pack build --target bundler --out-dir pkg --features javascript
```

If the local binaryen optimizer fails on your machine with a bulk-memory validator error, retry without optimization:

```bash
wasm-pack build --target bundler --out-dir pkg --no-opt --features javascript
```

Then publish the generated package from the generated folder, or consume it directly in a browser or bundler project:

```js
import init, { JsGameEngine } from "./pkg/dots_core.js";

await init();
const engine = new JsGameEngine(10, 10, true, "dots");
console.log(engine.currentPlayer);
console.log(engine.config);
engine.applyMove(0, 3, 5);
```

## Publish-ready package checklist

The project is set up to publish as:

- a Rust crate via Cargo
- a Python extension wheel via maturin
- a JavaScript package via wasm-pack output in the generated pkg directory

Before publishing, confirm:

1. the crate metadata in [Cargo.toml](Cargo.toml) is correct
2. the Python metadata file [pyproject.toml](pyproject.toml) is available
3. the generated JS package includes a package manifest in the output folder
4. the project README and package descriptions are accurate and up to date

## Module layout

- `types` — shared data models such as `Point`, `Move`, `GameConfig`, `Ownership`
- `game` — engine logic, validation, scoring, history, undo/redo
- `persistence` — JSON and binary serialization helpers
- `debug` — inspectable engine state output for testing and diagnostics
