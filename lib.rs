pub mod debug;
pub mod game;
pub mod persistence;
pub mod types;

pub use debug::{DebugOptions, debug_engine, debug_engine_basic};
pub use game::{GameEngine, GameError, GameHistory};
pub use persistence::{from_bytes, from_json, to_bytes, to_json};
pub use types::{
    Change, GameConfig, Move, Ownership, PlayerId, Point, PointState, ScoringMode,
};

#[cfg(feature = "python")]
use pyo3::exceptions::PyValueError;
#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyDict;

#[cfg(feature = "python")]
fn py_ownership(ownership: Ownership) -> Option<u8> {
    match ownership {
        Ownership::Player(player_id) => Some(player_id),
        Ownership::None => None,
    }
}

#[cfg(feature = "python")]
fn py_point(point: Point) -> (u16, u16) {
    (point.x, point.y)
}

#[cfg(feature = "python")]
fn py_change_dict(py: Python<'_>, change: &Change) -> PyResult<PyObject> {
    let change_dict = PyDict::new_bound(py);
    let move_dict = PyDict::new_bound(py);
    move_dict.set_item("player_id", change.mv.player_id)?;
    move_dict.set_item("point", py_point(change.mv.point))?;
    change_dict.set_item("mv", move_dict)?;
    change_dict.set_item("who_surrounded", py_ownership(change.who_surrounded))?;
    change_dict.set_item(
        "surrounded_points",
        change
            .surrounded_points
            .iter()
            .map(|point| py_point(*point))
            .collect::<Vec<_>>(),
    )?;
    change_dict.set_item(
        "edge_points_added",
        change
            .edge_points_added
            .iter()
            .map(|point| py_point(*point))
            .collect::<Vec<_>>(),
    )?;
    change_dict.set_item(
        "unsurrounded_points",
        change
            .unsurrounded_points
            .iter()
            .map(|point| py_point(*point))
            .collect::<Vec<_>>(),
    )?;
    change_dict.set_item(
        "edge_points_removed",
        change
            .edge_points_removed
            .iter()
            .map(|point| py_point(*point))
            .collect::<Vec<_>>(),
    )?;
    change_dict.set_item("score_changes", change.score_changes.to_vec())?;
    Ok(change_dict.to_object(py))
}

#[cfg(feature = "python")]
#[pyclass]
pub struct PyGameEngine {
    inner: GameEngine,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyGameEngine {
    #[new]
    fn new(width: u16, height: u16, initial_central_dots: bool, scoring_mode: &str) -> PyResult<Self> {
        let mode = match scoring_mode {
            "dots" => ScoringMode::Dots,
            "territory" => ScoringMode::Territory,
            other => {
                return Err(PyValueError::new_err(format!(
                    "scoring_mode must be 'dots' or 'territory', got '{other}'"
                )));
            }
        };

        Ok(Self {
            inner: GameEngine::new(GameConfig::new(
                width,
                height,
                initial_central_dots,
                mode,
            )),
        })
    }

    #[getter]
    fn current_player(&self) -> u8 {
        self.inner.current_player
    }

    #[getter]
    fn turn(&self) -> u32 {
        self.inner.turn
    }

    #[getter]
    fn scores(&self, py: Python<'_>) -> PyObject {
        self.inner.scores.to_vec().to_object(py)
    }

    #[getter]
    fn config(&self) -> (u16, u16, bool, String) {
        let c = &self.inner.config;

        (
            c.width,
            c.height,
            c.initial_central_dots,
            format!("{:?}", c.scoring_mode),
        )
    }

    #[getter]
    fn view_only(&self) -> bool {
        self.inner.view_only
    }

    #[getter]
    fn edges(&self, py: Python<'_>) -> PyResult<PyObject> {
        let result = PyDict::new_bound(py);
        for (point, neighbours) in &self.inner.edges {
            let neighbours_list = neighbours
                .iter()
                .map(|neighbour| py_point(*neighbour))
                .collect::<Vec<_>>();
            result.set_item(py_point(*point), neighbours_list)?;
        }
        Ok(result.to_object(py))
    }

    #[getter]
    fn board_state(&self, py: Python<'_>) -> PyResult<PyObject> {
        let rows = self
            .inner
            .board_state
            .iter()
            .map(|row| {
                row.iter()
                    .map(|state| {
                        let dict = PyDict::new_bound(py);
                        dict.set_item("ownership", py_ownership(state.ownership)).unwrap();
                        dict.set_item("blocked_by", py_ownership(state.blocked_by)).unwrap();
                        dict.set_item("is_edge", state.is_edge).unwrap();
                        dict.to_object(py)
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        Ok(rows.to_object(py))
    }

    #[getter]
    fn past(&self, py: Python<'_>) -> PyResult<PyObject> {
        let items = self
            .inner
            .past
            .iter()
            .map(|change| py_change_dict(py, change))
            .collect::<PyResult<Vec<_>>>()?;
        Ok(items.to_object(py))
    }

    #[getter]
    fn future(&self, py: Python<'_>) -> PyResult<PyObject> {
        let items = self
            .inner
            .future
            .iter()
            .map(|change| py_change_dict(py, change))
            .collect::<PyResult<Vec<_>>>()?;
        Ok(items.to_object(py))
    }

    fn apply_move(&mut self, player_id: u8, x: u16, y: u16) -> PyResult<String> {
        match self.inner.apply_move(Move::new(player_id, Point::new(x, y))) {
            Ok(change) => Ok(format!("{change:?}")),
            Err(err) => Err(PyValueError::new_err(err.to_string())),
        }
    }

    fn undo(&mut self) -> bool {
        self.inner.undo()
    }

    fn redo(&mut self) -> bool {
        self.inner.redo()
    }
}

#[cfg(feature = "python")]
#[pymodule]
fn dots_core(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyGameEngine>()?;
    Ok(())
}

#[cfg(feature = "javascript")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "javascript")]
#[wasm_bindgen]
pub struct JsGameEngine {
    inner: GameEngine,
}

#[cfg(feature = "javascript")]
#[wasm_bindgen]
impl JsGameEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u16, height: u16, initial_central_dots: bool, scoring_mode: &str) -> Result<JsGameEngine, JsValue> {
        let mode = match scoring_mode {
            "dots" => ScoringMode::Dots,
            "territory" => ScoringMode::Territory,
            other => {
                return Err(JsValue::from_str(&format!(
                    "scoring_mode must be 'dots' or 'territory', got '{other}'"
                )));
            }
        };

        Ok(JsGameEngine {
            inner: GameEngine::new(GameConfig::new(
                width,
                height,
                initial_central_dots,
                mode,
            )),
        })
    }

    #[wasm_bindgen(js_name = "applyMove")]
    pub fn apply_move(&mut self, player_id: u8, x: u16, y: u16) -> Result<String, JsValue> {
        match self.inner.apply_move(Move::new(player_id, Point::new(x, y))) {
            Ok(change) => Ok(format!("{change:?}")),
            Err(err) => Err(JsValue::from_str(&err.to_string())),
        }
    }

    #[wasm_bindgen]
    pub fn undo(&mut self) -> bool {
        self.inner.undo()
    }

    #[wasm_bindgen]
    pub fn redo(&mut self) -> bool {
        self.inner.redo()
    }
}

#[cfg(test)]
mod lib_test;
