pub mod capacity_scaling;
pub mod csr;
pub mod dinic;
pub mod edmonds_karp;
pub mod ford_fulkerson;
pub mod graph;
pub mod push_relabel_fifo;
pub mod push_relabel_highest_label;
pub mod shortest_augmenting_path;
pub mod status;

trait MaximumFlowSolver<Flow> {
    fn solve(&mut self, graph: &mut graph::Graph<Flow>, s: usize, t: usize, upper: Option<Flow>) -> Result<Flow, status::Status>;
}

pub use self::capacity_scaling::capacity_sacling;
pub use self::dinic::dinic;
pub use self::edmonds_karp::edmonds_karp;
pub use self::ford_fulkerson::ford_fulkerson;
pub use self::push_relabel_fifo::push_relabel_fifo;
pub use self::push_relabel_highest_label::push_relabel_highest_label;
pub use self::shortest_augmenting_path::shortest_augmenting_path;
