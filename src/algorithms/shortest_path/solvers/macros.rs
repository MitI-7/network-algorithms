macro_rules! impl_shortest_path_solver {
    ( $solver:ident, $run:ident $(, $bound:path )* $(,)? ) => {
        impl<W> ShortestPathSolver<W> for $solver<W>
        where
            W: FlowNum,
        {
            fn new(graph: &Graph<Directed, (), WeightEdge<W>>) -> Self {
                Self::new(graph)
            }

            fn solve(&mut self, source: NodeId) -> Result<(), Status> {
                self.$run(source)
            }
            
            fn distance(&self, u: NodeId) -> Option<W> {
                if self.reached.get(u.index()) {
                    Some(self.distances[u.index()])
                } else {
                    None
                }
            }
    
            fn reached(&self, u: NodeId) -> bool {
                self.reached.get(u.index())
            }
        }
    };
}

pub(crate) use impl_shortest_path_solver;
