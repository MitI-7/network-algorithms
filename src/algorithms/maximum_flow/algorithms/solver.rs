use crate::{
    algorithms::maximum_flow::{
        edge::MaximumFlowEdge,
        result::{MaximumFlowResult, MinimumCutResult},
        status::Status,
    },
    core::numeric::FlowNum,
    graph::{direction::Directed, graph::Graph, ids::NodeId},
};

pub trait MaximumFlowSolver<F: FlowNum> {
    fn new<N>(graph: &Graph<Directed, N, MaximumFlowEdge<F>>) -> Self;
    fn maximum_flow(&mut self, source: NodeId, sink: NodeId) -> Result<MaximumFlowResult<F>, Status>;
    fn maximum_flow_value(&mut self, source: NodeId, sink: NodeId) -> Result<F, Status>;
    fn minimum_cut(&mut self, source: NodeId, sink: NodeId) -> Result<MinimumCutResult<F>, Status>;
    fn minimum_cut_value(&mut self, source: NodeId, sink: NodeId) -> Result<F, Status>;
}
