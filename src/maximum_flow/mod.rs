pub mod capacity_scaling;
mod csr;
pub mod dinic;
pub mod edmonds_karp;
pub mod ford_fulkerson;
pub mod push_relabel_fifo;
pub mod push_relabel_highest_label;
pub mod shortest_augmenting_path;
pub mod status;
use crate::core::direction::Directed;
use crate::core::graph::Graph;
use crate::core::ids::NodeId;
use crate::edge::capacity::CapEdge;
use crate::traits::Zero;
use core::ops::{Add, AddAssign, Sub, SubAssign};

pub trait MaximumFlowSolver<Flow>
where
    Flow: FlowNum,
{
    fn solve(&mut self, graph: &mut Graph<Directed, (), CapEdge<Flow>>, source: NodeId, sink: NodeId, upper: Option<Flow>) -> Result<Flow, Status>;
}

pub use self::capacity_scaling::CapacityScaling;
pub use self::dinic::Dinic;
pub use self::edmonds_karp::EdmondsKarp;
pub use self::ford_fulkerson::FordFulkerson;
pub use self::push_relabel_fifo::PushRelabelFIFO;
pub use self::push_relabel_highest_label::PushRelabelHighestLabel;
pub use self::shortest_augmenting_path::ShortestAugmentingPath;
pub use self::status::Status;

// pub trait FlowNum = Zero + Ord + Add<Output = Self> + Sub<Output = Self> + AddAssign + SubAssign + Copy;
pub trait FlowNum: Zero + Ord + Add<Output = Self> + Sub<Output = Self> + AddAssign + SubAssign + Clone + Copy {}
impl<T> FlowNum for T where T: Zero + Ord + Add<Output = T> + Sub<Output = T> + AddAssign + SubAssign + Clone + Copy {}
