use crate::core::graph::Graph;
use crate::algorithms::minimum_cost_flow::MinimumCostFlowNum;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use crate::core::direction::Directed;
use crate::edge::capacity_cost::CapCostEdge;
use crate::node::excess::ExcessNode;

#[derive(Clone, Default, PartialEq, Debug)]
pub enum EdgeState {
    #[default]
    Lower,
    Upper,
    Tree,
}

#[derive(Default)]
pub struct SpanningTreeStructure<Flow> {
    pub(crate) num_nodes: usize,
    pub(crate) num_edges: usize,
    pub(crate) excesses: Box<[Flow]>,

    // node
    pub parent: Box<[usize]>,
    pub parent_edge_id: Box<[usize]>,
    pub potential: Box<[Flow]>,

    // edge
    pub from: Box<[usize]>,
    pub to: Box<[usize]>,
    pub upper: Box<[Flow]>,
    pub cost: Box<[Flow]>,
    pub flow: Box<[Flow]>,
    pub state: Box<[EdgeState]>,

    // tree structure
    pub(crate) root: usize,
    pub(crate) next_node_dft: Box<[usize]>,       // next nodes in depth-first thread
    pub(crate) prev_node_dft: Box<[usize]>,       // previous nodes in depth-first thread
    pub(crate) last_descendent_dft: Box<[usize]>, // last descendants in depth-first thread
    pub(crate) num_successors: Box<[usize]>,      // the number of successors of the node in the tree
}

