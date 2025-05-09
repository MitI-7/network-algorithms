use crate::graph::minimum_cost_flow_graph::Graph;
use crate::minimum_cost_flow::MinimumCostFlowNum;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use crate::graph::minimum_cost_flow_graph::Edge;

#[derive(Default)]
pub struct CSR<Flow> {
    pub num_nodes: usize,
    pub num_edges: usize,
    pub edge_index_to_inside_edge_index: Box<[usize]>,

    pub excesses: Box<[Flow]>,
    pub potentials: Box<[Flow]>,

    pub start: Box<[usize]>,
    pub to: Box<[usize]>,
    pub flow: Box<[Flow]>,
    pub upper: Box<[Flow]>,
    pub cost: Box<[Flow]>,
    pub rev: Box<[usize]>,

    pub is_reversed: Box<[bool]>,
}

impl<Flow> CSR<Flow>
where
    Flow: MinimumCostFlowNum,
{
    pub fn build(&mut self, graph: &Graph<Flow>, artificial_nodes: Option<&[usize]>, artificial_edges: Option<&[Edge<Flow>]>) {
        if graph.num_nodes() == 0 {
            return;
        }

        self.num_nodes = graph.num_nodes() + artificial_nodes.unwrap_or(&[]).len();
        self.num_edges = graph.num_edges() + artificial_edges.unwrap_or(&[]).len();

        // self.excesses = vec![Flow::zero(); self.num_nodes].into_boxed_slice();
        // for u in 0..self.num_nodes {
        //     self.excesses[u] = graph.excesses[u];
        // }

        self.edge_index_to_inside_edge_index = vec![usize::MAX; self.num_edges].into_boxed_slice();
        self.start = vec![0; self.num_nodes + 1].into_boxed_slice();
        self.to = vec![usize::MAX; self.num_edges * 2].into_boxed_slice();
        self.flow = vec![Flow::zero(); self.num_edges * 2].into_boxed_slice();
        self.upper = vec![Flow::zero(); self.num_edges * 2].into_boxed_slice();
        self.cost = vec![Flow::zero(); self.num_edges * 2].into_boxed_slice();
        self.rev = vec![usize::MAX; self.num_edges * 2].into_boxed_slice();
        self.potentials = vec![Flow::zero(); self.num_nodes].into_boxed_slice();

        self.make_csr(graph, artificial_nodes, artificial_edges);
    }

    fn make_csr(&mut self, graph: &Graph<Flow>, _artificial_nodes: Option<&[usize]>, artificial_edges: Option<&[Edge<Flow>]>) {
        let mut degree = vec![0; self.num_nodes];

        for edge in graph.edges.iter().chain(artificial_edges.unwrap_or(&[])) {
            degree[edge.to] += 1;
            degree[edge.from] += 1;
        }

        for i in 1..=self.num_nodes {
            self.start[i] += self.start[i - 1] + degree[i - 1];
        }

        let mut counter = vec![0; self.num_nodes];
        for (edge_index, edge) in graph.edges.iter().chain(artificial_edges.unwrap_or(&[])).enumerate() {
            assert!(edge.cost >= Flow::zero());
            assert!(edge.lower == Flow::zero());
            assert!(edge.upper >= Flow::zero());
            // assert!(edge.flow == Flow::zero());

            let (u, v) = (edge.from, edge.to);
            let inside_edge_index_u = self.start[u] + counter[u];
            counter[u] += 1;
            let inside_edge_index_v = self.start[v] + counter[v];
            self.edge_index_to_inside_edge_index[edge_index] = inside_edge_index_u;
            counter[v] += 1;

            assert_ne!(inside_edge_index_u, inside_edge_index_v);

            // u -> v
            self.to[inside_edge_index_u] = v;
            self.flow[inside_edge_index_u] = edge.flow;
            self.upper[inside_edge_index_u] = edge.upper;
            self.cost[inside_edge_index_u] = edge.cost;
            self.rev[inside_edge_index_u] = inside_edge_index_v;

            // v -> u
            self.to[inside_edge_index_v] = u;
            self.flow[inside_edge_index_v] = edge.upper - edge.flow;
            self.upper[inside_edge_index_v] = edge.upper;
            self.cost[inside_edge_index_v] = -edge.cost;
            self.rev[inside_edge_index_v] = inside_edge_index_u;
        }
    }

    pub fn set_flow(&self, graph: &mut Graph<Flow>) {
        // for u in 0..graph.num_nodes() {
        //     graph.excesses[u] = self.excesses[u];
        // }

        for edge_id in 0..graph.num_edges() {
            let i = self.edge_index_to_inside_edge_index[edge_id];
            let edge = &graph.edges[edge_id];
            graph.edges[edge_id].flow = if edge.cost >= Flow::zero() {
                self.flow[i] + edge.lower
            }
            else {
                edge.upper - self.flow[i]
            };
            assert!(graph.edges[edge_id].flow <= graph.edges[edge_id].upper);
            assert!(graph.edges[edge_id].flow >= graph.edges[edge_id].lower);
        }
    }

    #[inline]
    pub fn neighbors(&self, u: usize) -> std::ops::Range<usize> {
        self.start[u]..self.start[u + 1]
    }

    #[inline]
    pub fn push_flow(&mut self, u: usize, i: usize, flow: Flow) {
        let rev = self.rev[i];
        let to = self.to[i];
        self.flow[i] += flow;
        self.flow[rev] -= flow;
        self.excesses[u] -= flow;
        self.excesses[to] += flow;
    }

    pub fn calculate_distance_from_source(&mut self, source: usize) -> (Vec<Option<Flow>>, Vec<Option<usize>>) {
        let mut prev = vec![None; self.num_nodes];
        let mut bh = BinaryHeap::new();
        let mut dist: Vec<Option<Flow>> = vec![None; self.num_nodes];
        let mut visited = vec![false; self.num_nodes];

        bh.push((Reverse(Flow::zero()), source));
        dist[source] = Some(Flow::zero());

        while let Some((d, u)) = bh.pop() {
            if visited[u] {
                continue;
            }
            visited[u] = true;

            for edge_id in self.neighbors(u) {
                if self.residual_capacity(edge_id) == Flow::zero() {
                    continue;
                }

                let to = self.to[edge_id];
                let new_dist = d.0 + self.reduced_cost(u, edge_id);
                if dist[to].is_none() || dist[to].unwrap() > new_dist {
                    dist[to] = Some(new_dist);
                    prev[to] = Some(edge_id);
                    bh.push((Reverse(new_dist), to));
                }
            }
        }

        (dist, prev)
    }

    pub fn minimum_cost(&self) -> Flow {
        let mut c = Flow::zero();
        for i in 0..self.num_edges {
            c += self.flow[i] * self.cost[i];
        }
        c
    }

    #[inline]
    pub fn reduced_cost(&self, u: usize, i: usize) -> Flow {
        self.cost[i] - self.potentials[u] + self.potentials[self.to[i]]
    }

    #[inline]
    pub fn reduced_cost_rev(&self, u: usize, i: usize) -> Flow {
        -(self.cost[i] - self.potentials[u] + self.potentials[self.to[i]])
    }

    pub fn residual_capacity(&self, i: usize) -> Flow {
        self.upper[i] - self.flow[i]
    }

    pub fn is_feasible(&self, i: usize) -> bool {
        Flow::zero() <= self.flow[i] && self.flow[i] <= self.upper[i]
    }
}
