use crate::{
    algorithms::minimum_cost_flow::normalized_network::{NormalizedEdge, NormalizedNetwork},
    core::numeric::CostNum,
    graph::ids::NodeId,
};
use std::{cmp::Reverse, collections::BinaryHeap};
use crate::graph::node::Node;

#[derive(Clone, Default, PartialEq, Debug)]
pub enum EdgeState {
    #[default]
    Lower,
    Upper,
    Tree,
}

pub struct SpanningTreeStructure<F> {
    pub(crate) num_nodes: usize,
    pub(crate) num_edges: usize,
    pub(crate) excesses: Box<[F]>,

    // node
    pub parent: Box<[NodeId]>,
    pub parent_edge_id: Box<[usize]>,
    pub potential: Box<[F]>,

    // edge
    pub from: Box<[NodeId]>,
    pub to: Box<[NodeId]>,
    pub upper: Box<[F]>,
    pub cost: Box<[F]>,
    pub flow: Box<[F]>,
    pub state: Box<[EdgeState]>,
    pub(crate) is_reversed: Box<[bool]>,

    // tree structure
    pub(crate) root: NodeId,
    pub(crate) next_node_dft: Box<[NodeId]>, // next nodes in depth-first thread
    pub(crate) prev_node_dft: Box<[NodeId]>, // previous nodes in depth-first thread
    pub(crate) last_descendent_dft: Box<[NodeId]>, // last descendants in depth-first thread
    pub(crate) num_successors: Box<[usize]>, // the number of successors of the node in the tree

    pub(crate) _num_nodes_original_graph: usize,
    pub(crate) num_edges_original_graph: usize,
    pub(crate) lower_in_original_graph: Box<[F]>,
}

