macro_rules! impl_shortest_path_solver {
    ( $solver:ident, $run:ident $(, $bound:path )* $(,)? ) => {
        impl<W> ShortestPathSolver<W> for $solver<W>
        where
            W: FlowNum,
        {
            fn new(graph: &Graph<Directed, (), WeightEdge<W>>) -> Self {
                Self::new(graph)
            }

            fn solve(&mut self, source: NodeId) -> Result<ShortestPathResult<W>, Status> {
                self.$run(source)
            }
        }
    };
}

pub(crate) use impl_shortest_path_solver;
