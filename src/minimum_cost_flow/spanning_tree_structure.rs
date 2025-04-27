use crate::minimum_cost_flow::graph::Graph;
use crate::minimum_cost_flow::spanning_tree_structure::EdgeState::Lower;
use num_traits::NumAssign;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::ops::Neg;

#[derive(Default, Clone)]
pub struct Node<Flow> {
    pub parent: usize,
    pub parent_edge_id: usize,
    pub potential: Flow,
}

#[derive(Clone, Default, PartialEq, Debug)]
pub enum EdgeState {
    #[default]
    Lower,
    Upper,
    Tree,
}

// #[derive(Default)]
// pub struct InternalEdge<Flow> {
//     pub from: usize,
//     pub to: usize,
//     pub upper: Flow,
//     pub cost: Flow,
//     pub flow: Flow,
//     pub state: EdgeState,
// }
//
// impl<Flow> InternalEdge<Flow>
// where
//     Flow: NumAssign + Neg<Output = Flow> + Ord + Copy,
// {
//     pub fn is_feasible(&self) -> bool {
//         Flow::zero() <= self.flow && self.flow <= self.upper
//     }
//
//     pub fn is_lower(&self) -> bool {
//         self.flow == Flow::zero()
//     }
//
//     pub fn is_upper(&self) -> bool {
//         self.flow == self.upper
//     }
//
//     pub fn residual_capacity(&self) -> Flow {
//         self.upper - self.flow
//     }
//
//     pub fn opposite_side(&self, u: usize) -> usize {
//         debug_assert!(u == self.from || u == self.to);
//         u ^ self.to ^ self.from
//     }
// }

#[derive(Default)]
pub struct SpanningTreeStructure<Flow> {
    pub(crate) num_nodes: usize,
    pub(crate) num_edges: usize,
    pub(crate) excesses: Vec<Flow>,

    pub(crate) nodes: Box<[Node<Flow>]>,
    pub from: Box<[usize]>,
    pub to: Box<[usize]>,
    pub upper: Box<[Flow]>,
    pub cost: Box<[Flow]>,
    pub flow: Box<[Flow]>,
    pub state: Box<[EdgeState]>,

    pub(crate) root: usize,
    pub(crate) next_node_dft: Box<[usize]>,       // next nodes in depth-first thread
    pub(crate) prev_node_dft: Box<[usize]>,       // previous nodes in depth-first thread
    pub(crate) last_descendent_dft: Box<[usize]>, // last descendants in depth-first thread
    pub(crate) num_successors: Box<[usize]>,      // the number of successors of the node in the tree
}

#[allow(dead_code)]
impl<Flow> SpanningTreeStructure<Flow>
where
    Flow: NumAssign + Neg<Output = Flow> + Ord + Copy + Clone,
{
    pub(crate) fn build(&mut self, graph: &mut Graph<Flow>) {
        (self.num_nodes, self.num_edges) = (graph.num_nodes(), graph.num_edges());
        self.excesses = graph.excesses.clone();

        self.from = vec![0; graph.num_edges()].into_boxed_slice();
        self.to = vec![0; graph.num_edges()].into_boxed_slice();
        self.upper = vec![Flow::zero(); graph.num_edges()].into_boxed_slice();
        self.cost = vec![Flow::zero(); graph.num_edges()].into_boxed_slice();
        self.flow = vec![Flow::zero(); graph.num_edges()].into_boxed_slice();
        self.state = vec![Lower; graph.num_edges()].into_boxed_slice();

        for (i, edge) in graph.edges.iter().enumerate() {
            assert!(edge.upper >= Flow::zero() && edge.cost >= Flow::zero());
            self.from[i] = edge.from;
            self.to[i] = edge.to;
            self.flow[i] = edge.flow;
            self.upper[i] = edge.upper;
            self.cost[i] = edge.cost;
            self.state[i] = Lower;
        }

        self.root = usize::MAX;
        self.nodes = vec![Node { parent: usize::MAX, parent_edge_id: usize::MAX, potential: Flow::zero() }; self.num_nodes].into_boxed_slice();
        self.next_node_dft = vec![usize::MAX; self.num_nodes].into_boxed_slice();
        self.prev_node_dft = vec![usize::MAX; self.num_nodes].into_boxed_slice();
        self.last_descendent_dft = vec![usize::MAX; self.num_nodes].into_boxed_slice();
        self.num_successors = vec![0; self.num_nodes].into_boxed_slice();
    }

    #[inline]
    pub(crate) fn reduced_cost(&self, edge_id: usize) -> Flow {
        self.cost[edge_id] - self.nodes[self.from[edge_id]].potential + self.nodes[self.to[edge_id]].potential
    }

    pub(crate) fn update_flow_in_path(&mut self, source: usize, sink: usize, delta: Flow) {
        let mut now = sink;
        while now != source {
            let (parent, edge_id) = (self.nodes[now].parent, self.nodes[now].parent_edge_id);
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
            self.flow[self.nodes[now].parent_edge_id] += if now == self.from[self.nodes[now].parent_edge_id] {
                -delta
            } else {
                delta
            };
            now = self.nodes[now].parent;
        }

        let mut now = self.to[entering_edge_id];
        while now != apex {
            self.flow[self.nodes[now].parent_edge_id] += if now == self.from[self.nodes[now].parent_edge_id] {
                delta
            } else {
                -delta
            };
            now = self.nodes[now].parent;
        }
    }

    // change the root of subtree from now_root to new_root
    // O(|tree|)
    pub(crate) fn re_rooting(&mut self, _now_root: usize, new_root: usize, entering_edge_id: usize) {
        let mut ancestors = Vec::new();
        let mut now = new_root;
        while now != usize::MAX {
            ancestors.push(now);
            now = self.nodes[now].parent;
        }
        ancestors.reverse();

        for pair in ancestors.windows(2) {
            let (p, q) = (pair[0], pair[1]);
            let size_p = self.num_successors[p];
            let last_q = self.last_descendent_dft[q];

            self.nodes[p].parent = q;
            self.nodes[q].parent = usize::MAX;
            self.nodes[p].parent_edge_id = self.nodes[q].parent_edge_id;
            self.nodes[q].parent_edge_id = usize::MAX;
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
            self.nodes[now].potential += delta;
            if now == self.last_descendent_dft[new_root] {
                break;
            }
            now = self.next_node_dft[now];
        }
    }

    // remove leaving_edge_id
    pub(crate) fn detach_tree(&mut self, _root: usize, sub_tree_root: usize, leaving_edge_id: usize) {
        self.state[leaving_edge_id] = if self.is_lower(leaving_edge_id) {
            EdgeState::Lower
        } else {
            EdgeState::Upper
        };

        // detach sub tree
        self.nodes[sub_tree_root].parent = usize::MAX;
        self.nodes[sub_tree_root].parent_edge_id = usize::MAX;

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
            now = self.nodes[now].parent;
        }
    }

    // attach T2 under T1
    // O(1)
    // add entering_ege_id
    pub(crate) fn attach_tree(&mut self, _root: usize, attach_node: usize, sub_tree_root: usize, entering_edge_id: usize) {
        self.state[entering_edge_id] = EdgeState::Tree;

        let (p, q) = (attach_node, sub_tree_root); // p -> q

        // attach tree
        self.nodes[q].parent = p;
        self.nodes[q].parent_edge_id = entering_edge_id;

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
            now = self.nodes[now].parent;
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
        (0..self.num_edges).into_iter().all(|edge_id| match self.state[edge_id] {
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
            if self.nodes[u].parent != usize::MAX {
                num_successors[self.nodes[u].parent] += num_successors[u];
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
