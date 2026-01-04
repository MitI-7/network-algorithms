macro_rules! impl_maximum_flow_solver {
    ( $ solver:ident, $run:ident) => {
        impl<F> MaximumFlowSolver<F> for $solver<F>
        where
            F: FlowNum,
        {
            fn new<N>(graph: &Graph<Directed, N, MaximumFlowEdge<F>>) -> Self {
                Self::new(graph)
            }

            fn solve(&mut self, source: NodeId, sink: NodeId) -> Result<MaxFlowResult<F>, Status> {
                self.$run(source, sink)
            }
        }
    };
}

pub(crate) use impl_maximum_flow_solver;
