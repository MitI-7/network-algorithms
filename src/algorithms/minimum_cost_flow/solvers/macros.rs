macro_rules! impl_minimum_cost_flow_solver {
    ( $solver:ident, $run:ident $(, $bound:path )* $(,)? ) => {
        impl<F> MinimumCostFlowSolver<F> for $solver<F>
        where
            F: CostNum $(+ $bound)*,
        {
            fn new(graph: &Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>) -> Self
            where
                Self: Sized
            {
                Self::new(graph)
            }

            fn solve(&mut self) -> Result<F, MinimumCostFlowError> {
                self.$run()
            }

            fn flow(&self, edge_id: EdgeId) -> Result<F, MinimumCostFlowError> {
                self.flow(edge_id)
            }
            
            fn flows(&self) -> Result<Vec<F>, MinimumCostFlowError> {
                self.flows()
            }
            
            fn potential(&self, node_id: NodeId) -> Result<F, MinimumCostFlowError> {
                self.potential(node_id)
            }
            
            fn potentials(&self) -> Result<Vec<F>, MinimumCostFlowError> {
                self.potentials()
            }
        }
    };
}

pub(crate) use impl_minimum_cost_flow_solver;
