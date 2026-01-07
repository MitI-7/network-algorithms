macro_rules! impl_minimum_cost_flow_solver {
    ( $solver:ident, $run:ident $(, $bound:path )* $(,)? ) => {
        impl<F> MinimumCostFlowSolver<F> for $solver<F>
        where
            F: CostNum $(+ $bound)*,
        {
            fn new(graph: &Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>) -> Self {
                Self::new(graph)
            }

            fn minimum_cost_flow(&mut self) -> Result<MinimumCostFlowResult<F>, Status> {
                let objective_value = self.$run()?;
                Ok(MinimumCostFlowResult {
                    objective_value,
                    flows: self.make_minimum_cost_flow_in_original_graph()
                })
            }

            fn minimum_cost_flow_value(&mut self) -> Result<F, Status> {
                let objective_value = self.$run()?;
                Ok(objective_value)
            }
        }
    };
}

pub(crate) use impl_minimum_cost_flow_solver;
