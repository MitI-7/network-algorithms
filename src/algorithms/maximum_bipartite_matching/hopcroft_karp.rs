use crate::{
    graph::{bipartite_graph::BipartiteGraph, direction::Undirected},
    ids::{EdgeId, NodeId},
};
use std::collections::VecDeque;

#[derive(Default)]
pub enum WarmStart {
    #[default]
    None,
    Greedy,
    KarpSipser,
    UserDefined(Vec<usize>),
}

/// Bipartite Max Cardinality Matching algorithm
///
/// Bipartite Max Cardinality Matching algorithm. This class implements
/// the Hopcroft-Karp algorithm which has \f$ O(e\sqrt{n}) \f$ time
/// complexity.
///
/// \note In several cases the push-relabel based solvers have
/// better runtime performance than the augmenting path based ones.
#[derive(Default)]
pub struct HopcroftKarp {
    num_left_nodes: usize,
    num_right_nodes: usize,
    mate: Box<[Option<NodeId>]>, // mate[right_node] = Some(left_node)
    distances: Box<[usize]>,

    // csr format(left -> right)
    start: Box<[usize]>,
    to: Box<[NodeId]>,

    warm_start: WarmStart,
    queue: VecDeque<NodeId>,
}

impl HopcroftKarp {
    pub fn set_warm_start_user(mut self, matching: &[usize]) -> Self {
        self.warm_start = WarmStart::UserDefined(matching.to_vec());
        self
    }

    pub fn set_warm_start(mut self, warm_start: WarmStart) -> Self {
        self.warm_start = warm_start;
        self
    }

    pub fn solve(&mut self, graph: &BipartiteGraph<Undirected, (), ()>) -> Vec<usize> {
        self.preprocess(graph);

        match &self.warm_start {
            WarmStart::None => {}
            WarmStart::Greedy => {
                self.initial_solution_greedy(&graph.degree_left, &graph.degree_right);
            }
            WarmStart::KarpSipser => {
                self.initial_solution_karp_sipser(graph);
            }
            WarmStart::UserDefined(initial_matching) => {
                for &edge_id in initial_matching.iter() {
                    let edge = &graph.edges[edge_id];
                    self.mate[edge.v.index()] = Some(edge.u);
                }
            }
        }

        loop {
            if !self.update_distances() {
                break;
            }

            for u in (0..self.num_left_nodes).map(NodeId) {
                if self.distances[u.index()] == 0 {
                    self.dfs(u);
                }
            }
        }

        let mut matching = Vec::new();
        let (mut used_u, mut used_v) =
            (vec![false; self.num_left_nodes].into_boxed_slice(), vec![false; self.num_right_nodes].into_boxed_slice());
        for (edge_id, edge) in graph.edges.iter().enumerate() {
            // for multiple edge
            if used_u[edge.u.index()] || used_v[edge.v.index()] {
                continue;
            }

            if self.mate[edge.v.index()] == Some(edge.u) {
                matching.push(edge_id);
                used_u[edge.u.index()] = true;
                used_v[edge.v.index()] = true;
            }
        }

        matching
    }

    fn preprocess(&mut self, graph: &BipartiteGraph<Undirected, (), ()>) {
        self.num_left_nodes = graph.num_left_nodes();
        self.num_right_nodes = graph.num_right_nodes();

        self.start = vec![0; self.num_left_nodes + 1].into_boxed_slice();
        self.to = (0..graph.edges.len()).map(|_| NodeId(0)).collect();
        self.mate = vec![None; self.num_right_nodes].into_boxed_slice();
        self.distances = vec![0_usize; self.num_left_nodes].into_boxed_slice();
        self.queue = VecDeque::with_capacity(self.num_left_nodes);

        // make csr format
        for u in 1..=self.num_left_nodes {
            self.start[u] += self.start[u - 1] + graph.degree_left[u - 1];
        }

        let mut count = vec![0; self.num_left_nodes].into_boxed_slice();
        for edge in graph.edges.iter() {
            self.to[self.start[edge.u.index()] + count[edge.u.index()]] = edge.v;
            count[edge.u.index()] += 1;
        }
    }

    // make initial matching(greedy)
    fn initial_solution_greedy(&mut self, degree_u: &[usize], degree_v: &[usize]) {
        let mut deg_u: Vec<_> = (0..self.num_left_nodes).map(|u| (degree_u[u], NodeId(u))).collect();
        deg_u.sort_unstable();

        for (_, u) in deg_u {
            let mut best_v: Option<NodeId> = None;
            for i in self.neighbors(u).map(EdgeId) {
                let v = self.to[i.index()];
                if self.mate[v.index()].is_none()
                    && (best_v.is_none() || degree_v[v.index()] < degree_v[best_v.unwrap().index()])
                {
                    best_v = Some(v);
                }
            }

            if let Some(best_v) = best_v {
                self.mate[best_v.index()] = Some(u);
            }
        }
    }