#[allow(dead_code)]
impl<Flow> SpanningTreeStructure<Flow>
where
    Flow: MinimumCostFlowNum,
{
    pub(crate) fn build(&mut self, graph: &mut Graph<Directed, ExcessNode<Flow>, CapCostEdge<Flow>>) {
        (self.num_nodes, self.num_edges) = (graph.num_nodes(), graph.num_edges());
        // self.excesses = graph.excesses.clone().into_boxed_slice();
        // self.excesses = graph.b.clone().into_boxed_slice();
        let mut e = Vec::new();
        for u in 0..self.num_nodes {
            e.push(graph.nodes[u].b);
        }
        self.excesses = e.into_boxed_slice();

        self.parent = vec![usize::MAX; self.num_nodes].into_boxed_slice();
        self.parent_edge_id = vec![usize::MAX; self.num_nodes].into_boxed_slice();
        self.potential = vec![Flow::zero(); self.num_nodes].into_boxed_slice();

        self.from = vec![0; self.num_edges].into_boxed_slice();
        self.to = vec![0; self.num_edges].into_boxed_slice();
        self.upper = vec![Flow::zero(); self.num_edges].into_boxed_slice();
        self.cost = vec![Flow::zero(); self.num_edges].into_boxed_slice();
        self.flow = vec![Flow::zero(); self.num_edges].into_boxed_slice();
        self.state = vec![EdgeState::Lower; self.num_edges].into_boxed_slice();

        for (i, edge) in graph.edges.iter().enumerate() {
            assert!(edge.data.upper >= Flow::zero() && edge.data.cost >= Flow::zero());
            self.from[i] = edge.u.index();
            self.to[i] = edge.v.index();
            self.flow[i] = edge.data.flow;
            self.upper[i] = edge.data.upper;
            self.cost[i] = edge.data.cost;
            self.state[i] = EdgeState::Lower;
        }

        self.root = usize::MAX;
        self.next_node_dft = vec![usize::MAX; self.num_nodes].into_boxed_slice();
        self.prev_node_dft = vec![usize::MAX; self.num_nodes].into_boxed_slice();
        self.last_descendent_dft = vec![usize::MAX; self.num_nodes].into_boxed_slice();
        self.num_successors = vec![0; self.num_nodes].into_boxed_slice();
    }

    #[inline]
    pub(crate) fn reduced_cost(&self, edge_id: usize) -> Flow {
        self.cost[edge_id] - self.potential[self.from[edge_id]] + self.potential[self.to[edge_id]]
    }

    pub(crate) fn update_flow_in_path(&mut self, source: usize, sink: usize, delta: Flow) {
        let mut now = sink;
        while now != source {
            let (parent, edge_id) = (self.parent[now], self.parent_edge_id[now]);
            self.flow[edge_id] += if self.from[edge_id] == parent { delta } else { -delta };
            now = parent;
        }
        self.excesses[source] -= delta;
        self.excesses[sink] += delta;
    }

    pub(crate) fn update_flow_in_cycle(&mut self, entering_edge_id: usize, delta: Flow, apex: usize) {
        let delta = match self.state[entering_edge_id] {
            EdgeState::Upper => -delta,
            _ => delta,
        };
        self.flow[entering_edge_id] += delta;

        let mut now = self.from[entering_edge_id];
        while now != apex {
            self.flow[self.parent_edge_id[now]] += if now == self.from[self.parent_edge_id[now]] { -delta } else { delta };
            now = self.parent[now];
        }

        let mut now = self.to[entering_edge_id];
        while now != apex {
            self.flow[self.parent_edge_id[now]] += if now == self.from[self.parent_edge_id[now]] { delta } else { -delta };
            now = self.parent[now];
        }
    }

    // change the root of the subtree from now_root to new_root
    // O(|tree|)
    pub(crate) fn re_rooting(&mut self, _now_root: usize, new_root: usize, entering_edge_id: usize) {
        let mut ancestors = Vec::new();
        let mut now = new_root;
        while now != usize::MAX {
            ancestors.push(now);
            now = self.parent[now];
        }
        ancestors.reverse();

        for pair in ancestors.windows(2) {
            let (p, q) = (pair[0], pair[1]);
            let size_p = self.num_successors[p];
            let last_q = self.last_descendent_dft[q];

            self.parent[p] = q;
            self.parent[q] = usize::MAX;
            self.parent_edge_id[p] = self.parent_edge_id[q];
            self.parent_edge_id[q] = usize::MAX;
            self.num_successors[p] = size_p - self.num_successors[q];
            self.num_successors[q] = size_p;

            let prev_q = self.prev_node_dft[q];
            let next_last_q = self.next_node_dft[last_q];
            self.next_node_dft[prev_q] = next_last_q;
            self.prev_node_dft[next_last_q] = prev_q;
            self.next_node_dft[last_q] = q;
            self.prev_node_dft[q] = last_q;

            let mut last_p = self.last_descendent_dft[p];
            if last_p == last_q {
                self.last_descendent_dft[p] = prev_q;
                last_p = prev_q;
            }

            self.prev_node_dft[p] = last_q;
            self.next_node_dft[last_q] = p;
            self.next_node_dft[last_p] = q;
            self.prev_node_dft[q] = last_p;
            self.last_descendent_dft[q] = last_p;
        }

        // update potential
        let delta = if new_root == self.from[entering_edge_id] {
            self.reduced_cost(entering_edge_id)
        } else {
            -self.reduced_cost(entering_edge_id)
        };

        let mut now = new_root;
        while now != usize::MAX {
            self.potential[now] += delta;
            if now == self.last_descendent_dft[new_root] {
                break;
            }
            now = self.next_node_dft[now];
        }
    }

    // remove leaving_edge_id
    pub(crate) fn detach_tree(&mut self, _root: usize, sub_tree_root: usize, leaving_edge_id: usize) {
        self.state[leaving_edge_id] = if self.is_lower(leaving_edge_id) { EdgeState::Lower } else { EdgeState::Upper };

        // detach sub tree
        self.parent[sub_tree_root] = usize::MAX;
        self.parent_edge_id[sub_tree_root] = usize::MAX;

        let prev_t = self.prev_node_dft[sub_tree_root];
        let last_t = self.last_descendent_dft[sub_tree_root];
        let next_last_t = self.next_node_dft[last_t];
        self.next_node_dft[prev_t] = next_last_t;
        self.prev_node_dft[next_last_t] = prev_t;
        self.next_node_dft[last_t] = sub_tree_root;
        self.prev_node_dft[sub_tree_root] = last_t;

        let sub_tree_size = self.num_successors[sub_tree_root];
        let mut now = self.opposite_side(sub_tree_root, leaving_edge_id);
        while now != usize::MAX {
            self.num_successors[now] -= sub_tree_size;
            if self.last_descendent_dft[now] == last_t {
                self.last_descendent_dft[now] = prev_t;
            }
            now = self.parent[now];
        }
    }

    // attach T2 under T1
    // O(1)
    // add entering_ege_id
    pub(crate) fn attach_tree(&mut self, _root: usize, attach_node: usize, sub_tree_root: usize, entering_edge_id: usize) {
        self.state[entering_edge_id] = EdgeState::Tree;

        let (p, q) = (attach_node, sub_tree_root); // p -> q

        // attach tree
        self.parent[q] = p;
        self.parent_edge_id[q] = entering_edge_id;

        let last_p = self.last_descendent_dft[attach_node];
        let next_last_p = self.next_node_dft[last_p];
        let last_q = self.last_descendent_dft[q];
        self.next_node_dft[last_p] = q;
        self.prev_node_dft[q] = last_p;
        self.prev_node_dft[next_last_p] = last_q;
        self.next_node_dft[last_q] = next_last_p;

        let sub_tree_size = self.num_successors[q];
        let mut now = attach_node;
        while now != usize::MAX {
            self.num_successors[now] += sub_tree_size;
            if self.last_descendent_dft[now] == last_p {
                self.last_descendent_dft[now] = last_q
            }
            now = self.parent[now];
        }
    }

    // dijkstra
    pub(crate) fn shortest_path(&self, source: usize) -> (Vec<Flow>, Vec<Option<usize>>) {
        let mut graph = vec![Vec::new(); self.num_nodes];
        let mut total_cost = Flow::zero();
        for edge_id in 0..self.num_edges {
            graph[self.from[edge_id]].push(edge_id);
            assert!(self.cost[edge_id] >= Flow::zero());
            total_cost += self.cost[edge_id];
        }

        let mut distances = vec![total_cost + Flow::one(); self.num_nodes];
        let mut prev_edge_id = vec![None; self.num_nodes];
        let mut seen = vec![false; self.num_nodes];
        let mut bh = BinaryHeap::from([(Reverse(Flow::zero()), source)]);

        distances[source] = Flow::zero();
        while let Some((now_dist, u)) = bh.pop() {
            if seen[u] {
                continue;
            }
            seen[u] = true;

            for &edge_id in graph[u].iter() {
                let new_dist = now_dist.0 + self.cost[edge_id];

                let to = self.to[edge_id];
                if new_dist < distances[to] {
                    prev_edge_id[to] = Some(edge_id);
                    distances[to] = new_dist;
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
        self.excesses.iter().all(|&excess| excess == Flow::zero())
    }

    pub fn satisfy_optimality_conditions(&self) -> bool {
        (0..self.num_edges).all(|edge_id| match self.state[edge_id] {
            EdgeState::Tree => self.reduced_cost(edge_id) == Flow::zero(),
            EdgeState::Lower => self.upper[edge_id] == Flow::zero() || self.reduced_cost(edge_id) >= Flow::zero(),
            EdgeState::Upper => self.upper[edge_id] == Flow::zero() || self.reduced_cost(edge_id) <= Flow::zero(),
        })
    }

    pub fn validate_num_successors(&self, root: usize) -> bool {
        let mut order = Vec::new();
        let mut now = root;
        loop {
            order.push(now);
            now = self.next_node_dft[now];
            if now == root {
                break;
            }
        }

        let mut num_successors = vec![1; self.num_nodes];
        for &u in order.iter().rev() {
            if num_successors[u] != self.num_successors[u] {
                return false;
            }
            if self.parent[u] != usize::MAX {
                num_successors[self.parent[u]] += num_successors[u];
            }
        }

        true
    }

    pub fn is_feasible(&self, edge_id: usize) -> bool {
        Flow::zero() <= self.flow[edge_id] && self.flow[edge_id] <= self.upper[edge_id]
    }

    pub fn is_lower(&self, edge_id: usize) -> bool {
        self.flow[edge_id] == Flow::zero()
    }

    pub fn is_upper(&self, edge_id: usize) -> bool {
        self.flow[edge_id] == self.upper[edge_id]
    }

    pub fn residual_capacity(&self, edge_id: usize) -> Flow {
        self.upper[edge_id] - self.flow[edge_id]
    }

    pub fn opposite_side(&self, u: usize, edge_id: usize) -> usize {
        debug_assert!(u == self.from[edge_id] || u == self.to[edge_id]);
        u ^ self.to[edge_id] ^ self.from[edge_id]
    }
}
