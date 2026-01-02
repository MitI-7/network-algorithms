use crate::{
    FlowNum,
    graph::{direction::Directed, graph::Graph, ids::NodeId},
    maximum_flow::{edge::MaximumFlowEdge, status::Status},
};

pub fn validate_input<N, F: FlowNum>(
    graph: &Graph<Directed, N, MaximumFlowEdge<F>>,
    source: NodeId,
    sink: NodeId,
) -> Result<(), Status> {
    if source.index() >= graph.num_nodes() || sink.index() >= graph.num_nodes() || source == sink {
        return Err(Status::BadInput);
    }

    Ok(())
}
