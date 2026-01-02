//! Maximum flow user-facing imports.
//!
//! Use `use crate::maximum_flow::prelude::*;`

pub use super::graph::MaximumFlowGraph;
pub use super::result::MaxFlowResult;

// 代表的なアルゴリズムだけを並べる（全部出す必要はない）
pub use super::ford_fulkerson::FordFulkerson;
// pub use super::dinic::Dinic;  // 実装したら追加

// Solver trait を公開するならここに
pub use super::solver::MaximumFlowSolver;

// エラー型などを公開しているならそれも
pub use super::status::Status;
