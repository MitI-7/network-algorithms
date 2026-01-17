use crate::graph::edge::Edge;
use crate::{
    algorithms::maximum_flow::{
        edge::MaximumFlowEdge,
        error::MaximumFlowError,
        residual_network::ResidualNetwork,
        solvers::{macros::impl_maximum_flow_solver, solver::MaximumFlowSolver},
        status::Status,
        validate::validate_input,
    },
    core::numeric::FlowNum,
    graph::{
        direction::Directed,
        graph::Graph,
        ids::{ArcId, EdgeId, NodeId},
    },
};
use num_traits::One;
use std::collections::VecDeque;

pub struct CapacityScaling<F> {
    status: Status,
    source: Option<NodeId>,

    rn: ResidualNetwork<F>,
    current_edge: Box<[usize]>,
    que: VecDeque<NodeId>,
    cutoff: Option<F>,
}

impl<F> CapacityScaling<F>
where
    F: FlowNum + One,
{
    fn new<N>(graph: &Graph<Directed, N, MaximumFlowEdge<F>>) -> Self {
        let rn = ResidualNetwork::from(graph, |e| e.data.upper);
        Self::new_with_residual_network(rn)
    }

    pub fn new_with<N, E, UF>(graph: &Graph<Directed, N, E>, upper_fn: UF) -> Self
    where
        UF: Fn(&Edge<E>) -> F,
    {
        let rn = ResidualNetwork::from(graph, upper_fn);
        Self::new_with_residual_network(rn)
    }

    fn new_with_residual_network(rn: ResidualNetwork<F>) -> Self {
        let num_nodes = rn.num_nodes;
        Self {
            status: Status::NotSolved,
            source: None,
            rn,
            current_edge: vec![0_usize; num_nodes].into_boxed_slice(),
            que: VecDeque::new(),
            cutoff: None,
        }
    }

    fn run(&mut self, source: NodeId, sink: NodeId) -> Result<F, MaximumFlowError> {
        validate_input(&self.rn, source, sink)?;

        self.source = Some(source);
        let max_capacity = *self.rn.upper.iter().max().unwrap_or(&F::zero());
        let mut deltas: Vec<F> = Vec::new();
        let mut d = F::one();
        while d <= max_capacity {
            deltas.push(d);
            d = d + d;
        }

        let mut residual = self.cutoff.unwrap_or_else(|| {
            self.rn
                .neighbors(source)
                .fold(F::zero(), |sum, arc_id| sum + self.rn.upper[arc_id.index()])
        });
        let mut flow = F::zero();
        for delta in deltas.into_iter().rev() {
            // solve maximum flow in delta-residual network
            loop {
                self.bfs(source, sink, delta);

                // no s-t path
                if self.rn.distances_to_sink[source.index()] >= self.rn.num_nodes {
                    break;
                }

                self.current_edge
                    .iter_mut()
                    .enumerate()
                    .for_each(|(u, e)| *e = self.rn.start[u]);
                match self.dfs(source, sink, residual, delta) {
                    Some(delta) => {
                        flow += delta;
                        residual -= delta;
                    }
                    None => break,
                }
            }
        }

        self.status = Status::Optimal;
        Ok(flow)
    }

    fn bfs(&mut self, source: NodeId, sink: NodeId, delta: F) {
        self.que.clear();
        self.que.push_back(sink);
        self.rn.distances_to_sink.fill(self.rn.num_nodes);
        self.rn.distances_to_sink[sink.index()] = 0;

        while let Some(v) = self.que.pop_front() {
            for i in self.rn.neighbors(v) {
                // e.to -> v
                let to = self.rn.to[i.index()];
                let rev = self.rn.rev[i.index()];
                if self.rn.residual_capacities[rev.index()] >= delta
                    && self.rn.distances_to_sink[to.index()] == self.rn.num_nodes
                {
                    self.rn.distances_to_sink[to.index()] = self.rn.distances_to_sink[v.index()] + 1;
                    if to != source {
                        self.que.push_back(to);
                    }
                }
            }
        }
    }

    fn dfs(&mut self, u: NodeId, sink: NodeId, upper: F, delta: F) -> Option<F> {
        if u == sink {
            return Some(upper);
        }

        let mut res = F::zero();
        for i in self.current_edge[u.index()]..self.rn.start[u.index() + 1] {
            let i = ArcId(i);
            self.current_edge[u.index()] = i.index();
            let v = self.rn.to[i.index()];
            let residual_capacity = self.rn.residual_capacities[i.index()];

            if !self.rn.is_admissible_arc(u, i) || residual_capacity < delta {
                continue;
            }

            if let Some(d) = self.dfs(v, sink, residual_capacity.min(upper - res), delta) {
                self.rn.push_flow_without_excess(u, i, d);
                res += d;
                if res == upper {
                    return Some(res);
                }
            }
        }
        self.current_edge[u.index()] = self.rn.start[u.index() + 1];
        self.rn.distances_to_sink[u.index()] = self.rn.num_nodes;

        Some(res)
    }
}

impl_maximum_flow_solver!(CapacityScaling, run, One);
