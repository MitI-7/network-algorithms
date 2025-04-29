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
