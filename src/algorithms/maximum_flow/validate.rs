use crate::{
    core::numeric::FlowNum,
    graph::ids::NodeId,
    maximum_flow::{residual_network::ResidualNetwork, status::Status},
};

pub fn validate_input<F: FlowNum>(
    rn: &ResidualNetwork<F>,
    source: NodeId,
    sink: NodeId,
) -> Result<(), Status> {
    if source.index() >= rn.num_nodes || sink.index() >= rn.num_nodes || source == sink {
        return Err(Status::BadInput);
    }

    Ok(())
}
