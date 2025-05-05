pub mod capacity_scaling;
mod csr;
pub mod dinic;
pub mod edmonds_karp;
pub mod ford_fulkerson;
pub mod graph;
pub mod push_relabel_fifo;
pub mod push_relabel_highest_label;
pub mod shortest_augmenting_path;
pub mod status;
use crate::traits::Zero;
use core::ops::{Add, AddAssign, Sub, SubAssign};

pub trait MaximumFlowSolver<Flow> {
    fn solve(&mut self, graph: &mut graph::Graph<Flow>, s: usize, t: usize, upper: Option<Flow>) -> Result<Flow, status::Status>;
}

pub use self::capacity_scaling::CapacityScaling;
pub use self::dinic::Dinic;
pub use self::edmonds_karp::EdmondsKarp;
pub use self::ford_fulkerson::FordFulkerson;
pub use self::graph::Graph;
pub use self::push_relabel_fifo::PushRelabelFIFO;
pub use self::push_relabel_highest_label::PushRelabelHighestLabel;
pub use self::shortest_augmenting_path::ShortestAugmentingPath;
pub use self::status::Status;

// pub trait FlowNum = Zero + Ord + Add<Output = Self> + Sub<Output = Self> + AddAssign + SubAssign + Copy;
/// “マーカー” だけの本物の trait
pub trait FlowNum: Zero + Ord + Add<Output = Self> + Sub<Output = Self> + AddAssign + SubAssign + Copy {}

// blanket impl ── 条件を満たす型すべてに一発実装
impl<T> FlowNum for T where T: Zero + Ord + Add<Output = T> + Sub<Output = T> + AddAssign + SubAssign + Copy {}
