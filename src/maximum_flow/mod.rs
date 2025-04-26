use crate::maximum_flow::status::Status;

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
    fn solve(&mut self, graph: &mut graph::Graph<Flow>, s: usize, t: usize, upper: Option<Flow>) -> Result<Flow, Status>;
}
