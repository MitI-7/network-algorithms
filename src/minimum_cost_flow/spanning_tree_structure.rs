use crate::minimum_cost_flow::network_simplex_pivot_rules::{BlockSearchPivotRule, PivotRule};
use crate::minimum_cost_flow::parametric_network_simplex::ParametricNetworkSimplex;
use crate::minimum_cost_flow::primal_network_simplex::PrimalNetworkSimplex;
use crate::minimum_cost_flow::status::Status;
use num_traits::NumAssign;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};
use std::ops::Neg;

#[derive(Clone)]
pub struct Node<Flow> {
    pub parent: usize,
    pub parent_edge_id: usize,
    pub potential: Flow,
}

#[derive(PartialEq, Debug)]
pub enum EdgeState {
    Upper,
    Tree,
    Lower,
}

#[derive(Clone, Debug)]
pub struct Edge<Flow> {
    pub from: usize,
    pub to: usize,
    pub lower: Flow,
    pub upper: Flow,
    pub cost: Flow,
    pub flow: Flow,
}

pub struct InternalEdge<Flow> {
    pub from: usize,
    pub to: usize,
    pub upper: Flow,
    pub cost: Flow,
    pub flow: Flow,
    pub state: EdgeState,
}

impl<Flow> InternalEdge<Flow>
where
    Flow: NumAssign + Neg<Output = Flow> + Ord + Copy,
{
    pub fn is_feasible(&self) -> bool {
        Flow::zero() <= self.flow && self.flow <= self.upper
    }

    pub fn is_lower(&self) -> bool {
        self.flow == Flow::zero()
    }

    pub fn is_upper(&self) -> bool {
        self.flow == self.upper
    }

    pub fn residual_capacity(&self) -> Flow {
        self.upper - self.flow
    }

    pub fn opposite_side(&self, u: usize) -> usize {
        debug_assert!(u == self.from || u == self.to);
        u ^ self.to ^ self.from
    }
}

#[derive(Default)]
pub struct SpanningTreeStructure<Flow> {
    pub(crate) num_nodes: usize,
    pub(crate) num_edges: usize,
    b: Vec<Flow>,
    pub(crate) excesses: Vec<Flow>,
    lowers: Vec<Flow>,
    is_rev: Vec<bool>,

    pub(crate) nodes: Vec<Node<Flow>>,
    pub(crate) edges: Vec<InternalEdge<Flow>>,

    pub(crate) root: usize,
    pub(crate) next_node_dft: Vec<usize>,       // next nodes in depth-first thread
    pub(crate) prev_node_dft: Vec<usize>,       // previous nodes in depth-first thread
    pub(crate) last_descendent_dft: Vec<usize>, // last descendants in depth-first thread
    pub(crate) num_successors: Vec<usize>,      // the number of successors of the node in the tree
    status: Status,
}

