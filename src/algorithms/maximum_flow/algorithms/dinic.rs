use crate::{
    algorithms::maximum_flow::{
        algorithms::solver::{BuildMaximumFlowSolver, MaximumFlowSolver},
        edge::MaximumFlowEdge,
        residual_network_core::ResidualNetworkCore,
        result::MaxFlowResult,
        status::Status,
        validate::validate_input,
    },
    core::numeric::FlowNum,
    graph::{
        direction::Directed,
        graph::Graph,
        ids::{ArcId, NodeId},
    },
};
use std::{collections::VecDeque, marker::PhantomData};

#[derive(Default)]
pub struct Dinic<N, F> {
    rn: ResidualNetworkCore<N, F>,
    residual_capacities: Box<[F]>,
    current_edge: Box<[usize]>,
    distances_to_sink: Box<[usize]>,
    que: VecDeque<NodeId>,
    phantom: PhantomData<N>,
}
impl<N, F> BuildMaximumFlowSolver<N, F> for Dinic<N, F>
where
    F: FlowNum,
{
    fn new(graph: &Graph<Directed, N, MaximumFlowEdge<F>>) -> Self {
        Dinic::new(graph)
    }
}

impl<N, F> MaximumFlowSolver<N, F> for Dinic<N, F>
where
    F: FlowNum,
{
    fn solve(&mut self, source: NodeId, sink: NodeId, upper: Option<F>) -> Result<MaxFlowResult<F>, Status> {
        Dinic::run(self, source, sink, upper)
    }
}

impl<N, F> Dinic<N, F>
where
    F: FlowNum,
{
    fn new(graph: &Graph<Directed, N, MaximumFlowEdge<F>>) -> Self {
        let rn = ResidualNetworkCore::from_graph(graph);
        let num_nodes = rn.num_nodes;
        let num_edges = rn.num_edges;

        Self {
            rn,
            residual_capacities: vec![F::zero(); num_edges * 2].into_boxed_slice(),
            current_edge: vec![0_usize; num_nodes].into_boxed_slice(),
            distances_to_sink: vec![0; num_nodes].into_boxed_slice(),
            que: VecDeque::new(),
            phantom: PhantomData,
        }
    }

    fn run(&mut self, source: NodeId, sink: NodeId, upper: Option<F>) -> Result<MaxFlowResult<F>, Status> {
        validate_input(&self.rn, source, sink)?;

        self.residual_capacities.copy_from_slice(&self.rn.upper);

        let mut residual = upper.unwrap_or_else(|| {
            self.rn
                .neighbors(source)
                .fold(F::zero(), |sum, arc_id| sum + self.rn.upper[arc_id.index()])
        });
        let mut objective_value = F::zero();
        while residual > F::zero() {
            self.update_distances_to_sink(source, sink);

            // no s-t path
            if self.distances_to_sink[source.index()] >= self.rn.num_nodes {
                break;
            }

            self.current_edge
                .iter_mut()
                .enumerate()
                .for_each(|(u, e)| *e = self.rn.start[u]);
            match self.dfs(source, sink, residual) {
                Some(delta) => {
                    objective_value += delta;
                    residual -= delta;
                }
                None => break,
            }
        }

        Ok(MaxFlowResult { objective_value, flows: self.rn.get_flows(&self.residual_capacities) })
    }

    fn dfs(&mut self, u: NodeId, sink: NodeId, upper: F) -> Option<F> {
        if u == sink {
            return Some(upper);
        }

        let mut res = F::zero();
        for arc_id in self.current_edge[u.index()]..self.rn.start[u.index() + 1] {
            let arc_id = ArcId(arc_id);
            self.current_edge[u.index()] = arc_id.index();

            let v = self.rn.to[arc_id.index()];
            let residual_capacity = self.residual_capacities[arc_id.index()];

            if !self.is_admissible_edge(u, arc_id) {
                continue;
            }

            if let Some(d) = self.dfs(v, sink, residual_capacity.min(upper - res)) {
                self.rn.push_flow(u, arc_id, d, &mut self.residual_capacities, None);
                res += d;
                if res == upper {
                    return Some(res);
                }
            }
        }
        self.current_edge[u.index()] = self.rn.start[u.index() + 1];
        self.distances_to_sink[u.index()] = self.rn.num_nodes;

        Some(res)
    }

    // O(n + m)
    // calculate the distance from u to sink in the residual network
    // if such a path does not exist, distance[u] becomes self.num_nodes
    pub(crate) fn update_distances_to_sink(&mut self, source: NodeId, sink: NodeId) {
        self.que.clear();
        self.que.push_back(sink);
        self.distances_to_sink.fill(self.rn.num_nodes);
        self.distances_to_sink[sink.index()] = 0;

        while let Some(v) = self.que.pop_front() {
            for arc_id in self.rn.neighbors(v) {
                let to = self.rn.to[arc_id.index()];
                let rev_arc_id = self.rn.rev[arc_id.index()];
                if self.residual_capacities[rev_arc_id.index()] > F::zero()
                    && self.distances_to_sink[to.index()] == self.rn.num_nodes
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
    fn is_admissible_edge(&self, from: NodeId, arc_id: ArcId) -> bool {
        self.residual_capacities[arc_id.index()] > F::zero()
            && self.distances_to_sink[from.index()] == self.distances_to_sink[self.rn.to[arc_id.index()].index()] + 1
    }
}
