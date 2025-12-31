use pyo3::prelude::*;
mod engine;
mod solver;

use engine::PopStarEngine;
use solver::PopStarSolver;

/// Python 包装类：PopStarEngine
#[pyclass]
struct PyPopStarEngine {
    inner: PopStarEngine,
}

#[pymethods]
impl PyPopStarEngine {
    #[new]
    #[pyo3(signature = (board=None))]
    fn new(board: Option<Vec<i8>>) -> Self {
        PyPopStarEngine {
            inner: PopStarEngine::new(board),
        }
    }

    fn eliminate(&mut self, r: usize, c: usize) -> i32 {
        self.inner.eliminate(r, c, None)
    }

    fn copy(&self) -> Self {
        PyPopStarEngine {
            inner: self.inner.clone(),
        }
    }

    #[getter]
    fn get_total_score(&self) -> i32 {
        self.inner.total_score
    }

    #[getter]
    fn get_board(&self) -> Vec<Vec<i8>> {
        // 返回二维列表
        let mut res = Vec::new();
        for r in 0..10 {
            let mut row = Vec::new();
            for c in 0..10 {
                row.push(self.inner.board[r * 10 + c]);
            }
            res.push(row);
        }
        res
    }

    fn has_moves(&self) -> bool {
        self.inner.has_moves()
    }
}

/// Python 包装类：PopStarSolver
#[pyclass]
struct PyPopStarSolver {}

#[pymethods]
impl PyPopStarSolver {
    #[new]
    fn new() -> Self {
        PyPopStarSolver {}
    }

    /// 求解接口
    /// 返回: ( (r, c), predicted_score, path_list )
    fn solve(
        &self,
        engine: &PyPopStarEngine,
        iterations: usize,
    ) -> PyResult<(Option<(usize, usize)>, i32, Vec<(usize, usize)>)> {
        // 创建一个新的 solver 实例处理这次请求，避免状态混淆
        // 需要 clone 引擎
        let mut solver = PopStarSolver::new(engine.inner.clone());
        let (move_opt, score, path) = solver.solve(iterations);
        Ok((move_opt, score, path))
    }
}

/// PopStar Rust 扩展模块入口
#[pymodule]
fn popstar_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyPopStarEngine>()?;
    m.add_class::<PyPopStarSolver>()?;
    Ok(())
}
