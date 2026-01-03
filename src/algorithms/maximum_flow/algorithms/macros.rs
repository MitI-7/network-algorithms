macro_rules! impl_maximum_flow_solver {
    ( $ solver:ident, $run:ident) => {
        impl<N, F> MaximumFlowSolver<N, F> for $solver<N, F>
        where
            F: FlowNum,
        {
            fn new(graph: &Graph<Directed, N, MaximumFlowEdge<F>>) -> Self {
                Self::new(graph)
            }

            fn solve(&mut self, source: NodeId, sink: NodeId) -> Result<MaxFlowResult<F>, Status> {
                self.$run(source, sink)
            }
        }
    };
}

pub(crate) use impl_maximum_flow_solver;