#[allow(dead_code)]
impl<Flow> SpanningTreeStructure<Flow>
where
    Flow: NumAssign + Neg<Output = Flow> + Ord + Copy + Clone,
{
    #[inline]
    pub fn num_nodes(&self) -> usize {
        self.num_nodes
    }

    #[inline]
    pub fn num_edges(&self) -> usize {
        self.num_edges
    }

    pub fn add_node(&mut self) -> usize {
        self.b.push(Flow::zero());
        self.excesses.push(Flow::zero());
        self.nodes.push(Node { parent: usize::MAX, parent_edge_id: usize::MAX, potential: Flow::zero() });
        self.next_node_dft.push(usize::MAX);
        self.prev_node_dft.push(usize::MAX);
        self.last_descendent_dft.push(usize::MAX);
        self.num_successors.push(0);
        self.num_nodes += 1;
        self.num_nodes - 1
    }

    pub fn add_nodes(&mut self, num_nodes: usize) -> Vec<usize> {
        self.b.extend(vec![Flow::zero(); num_nodes]);
        self.excesses.extend(vec![Flow::zero(); num_nodes]);
        self.nodes.extend(vec![Node { parent: usize::MAX, parent_edge_id: usize::MAX, potential: Flow::zero() }; num_nodes]);
        self.next_node_dft.extend(vec![usize::MAX; num_nodes]);
        self.prev_node_dft.extend(vec![usize::MAX; num_nodes]);
        self.last_descendent_dft.extend(vec![usize::MAX; num_nodes]);
        self.num_successors.extend(vec![0; num_nodes]);
        self.num_nodes += num_nodes;
        ((self.num_nodes - num_nodes)..self.num_nodes).collect()
    }

    pub fn add_directed_edge(&mut self, from: usize, to: usize, lower: Flow, upper: Flow, cost: Flow) -> Option<usize> {
        if lower > upper || from >= self.num_nodes || to >= self.num_nodes {
            return None;
        }

        if cost >= Flow::zero() {
            self.edges.push(InternalEdge { from, to, upper: upper - lower, cost, flow: Flow::zero(), state: EdgeState::Lower });
            self.excesses[from] -= lower;
            self.excesses[to] += lower;
            self.lowers.push(lower);
            self.is_rev.push(false);
        } else {
            self.edges
                .push(InternalEdge { from: to, to: from, upper: upper - lower, cost: -cost, flow: Flow::zero(), state: EdgeState::Lower });
            self.excesses[from] -= upper;
            self.excesses[to] += upper;
            self.lowers.push(lower);
            self.is_rev.push(true);
        }

        self.num_edges += 1;
        Some(self.num_edges - 1)
    }

    pub fn add_supply(&mut self, u: usize, supply: Flow) {
        self.b[u] += supply;
        self.excesses[u] += supply;
    }

    pub fn get_node(&self, u: usize) -> Option<&Node<Flow>> {
        if u >= self.nodes.len() {
            return None;
        }
        Some(&self.nodes[u])
    }

    pub fn get_edge(&self, edge_id: usize) -> Option<Edge<Flow>> {
        if edge_id >= self.edges.len() {
            return None;
        }
        let e = &self.edges[edge_id];
        let lower = self.lowers[edge_id];
        if self.is_rev[edge_id] {
            Some(Edge { from: e.to, to: e.from, lower, upper: e.upper + lower, cost: -e.cost, flow: e.upper - e.flow + lower })
        } else {
            Some(Edge { from: e.from, to: e.to, lower, upper: e.upper + lower, cost: e.cost, flow: e.flow + lower })
        }
    }

    pub fn pop_node(&mut self) {
        if self.num_nodes > 0 {
            self.b.pop();
            self.excesses.pop();
            self.nodes.pop();
            self.next_node_dft.pop();
            self.prev_node_dft.pop();
            self.last_descendent_dft.pop();
            self.num_successors.pop();
            self.num_nodes -= 1;
        }
    }

    pub fn pop_nodes(&mut self, num_nodes: usize) {
        if self.num_nodes >= num_nodes {
            self.b.truncate(self.num_nodes - num_nodes);
            self.excesses.truncate(self.num_nodes - num_nodes);
            self.nodes.truncate(self.num_nodes - num_nodes);
            self.next_node_dft.truncate(self.num_nodes - num_nodes);
            self.prev_node_dft.truncate(self.num_nodes - num_nodes);
            self.last_descendent_dft.truncate(self.num_nodes - num_nodes);
            self.num_successors.truncate(self.num_nodes - num_nodes);
            self.num_nodes = self.nodes.len();
        }
    }

    pub fn pop_edge(&mut self) {
        if self.num_edges > 0 {
            self.edges.pop();
            self.num_edges -= 1;
        }
    }

    pub fn pop_edges(&mut self, num_edges: usize) {
        if self.num_edges >= num_edges {
            self.edges.truncate(self.num_edges - num_edges);
            self.num_edges = self.edges.len();
        }
    }

    // after obtaining the optimal solution, modify the node supply and node demand
    // b[supply_node_id] += supply and b[demand_node_id] -= supply;
    pub fn change_excess(&mut self, supply_node_id: usize, demand_node_id: usize, supply: Flow) -> Status {
        if supply_node_id >= self.num_nodes || demand_node_id >= self.num_nodes {
            return Status::BadInput;
        }
        if self.status != Status::Optimal {
            return Status::BadInput;
        }

        self.b[supply_node_id] += supply;
        self.b[demand_node_id] += supply;
        self.excesses[supply_node_id] += supply;
        self.excesses[demand_node_id] += supply;

        ParametricNetworkSimplex::new(self).run();

        if self.satisfy_optimality_conditions() {
            Status::Optimal
        } else {
            Status::Infeasible
        }
    }

    // after obtaining the optimal solution, modify the edge upper
    pub fn change_edge_upper(&mut self, edge_id: usize, new_upper: Flow) -> Status {
        if edge_id >= self.num_edges {
            return Status::BadInput;
        }
        if new_upper < Flow::zero() {
            return Status::BadInput;
        }
        if self.status != Status::Optimal {
            return Status::BadInput;
        }

        let edge = &mut self.edges[edge_id];
        let (from, to) = (edge.from, edge.to);
        let lambda = new_upper - edge.upper;

        if lambda == Flow::zero() || edge.state == EdgeState::Tree || edge.state == EdgeState::Lower {
            return Status::Optimal;
        }

        if lambda > Flow::zero() {
            // increase upper
            edge.flow += lambda;
            self.change_excess(to, from, lambda)
        } else {
            // decrease upper
            let (from, to) = (edge.from, edge.to);
            edge.flow -= lambda;
            self.change_excess(from, to, lambda)
        }
    }

    // after obtaining the optimal solution, modify the edge cost
    pub fn change_edge_cost(&mut self, edge_id: usize, new_cost: Flow) -> Status {
        if edge_id >= self.num_edges {
            return Status::BadInput;
        }
        if self.status != Status::Optimal {
            return Status::BadInput;
        }

        let edge = &self.edges[edge_id];
        let lambda = new_cost - edge.cost;
        let reduced_cost = self.reduced_cost(edge);
        if lambda == Flow::zero() || (edge.state == EdgeState::Lower && reduced_cost >= Flow::zero() || (edge.state == EdgeState::Upper && reduced_cost <= Flow::zero())) {
            return Status::Optimal;
        }

        let mut pivot = BlockSearchPivotRule::new(self.num_edges);
        match edge.state {
            EdgeState::Tree => {
                // changing the cost fo edge changes some node potentials
                let (from, to) = (edge.from, edge.to);
                let (subtree_root, lambda) = if self.num_successors[to] > self.num_successors[from] { (from, lambda) } else { (to, -lambda) };
                let mut stack = VecDeque::from([subtree_root]);
                while let Some(now) = stack.pop_back() {
                    self.nodes[now].potential += lambda;
                }

                if self.satisfy_optimality_conditions() {
                    return Status::Optimal;
                }

                PrimalNetworkSimplex::new(self).run(&mut pivot, &[]);
            }
            EdgeState::Lower => {
                assert!(reduced_cost < Flow::zero());
                PrimalNetworkSimplex::new(self).run(&mut pivot, &[]);
            }
            EdgeState::Upper => {
                assert!(reduced_cost > Flow::zero());
                PrimalNetworkSimplex::new(self).run(&mut pivot, &[]);
            }
        };

        debug_assert!(self.satisfy_optimality_conditions());
        Status::Optimal
    }

    pub(crate) fn is_unbalance(&self) -> bool {
        self.excesses.iter().fold(Flow::zero(), |sum, &excess| sum + excess) != Flow::zero()
    }

    pub fn minimum_cost(&self) -> Flow {
        (0..self.num_edges).fold(Flow::zero(), |sum, edge_id| sum + self.get_edge(edge_id).unwrap().flow * self.get_edge(edge_id).unwrap().cost)
    }

    #[inline]
    pub(crate) fn reduced_cost(&self, edge: &InternalEdge<Flow>) -> Flow {
        edge.cost - self.nodes[edge.from].potential + self.nodes[edge.to].potential
    }

    // the network has one supply node (self.source) and one demand node (self.sink).
    pub(crate) fn construct_extend_network_one_supply_one_demand(&mut self) -> (usize, usize, Vec<usize>, Vec<usize>) {
        let mut artificial_edges = Vec::new();

        let excess_node_ids: Vec<usize> = self.excesses.iter().enumerate().filter(|&(_, &e)| e > Flow::zero()).map(|(u, _)| u).collect();
        let deficit_node_ids: Vec<usize> = self.excesses.iter().enumerate().filter(|&(_, &e)| e < Flow::zero()).map(|(u, _)| u).collect();

        if excess_node_ids.len() == 1 && deficit_node_ids.len() == 1 {
            return (excess_node_ids[0], deficit_node_ids[0], vec![], vec![]);
        }

        // add artificial nodes
        let source = self.add_node();
        let sink = self.add_node();

        // add artificial edges
        for u in 0..self.num_nodes {
            if u == source || u == sink {
                continue;
            }
            if self.excesses[u] > Flow::zero() {
                artificial_edges.push(self.add_directed_edge(source, u, Flow::zero(), self.excesses[u], Flow::zero()).unwrap());
                self.excesses[source] = self.excesses[source] + self.excesses[u];
            }
            if self.excesses[u] < Flow::zero() {
                artificial_edges.push(self.add_directed_edge(u, sink, Flow::zero(), -self.excesses[u], Flow::zero()).unwrap());
                self.excesses[sink] = self.excesses[sink] + self.excesses[u];
            }
            self.excesses[u] = Flow::zero();
        }

        (source, sink, vec![source, sink], artificial_edges)
    }

    pub(crate) fn construct_extend_network_feasible_solution(&mut self) -> (usize, Vec<usize>, Vec<usize>) {
        let inf_cost = self.edges.iter().map(|e| e.cost).fold(Flow::one(), |acc, cost| acc + cost); // all edge costs are non-negative

        // add artificial nodes
        let root = self.add_node();
        (self.nodes[root].parent, self.nodes[root].parent_edge_id) = (usize::MAX, usize::MAX);

        // add artificial edges
        let mut extend_edges = Vec::new();
        let mut prev_node = root;
        for u in 0..self.num_nodes {
            if u == root {
                continue;
            }

            let excess = self.excesses[u];
            if excess >= Flow::zero() {
                // u -> root
                let idx = self.add_directed_edge(u, root, Flow::zero(), excess, inf_cost).unwrap();
                (self.nodes[u].potential, self.edges[idx].flow, self.edges[idx].state) = (inf_cost, excess, EdgeState::Tree);
                extend_edges.push(idx);
            } else {
                // root -> u
                let idx = self.add_directed_edge(root, u, Flow::zero(), -excess, inf_cost).unwrap();
                (self.nodes[u].potential, self.edges[idx].flow, self.edges[idx].state) = (-inf_cost, -excess, EdgeState::Tree);
                extend_edges.push(idx);
            }

            (self.nodes[u].parent, self.nodes[u].parent_edge_id) = (root, *extend_edges.last().unwrap());
            self.next_node_dft[prev_node] = u;
            self.prev_node_dft[u] = prev_node;
            self.last_descendent_dft[u] = u;
            self.num_successors[u] = 1;
            self.excesses[u] = Flow::zero();
            prev_node = u;
        }
        self.next_node_dft[prev_node] = root;
        self.prev_node_dft[root] = prev_node;
        self.last_descendent_dft[root] = prev_node;

        self.num_successors[root] = self.num_nodes;
        (root, vec![root], extend_edges)
    }

    pub(crate) fn remove_artificial_sub_graph(&mut self, artificial_nodes: &[usize], artificial_edges: &[usize]) {
        self.pop_edges(artificial_edges.len());
        self.pop_nodes(artificial_nodes.len());
    }

    pub(crate) fn update_flow_in_path(&mut self, source: usize, sink: usize, delta: Flow) {
        let mut now = sink;
        while now != source {
            let (parent, edge_id) = (self.nodes[now].parent, self.nodes[now].parent_edge_id);
            let edge = &mut self.edges[edge_id];
            edge.flow += if edge.from == parent { delta } else { -delta };
            now = parent;
        }
        self.excesses[source] -= delta;
        self.excesses[sink] += delta;
    }

    pub(crate) fn update_flow_in_cycle(&mut self, entering_edge_id: usize, delta: Flow, apex: usize) {
        let delta = match self.edges[entering_edge_id].state {
            EdgeState::Upper => -delta,
            _ => delta,
        };
        self.edges[entering_edge_id].flow += delta;

        let mut now = self.edges[entering_edge_id].from;
        while now != apex {
            let edge = &mut self.edges[self.nodes[now].parent_edge_id];
            edge.flow += if now == edge.from { -delta } else { delta };
            now = self.nodes[now].parent;
        }

        let mut now = self.edges[entering_edge_id].to;
        while now != apex {
            let edge = &mut self.edges[self.nodes[now].parent_edge_id];
            edge.flow += if now == edge.from { delta } else { -delta };
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
        let entering_edge = &self.edges[entering_edge_id];
        let delta = if new_root == entering_edge.from {
            self.reduced_cost(entering_edge)
        } else {
            -self.reduced_cost(entering_edge)
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
        let leaving_edge = &mut self.edges[leaving_edge_id];
        leaving_edge.state = if leaving_edge.is_lower() { EdgeState::Lower } else { EdgeState::Upper };

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
        let mut now = leaving_edge.opposite_side(sub_tree_root);
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
        self.edges[entering_edge_id].state = EdgeState::Tree;

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
        for (edge_id, edge) in self.edges.iter().enumerate() {
            graph[edge.from].push(edge_id);
            assert!(edge.cost >= Flow::zero());
            total_cost += edge.cost;
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
                let edge = &self.edges[edge_id];
                let new_dist = now_dist.0 + edge.cost;

                if new_dist < distances[edge.to] {
                    prev_edge_id[edge.to] = Some(edge_id);
                    distances[edge.to] = new_dist;
                    bh.push((Reverse(new_dist), edge.to));
                }
            }
        }

        (distances, prev_edge_id)
    }

    pub fn satisfy_constraints(&self) -> bool {
        self.edges.iter().all(|edge| edge.is_feasible()) && self.excesses.iter().all(|&excess| excess == Flow::zero())
    }

    pub fn satisfy_optimality_conditions(&self) -> bool {
        self.edges.iter().all(|edge| match edge.state {
            EdgeState::Tree => self.reduced_cost(edge) == Flow::zero(),
            EdgeState::Lower => edge.upper == Flow::zero() || self.reduced_cost(edge) >= Flow::zero(),
            EdgeState::Upper => edge.upper == Flow::zero() || self.reduced_cost(edge) <= Flow::zero(),
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
}
