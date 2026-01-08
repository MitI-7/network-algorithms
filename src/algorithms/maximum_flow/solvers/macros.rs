macro_rules! impl_maximum_flow_solver {
    ( $solver:ident, $run:ident $(, $bound:path )* $(,)? ) => {
        impl<F> MaximumFlowSolver<F> for $solver<F>
        where
            F: FlowNum $( + $bound )*,
        {
            fn new<N>(graph: &Graph<Directed, N, MaximumFlowEdge<F>>) -> Self
            where
                Self: Sized,
            {
                Self::new(graph)
            }

            fn solve(&mut self, source: NodeId, sink: NodeId) -> Result<F, Status> {
                let objective_value = self.$run(source, sink)?;
                Ok(objective_value)
            }

            fn flow(&self, edge_id: EdgeId) -> Option<F> {
                if edge_id.index() >= self.rn.num_edges {
                    return None;
                }

                let arc_id = self.rn.edge_id_to_arc_id[edge_id.index()];
                Some(self.rn.upper[arc_id.index()] - self.rn.residual_capacities[arc_id.index()])
            }

            fn minimum_cut(&mut self) -> Result<Vec<bool>, Status> {
                Ok(self.rn.reachable_from_source(self.source))
            }
        }
    };
}

pub(crate) use impl_maximum_flow_solver;
