macro_rules! impl_minimum_cost_flow_solver {
    ( $ solver:ident, $run:ident) => {
        impl<F> MinimumCostFlowSolver<F> for $solver<F>
        where
            F: CostNum,
        {
            fn new(graph: &Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>) -> Self {
                Self::new(graph)
            }

            fn solve(&mut self) -> Result<MinimumCostFlowResult<F>, Status> {
                self.$run()
            }
        }
    };
}

pub(crate) use impl_minimum_cost_flow_solver;