#[allow(dead_code)]
impl<F> SpanningTreeStructure<F>
where
    F: CostNum,
{
    pub fn new(
        graph: &NormalizedNetwork<F>,
        artificial_nodes: Option<&[NodeId]>,
        artificial_edges: Option<&[NormalizedEdge<F>]>,
        initial_flows: Option<&[F]>,
        fix_excesses: Option<&[F]>,
    ) -> Self {
        let num_nodes = graph.num_nodes() + artificial_nodes.unwrap_or(&[]).len();
        let num_edges = graph.num_edges() + artificial_edges.unwrap_or(&[]).len();

        let mut st = Self {
            num_nodes,
            num_edges,
            excesses: vec![F::zero(); num_nodes].into_boxed_slice(),

            parent: vec![NodeId(usize::MAX); num_nodes].into_boxed_slice(),
            parent_edge_id: vec![usize::MAX; num_nodes].into_boxed_slice(),
            potential: vec![F::zero(); num_nodes].into_boxed_slice(),

            from: vec![NodeId(usize::MAX); num_edges].into_boxed_slice(),
            to: vec![NodeId(usize::MAX); num_edges].into_boxed_slice(),
            upper: vec![F::zero(); num_edges].into_boxed_slice(),
            cost: vec![F::zero(); num_edges].into_boxed_slice(),
            flow: vec![F::zero(); num_edges].into_boxed_slice(),
            state: vec![EdgeState::Lower; num_edges].into_boxed_slice(),
            is_reversed: vec![false; num_edges].into_boxed_slice(),

            root: NodeId(usize::MAX),
            next_node_dft: vec![NodeId(usize::MAX); num_nodes].into_boxed_slice(),
            prev_node_dft: vec![NodeId(usize::MAX); num_nodes].into_boxed_slice(),
            last_descendent_dft: vec![NodeId(usize::MAX); num_nodes].into_boxed_slice(),
            num_successors: vec![0; num_nodes].into_boxed_slice(),

            _num_nodes_original_graph: graph.num_nodes(),
            num_edges_original_graph: graph.num_edges(),
            lower_in_original_graph: vec![F::zero(); num_edges * 2].into_boxed_slice(),
        };

        st.build(graph, artificial_nodes, artificial_edges, initial_flows, fix_excesses);
        st
    }

    fn build(
        &mut self,
        graph: &NormalizedNetwork<F>,
        _artificial_nodes: Option<&[NodeId]>,
        artificial_edges: Option<&[NormalizedEdge<F>]>,
        initial_flows: Option<&[F]>,
        fix_excesses: Option<&[F]>,
    ) {
        for (u, e) in graph.excesses().iter().enumerate() {
            self.excesses[u] = *e;
        }

        if let Some(fix) = fix_excesses {
            for u in 0..self.num_nodes {
                self.excesses[u] += fix[u];
            }
        }

        for (edge_id, edge) in graph
            .iter_edges()
            .chain(artificial_edges.into_iter().flatten().copied())
            .enumerate()
        {
            assert!(edge.upper >= F::zero() && edge.cost >= F::zero());
            let initial_flow = initial_flows.map_or(F::zero(), |init| init[edge_id]);

            self.from[edge_id] = edge.u;
            self.to[edge_id] = edge.v;
            self.flow[edge_id] = initial_flow;
            self.upper[edge_id] = edge.upper;
            self.cost[edge_id] = edge.cost;
            self.state[edge_id] = EdgeState::Lower;
            self.is_reversed[edge_id] = edge.is_reversed;
            self.lower_in_original_graph[edge_id] = edge.lower;
        }
    }

    #[inline]
    pub(crate) fn reduced_cost(&self, edge_id: usize) -> F {
        self.cost[edge_id] - self.potential[self.from[edge_id].index()] + self.potential[self.to[edge_id].index()]
    }

    pub(crate) fn update_flow_in_path(&mut self, source: NodeId, sink: NodeId, delta: F) {
        let mut now = sink;
        while now != source {
            let (parent, edge_id) = (self.parent[now.index()], self.parent_edge_id[now.index()]);
            self.flow[edge_id] += if self.from[edge_id] == parent { delta } else { -delta };
            now = parent;
        }
        self.excesses[source.index()] -= delta;
        self.excesses[sink.index()] += delta;
    }

    pub(crate) fn update_flow_in_cycle(&mut self, entering_edge_id: usize, delta: F, apex: NodeId) {
        let delta = match self.state[entering_edge_id] {
            EdgeState::Upper => -delta,
            _ => delta,
        };
        self.flow[entering_edge_id] += delta;

        let mut now = self.from[entering_edge_id];
        while now != apex {
            self.flow[self.parent_edge_id[now.index()]] += if now == self.from[self.parent_edge_id[now.index()]] {
                -delta
            } else {
                delta
            };
            now = self.parent[now.index()];
        }

        let mut now = self.to[entering_edge_id];
        while now != apex {
            self.flow[self.parent_edge_id[now.index()]] += if now == self.from[self.parent_edge_id[now.index()]] {
                delta
            } else {
                -delta
            };
            now = self.parent[now.index()];
        }
    }

    // change the root of the subtree from now_root to new_root
    // O(|tree|)
    pub(crate) fn re_rooting(&mut self, _now_root: NodeId, new_root: NodeId, entering_edge_id: usize) {
        let mut ancestors = Vec::new();
        let mut now = new_root;
        while now != NodeId(usize::MAX) {
            ancestors.push(now);
            now = self.parent[now.index()];
        }
        ancestors.reverse();

        for pair in ancestors.windows(2) {
            let (p, q) = (pair[0], pair[1]);
            let size_p = self.num_successors[p.index()];
            let last_q = self.last_descendent_dft[q.index()];

            self.parent[p.index()] = q;
            self.parent[q.index()] = NodeId(usize::MAX);
            self.parent_edge_id[p.index()] = self.parent_edge_id[q.index()];
            self.parent_edge_id[q.index()] = usize::MAX;
            self.num_successors[p.index()] = size_p - self.num_successors[q.index()];
            self.num_successors[q.index()] = size_p;

            let prev_q = self.prev_node_dft[q.index()];
            let next_last_q = self.next_node_dft[last_q.index()];
            self.next_node_dft[prev_q.index()] = next_last_q;
            self.prev_node_dft[next_last_q.index()] = prev_q;
            self.next_node_dft[last_q.index()] = q;
            self.prev_node_dft[q.index()] = last_q;

            let mut last_p = self.last_descendent_dft[p.index()];
            if last_p == last_q {
                self.last_descendent_dft[p.index()] = prev_q;
                last_p = prev_q;
            }

            self.prev_node_dft[p.index()] = last_q;
            self.next_node_dft[last_q.index()] = p;
            self.next_node_dft[last_p.index()] = q;
            self.prev_node_dft[q.index()] = last_p;
            self.last_descendent_dft[q.index()] = last_p;
        }

        // update potential
        let delta = if new_root == self.from[entering_edge_id] {
            self.reduced_cost(entering_edge_id)
        } else {
            -self.reduced_cost(entering_edge_id)
        };

        let mut now = new_root;
        while now != NodeId(usize::MAX) {
            self.potential[now.index()] += delta;
            if now == self.last_descendent_dft[new_root.index()] {
                break;
            }
            now = self.next_node_dft[now.index()];
        }
    }

    // remove leaving_edge_id
    pub(crate) fn detach_tree(&mut self, _root: NodeId, sub_tree_root: NodeId, leaving_edge_id: usize) {
        self.state[leaving_edge_id] = if self.is_lower(leaving_edge_id) {
            EdgeState::Lower
        } else {
            EdgeState::Upper
        };

        // detach sub tree
        self.parent[sub_tree_root.index()] = NodeId(usize::MAX);
        self.parent_edge_id[sub_tree_root.index()] = usize::MAX;

        let prev_t = self.prev_node_dft[sub_tree_root.index()];
        let last_t = self.last_descendent_dft[sub_tree_root.index()];
        let next_last_t = self.next_node_dft[last_t.index()];
        self.next_node_dft[prev_t.index()] = next_last_t;
        self.prev_node_dft[next_last_t.index()] = prev_t;
        self.next_node_dft[last_t.index()] = sub_tree_root;
        self.prev_node_dft[sub_tree_root.index()] = last_t;

        let sub_tree_size = self.num_successors[sub_tree_root.index()];
        let mut now = self.opposite_side(sub_tree_root, leaving_edge_id);
        while now != NodeId(usize::MAX) {
            self.num_successors[now.index()] -= sub_tree_size;
            if self.last_descendent_dft[now.index()] == last_t {
                self.last_descendent_dft[now.index()] = prev_t;
            }
            now = self.parent[now.index()];
        }
    }

    // attach T2 under T1
    // O(1)
    // add entering_ege_id
    pub(crate) fn attach_tree(
        &mut self,
        _root: NodeId,
        attach_node: NodeId,
        sub_tree_root: NodeId,
        entering_edge_id: usize,
    ) {
        self.state[entering_edge_id] = EdgeState::Tree;

        let (p, q) = (attach_node, sub_tree_root); // p -> q

        // attach tree
        self.parent[q.index()] = p;
        self.parent_edge_id[q.index()] = entering_edge_id;

        let last_p = self.last_descendent_dft[attach_node.index()];
        let next_last_p = self.next_node_dft[last_p.index()];
        let last_q = self.last_descendent_dft[q.index()];
        self.next_node_dft[last_p.index()] = q;
        self.prev_node_dft[q.index()] = last_p;
        self.prev_node_dft[next_last_p.index()] = last_q;
        self.next_node_dft[last_q.index()] = next_last_p;

        let sub_tree_size = self.num_successors[q.index()];
        let mut now = attach_node;
        while now != NodeId(usize::MAX) {
            self.num_successors[now.index()] += sub_tree_size;
            if self.last_descendent_dft[now.index()] == last_p {
                self.last_descendent_dft[now.index()] = last_q
            }
            now = self.parent[now.index()];
        }
    }

    // dijkstra
    pub(crate) fn shortest_path(&self, source: NodeId) -> (Vec<F>, Vec<Option<usize>>) {
        let mut graph = vec![Vec::new(); self.num_nodes];
        let mut total_cost = F::zero();
        for edge_id in 0..self.num_edges {
            graph[self.from[edge_id].index()].push(edge_id);
            assert!(self.cost[edge_id] >= F::zero());
            total_cost += self.cost[edge_id];
        }

        let mut distances = vec![total_cost + F::one(); self.num_nodes];
        let mut prev_edge_id = vec![None; self.num_nodes];
        let mut seen = vec![false; self.num_nodes];
        let mut bh = BinaryHeap::from([(Reverse(F::zero()), source)]);

        distances[source.index()] = F::zero();
        while let Some((now_dist, u)) = bh.pop() {
            if seen[u.index()] {
                continue;
            }
            seen[u.index()] = true;

            for &edge_id in graph[u.index()].iter() {
                let new_dist = now_dist.0 + self.cost[edge_id];

                let to = self.to[edge_id];
                if new_dist < distances[to.index()] {
                    prev_edge_id[to.index()] = Some(edge_id);
                    distances[to.index()] = new_dist;
                    bh.push((Reverse(new_dist), to));
                }
            }
        }

        (distances, prev_edge_id)
    }

    pub fn satisfy_constraints(&self) -> bool {
        for edge_id in 0..self.num_edges {
            if !self.is_feasible(edge_id) {
                return false;
            }
        }
        self.excesses.iter().all(|&excess| excess == F::zero())
    }

    pub fn satisfy_optimality_conditions(&self) -> bool {
        (0..self.num_edges).all(|edge_id| match self.state[edge_id] {
            EdgeState::Tree => self.reduced_cost(edge_id) == F::zero(),
            EdgeState::Lower => self.upper[edge_id] == F::zero() || self.reduced_cost(edge_id) >= F::zero(),
            EdgeState::Upper => self.upper[edge_id] == F::zero() || self.reduced_cost(edge_id) <= F::zero(),
        })
    }

    pub fn validate_num_successors(&self, root: NodeId) -> bool {
        let mut order = Vec::new();
        let mut now = root;
        loop {
            order.push(now);
            now = self.next_node_dft[now.index()];
            if now == root {
                break;
            }
        }

        let mut num_successors = vec![1; self.num_nodes];
        for &u in order.iter().rev() {
            if num_successors[u.index()] != self.num_successors[u.index()] {
                return false;
            }
            if self.parent[u.index()] != NodeId(usize::MAX) {
                num_successors[self.parent[u.index()].index()] += num_successors[u.index()];
            }
        }

        true
    }

    pub fn is_feasible(&self, edge_id: usize) -> bool {
        F::zero() <= self.flow[edge_id] && self.flow[edge_id] <= self.upper[edge_id]
    }

    pub fn is_lower(&self, edge_id: usize) -> bool {
        self.flow[edge_id] == F::zero()
    }

    pub fn is_upper(&self, edge_id: usize) -> bool {
        self.flow[edge_id] == self.upper[edge_id]
    }

    pub fn residual_capacity(&self, edge_id: usize) -> F {
        self.upper[edge_id] - self.flow[edge_id]
    }

    pub fn opposite_side(&self, u: NodeId, edge_id: usize) -> NodeId {
        debug_assert!(u == self.from[edge_id] || u == self.to[edge_id]);
        NodeId(u.index() ^ self.to[edge_id].index() ^ self.from[edge_id].index())
    }

    pub fn calculate_objective_value_in_original_graph(&self) -> F {
        let mut objective_value = F::zero();
        for edge_id in 0..self.num_edges_original_graph {
            let flow = self.flow[edge_id];
            if self.is_reversed[edge_id] {
                let original_flow = self.upper[edge_id] + self.lower_in_original_graph[edge_id] - flow;
                objective_value += original_flow * -self.cost[edge_id];
            } else {
                let original_flow = flow + self.lower_in_original_graph[edge_id];
                objective_value += original_flow * self.cost[edge_id];
            };
        }
        objective_value
    }

    pub fn make_minimum_cost_flow_in_original_graph(&self) -> Vec<F> {
        let mut flows = Vec::with_capacity(self.num_edges_original_graph);
        for edge_id in 0..self.num_edges_original_graph {
            let flow = self.flow[edge_id];
            if self.is_reversed[edge_id] {
                let original_flow = self.upper[edge_id] + self.lower_in_original_graph[edge_id] - flow;
                flows.push(original_flow);
            } else {
                let original_flow = flow + self.lower_in_original_graph[edge_id];
                flows.push(original_flow);
            };
        }
        flows
    }
}
