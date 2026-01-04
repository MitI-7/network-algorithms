macro_rules! impl_maximum_flow_solver {
    ( $ solver:ident, $run:ident) => {
        impl<F> MaximumFlowSolver<F> for $solver<F>
        where
            F: FlowNum,
        {
            fn new<N>(graph: &Graph<Directed, N, MaximumFlowEdge<F>>) -> Self {
                Self::new(graph)
            }

            fn maximum_flow(&mut self, source: NodeId, sink: NodeId) -> Result<MaximumFlowResult<F>, Status> {
                let objective_value = self.$run(source, sink)?;
                Ok(MaximumFlowResult { objective_value, flows: self.rn.get_flows(&self.rn.residual_capacities) })
            }

            fn maximum_flow_value(&mut self, source: NodeId, sink: NodeId) -> Result<F, Status> {
                let objective_value = self.$run(source, sink)?;
                Ok(objective_value)
            }

            fn minimum_cut(&mut self, source: NodeId, sink: NodeId) -> Result<MinimumCutResult<F>, Status> {
                let objective_value = self.$run(source, sink)?;
                Ok(MinimumCutResult { objective_value, minimum_cut: self.rn.reachable_from_source(source) })
            }

            fn minimum_cut_value(&mut self, source: NodeId, sink: NodeId) -> Result<F, Status> {
                let objective_value = self.$run(source, sink)?;
                Ok(objective_value)
            }
        }
    };
}

pub(crate) use impl_maximum_flow_solver;
