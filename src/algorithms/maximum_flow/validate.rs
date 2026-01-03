use crate::{
    core::numeric::FlowNum,
    graph::ids::NodeId,
    maximum_flow::{residual_network_core::ResidualNetworkCore, status::Status},
};

pub fn validate_input<N, F: FlowNum>(
    rn: &ResidualNetworkCore<N, F>,
    source: NodeId,
    sink: NodeId,
) -> Result<(), Status> {
    if source.index() >= rn.num_nodes || sink.index() >= rn.num_nodes || source == sink {
        return Err(Status::BadInput);
    }

    Ok(())
}