    // O(m)
    fn initial_solution_karp_sipser(&mut self, graph: &BipartiteGraph<Undirected, (), ()>) {
        // make csr format(right -> left)
        let mut start_r = vec![0; self.num_right_nodes + 1].into_boxed_slice();
        let mut to_r: Box<[usize]> = (0..graph.edges.len()).map(|_| 0).collect();

        for v in 1..=self.num_right_nodes {
            start_r[v] += start_r[v - 1] + graph.degree_right[v - 1];
        }

        let mut count = vec![0; self.num_right_nodes].into_boxed_slice();
        for edge in graph.edges.iter() {
            to_r[start_r[edge.v.index()] + count[edge.v.index()]] = edge.u.index();
            count[edge.v.index()] += 1;
        }

        let mut degree_left = graph.degree_left.clone();
        let mut degree_right = graph.degree_right.clone();
        let mut used_left = vec![false; self.num_left_nodes];
        let mut used_right = vec![false; self.num_right_nodes];

        let mut que = VecDeque::with_capacity(self.num_left_nodes + self.num_right_nodes);
        let iter_left = degree_left
            .iter()
            .enumerate()
            .filter_map(|(u, &d)| (d == 1).then_some(u));
        let iter_right = degree_right
            .iter()
            .enumerate()
            .filter_map(|(v, &d)| (d == 1).then_some(self.num_left_nodes + v));
        que.extend(iter_left.chain(iter_right));

        // phase-1
        while let Some(node_id) = que.pop_front() {
            let node_id = NodeId(node_id);
            if node_id.index() < self.num_left_nodes {
                let u = node_id;
                if used_left[u.index()] || degree_left[u.index()] != 1 {
                    continue;
                }

                let v = match self.neighbors(u).find(|&i| !used_right[self.to[i].index()]) {
                    Some(i) => self.to[i],
                    None => continue,
                };

                self.mate[v.index()] = Some(u);
                used_left[u.index()] = true;
                used_right[v.index()] = true;

                for i in start_r[v.index()]..start_r[v.index() + 1] {
                    let u2 = to_r[i];
                    if !used_left[u2] {
                        degree_left[u2] -= 1;
                        if degree_left[u2] == 1 {
                            que.push_back(u2);
                        }
                    }
                }
            } else {
                let v = node_id.index() - self.num_left_nodes;
                if used_right[v] || degree_right[v] != 1 {
                    continue;
                }

                let u = match (start_r[v]..start_r[v + 1]).find(|&i| !used_left[to_r[i]]) {
                    Some(i) => to_r[i],
                    None => continue,
                };

                self.mate[v] = Some(NodeId(u));
                used_left[u] = true;
                used_right[v] = true;

                for i in self.neighbors(NodeId(u)) {
                    let v2 = self.to[i];
                    if !used_right[v2.index()] {
                        degree_right[v2.index()] -= 1;
                        if degree_right[v2.index()] == 1 {
                            que.push_back(self.num_left_nodes + v2.index());
                        }
                    }
                }
            }
        }

        // phase-2 greedy
        let mut nodes: Vec<_> = (0..self.num_left_nodes)
            .map(NodeId)
            .filter(|&u| !used_left[u.index()])
            .collect();
        nodes.sort_unstable_by_key(|&u| degree_left[u.index()]);

        for u in nodes {
            assert!(!used_left[u.index()]);
            let mut best_v: Option<NodeId> = None;
            for i in self.neighbors(u) {
                let v = self.to[i];
                if self.mate[v.index()].is_none()
                    && (best_v.is_none() || degree_right[v.index()] < degree_right[best_v.unwrap().index()])
                {
                    best_v = Some(v);
                }
            }

            if let Some(best_v) = best_v {
                self.mate[best_v.index()] = Some(u);
            }
        }
    }

    fn update_distances(&mut self) -> bool {
        // initialize
        self.distances.fill(0);
        for &u in self.mate.iter().flatten() {
            self.distances[u.index()] = usize::MAX;
        }

        self.queue.clear();
        for (u, &d) in self.distances.iter().enumerate() {
            if d == 0 {
                self.queue.push_back(NodeId(u));
            }
        }

        let mut found = false;
        while let Some(u1) = self.queue.pop_front() {
            for i in self.neighbors(u1) {
                let v = self.to[i];
                match self.mate[v.index()] {
                    Some(u2) => {
                        // u1 -> v -> u2
                        if self.distances[u2.index()] == usize::MAX {
                            self.distances[u2.index()] = self.distances[u1.index()] + 1;
                            self.queue.push_back(u2);
                        }
                    }
                    None => {
                        // find an augmenting path
                        found = true;
                    }
                }
            }
        }
        found
    }

    fn dfs(&mut self, u: NodeId) -> bool {
        let now_dist = std::mem::replace(&mut self.distances[u.index()], usize::MAX); // use node u

        for i in self.neighbors(u).map(EdgeId) {
            let v = self.to[i.index()];
            let u2 = self.mate[v.index()];
            if u2.is_none() || (self.distances[u2.unwrap().index()] == now_dist + 1 && self.dfs(u2.unwrap())) {
                // found an augmenting path
                self.mate[v.index()] = Some(u);
                return true;
            }
        }
        false
    }

    #[inline(always)]
    pub fn neighbors(&self, u: NodeId) -> std::ops::Range<usize> {
        self.start[u.index()]..self.start[u.index() + 1]
    }
}
