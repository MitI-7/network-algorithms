use crate::{
    algorithms::maximum_flow::{
        edge::MaximumFlowEdge, residual_network_core::ResidualNetworkCore, result::MaxFlowResult,
        status::Status, validate::validate_input,
    },
    core::numeric::FlowNum,
    graph::{
        direction::Directed,
        graph::Graph,
        ids::{ArcId, NodeId},
    },
};
use std::collections::VecDeque;
use std::marker::PhantomData;
use crate::algorithms::maximum_flow::algorithms::solver::MaximumFlowSolver;

#[derive(Default)]
pub struct Dinic<N, F> {
    residual_capacities: Box<[F]>,
    excesses: Box<[F]>,
    current_edge: Vec<usize>,
    distances_to_sink: Vec<usize>,
    que: VecDeque<NodeId>,
    phantom: PhantomData<N>,
}

impl<N, F> MaximumFlowSolver<N, F> for Dinic<N, F>
where
    F: FlowNum,
{
    type Prepared = ResidualNetworkCore<N, F>;

    fn solve(
        &mut self,
        graph: &Graph<Directed, N, MaximumFlowEdge<F>>,
        source: NodeId,
        sink: NodeId,
        upper: Option<F>,
    ) -> Result<MaxFlowResult<F>, Status> {
        validate_input(graph, source, sink)?;
        let rn = ResidualNetworkCore::from_graph(graph);
        self.run_with_prepared(&rn, source, sink, upper)
    }

    fn prepare(
        &mut self,
        graph: &Graph<Directed, N, MaximumFlowEdge<F>>,
    ) -> Result<Self::Prepared, Status> {
        Ok(ResidualNetworkCore::from_graph(graph))
    }

    fn solve_with_prepared(
        &mut self,
        rn: &Self::Prepared,
        s: NodeId,
        t: NodeId,
        upper: Option<F>,
    ) -> Result<MaxFlowResult<F>, Status> {
        self.run_with_prepared(rn, s, t, upper)
    }
}

impl<N, F> Dinic<N, F>
where
    F: FlowNum,
{
    fn run_with_prepared(
        &mut self,
        rn: &ResidualNetworkCore<N, F>,
        source: NodeId,
        sink: NodeId,
        upper: Option<F>,
    ) -> Result<MaxFlowResult<F>, Status> {
        // initialize
        if self.residual_capacities.len() != rn.num_edges * 2 {
            self.residual_capacities = vec![F::zero(); rn.num_edges * 2].into_boxed_slice();
        } else {
            self.residual_capacities.fill(F::zero());
        }
        if self.excesses.len() != rn.num_nodes {
            self.excesses = vec![F::zero(); rn.num_nodes].into_boxed_slice();
        } else {
            self.excesses.fill(F::zero());
        }

        self.residual_capacities.copy_from_slice(&rn.upper);
        self.distances_to_sink.resize(rn.num_nodes, 0);

        self.current_edge.resize(rn.num_nodes, 0);

        let mut residual = upper.unwrap_or_else(|| {
            rn.neighbors(source)
                .fold(F::zero(), |sum, arc_id| sum + rn.upper[arc_id.index()])
        });
        let mut objective_value = F::zero();
        while residual > F::zero() {
            self.update_distances_to_sink(rn, source, sink);

            // no s-t path
            if self.distances_to_sink[source.index()] >= rn.num_nodes {
                break;
            }

            self.current_edge
                .iter_mut()
                .enumerate()
                .for_each(|(u, e)| *e = rn.start[u]);
            match self.dfs(rn, source, sink, residual) {
                Some(delta) => {
                    objective_value += delta;
                    residual -= delta;
                }
                None => break,
            }
        }

        Ok(MaxFlowResult {
            objective_value,
            flows: rn.get_flows(&self.residual_capacities),
        })
    }

    fn dfs(
        &mut self,
        rn: &ResidualNetworkCore<N, F>,
        u: NodeId,
        sink: NodeId,
        upper: F,
    ) -> Option<F> {
        if u == sink {
            return Some(upper);
        }

        let mut res = F::zero();
        for arc_id in self.current_edge[u.index()]..rn.start[u.index() + 1] {
            let arc_id = ArcId(arc_id);
            self.current_edge[u.index()] = arc_id.index();

            let v = rn.to[arc_id.index()];
            let residual_capacity = self.residual_capacities[arc_id.index()];

            if !self.is_admissible_edge(rn, u, arc_id) {
                continue;
            }

            if let Some(d) = self.dfs(rn, v, sink, residual_capacity.min(upper - res)) {
                rn.push_flow(u, arc_id, d, &mut self.residual_capacities, None);
                res += d;
                if res == upper {
                    return Some(res);
                }
            }
        }
        self.current_edge[u.index()] = rn.start[u.index() + 1];
        self.distances_to_sink[u.index()] = rn.num_nodes;

        Some(res)
    }

    // O(n + m)
    // calculate the distance from u to sink in the residual network
    // if such a path does not exist, distance[u] becomes self.num_nodes
    pub(crate) fn update_distances_to_sink(
        &mut self,
        rn: &ResidualNetworkCore<N, F>,
        source: NodeId,
        sink: NodeId,
    ) {
        self.que.clear();
        self.que.push_back(sink);
        self.distances_to_sink.fill(rn.num_nodes);
        self.distances_to_sink[sink.index()] = 0;

        while let Some(v) = self.que.pop_front() {
            for arc_id in rn.neighbors(v) {
                let to = rn.to[arc_id.index()];
                let rev_arc_id = rn.rev[arc_id.index()];
                if self.residual_capacities[rev_arc_id.index()] > F::zero()
                    && self.distances_to_sink[to.index()] == rn.num_nodes
                {
                    self.distances_to_sink[to.index()] = self.distances_to_sink[v.index()] + 1;
                    if to != source {
                        self.que.push_back(to);
                    }
                }
            }
        }
    }

    #[inline]
    fn is_admissible_edge(
        &self,
        rn: &ResidualNetworkCore<N, F>,
        from: NodeId,
        arc_id: ArcId,
    ) -> bool {
        self.residual_capacities[arc_id.index()] > F::zero()
            && self.distances_to_sink[from.index()]
                == self.distances_to_sink[rn.to[arc_id.index()].index()] + 1
    }
}
