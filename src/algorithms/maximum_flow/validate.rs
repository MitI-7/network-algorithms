use crate::{
    core::numeric::FlowNum,
    graph::ids::NodeId,
    algorithms::maximum_flow::{residual_network::ResidualNetwork, error::MaximumFlowError},
};

pub fn validate_input<F: FlowNum>(
    rn: &ResidualNetwork<F>,
    source: NodeId,
    sink: NodeId,
) -> Result<(), MaximumFlowError> {
    if source.index() >= rn.num_nodes || sink.index() >= rn.num_nodes || source == sink {
        return Err(MaximumFlowError::InvalidTerminal {source, sink, num_nodes: rn.num_nodes});
    }

    Ok(())
}
