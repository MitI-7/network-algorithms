use crate::data_structures::{BitVector, SimpleQueue};
use crate::graph::bipartite_graph::BipartiteGraph;

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
/// \note In several cases the push-relabel based algorithms have
/// better runtime performance than the augmenting path based ones.
#[derive(Default)]
pub struct HopcroftKarp {
    num_left_nodes: usize,
    num_right_nodes: usize,
    mate: Box<[Option<usize>]>, // mate[right_node] = Some(left_node)
    distances: Box<[usize]>,

    // csr format(left -> right)
    start: Box<[usize]>,
    to: Box<[usize]>,

    warm_start: WarmStart,
    queue: SimpleQueue<usize>,
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

    pub fn solve(&mut self, graph: &BipartiteGraph) -> Vec<usize> {
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
                    self.mate[edge.v] = Some(edge.u);
                }
            }
        }

        loop {
            if !self.update_distances() {
                break;
            }

            for u in 0..self.num_left_nodes {
                if self.distances[u] == 0 {
                    self.dfs(u);
                }
            }
        }

        let mut matching = Vec::new();
        let (mut used_u, mut used_v) = (vec![false; self.num_left_nodes].into_boxed_slice(), vec![false; self.num_right_nodes].into_boxed_slice());
        for (edge_id, edge) in graph.edges.iter().enumerate() {
            // for multiple edge
            if used_u[edge.u] || used_v[edge.v] {
                continue;
            }

            if self.mate[edge.v] == Some(edge.u) {
                matching.push(edge_id);
                used_u[edge.u] = true;
                used_v[edge.v] = true;
            }
        }

        matching
    }

    fn preprocess(&mut self, graph: &BipartiteGraph) {
        self.num_left_nodes = graph.num_left_nodes();
        self.num_right_nodes = graph.num_right_nodes();

        self.start = vec![0; self.num_left_nodes + 1].into_boxed_slice();
        self.to = (0..graph.edges.len()).map(|_| 0).collect();
        self.mate = vec![None; self.num_right_nodes].into_boxed_slice();
        self.distances = vec![0_usize; self.num_left_nodes].into_boxed_slice();
        self.queue = SimpleQueue::with_capacity(self.num_left_nodes);

        // make csr format
        for u in 1..=self.num_left_nodes {
            self.start[u] += self.start[u - 1] + graph.degree_left[u - 1];
        }

        let mut count = vec![0; self.num_left_nodes].into_boxed_slice();
        for edge in graph.edges.iter() {
            self.to[self.start[edge.u] + count[edge.u]] = edge.v;
            count[edge.u] += 1;
        }
    }

    // make initial matching(greedy)
    fn initial_solution_greedy(&mut self, degree_u: &[usize], degree_v: &[usize]) {
        let mut deg_u: Vec<_> = (0..self.num_left_nodes).map(|u| (degree_u[u], u)).collect();
        deg_u.sort_unstable();

        for (_, u) in deg_u {
            let mut best_v = None;
            for i in self.neighbors(u) {
                let v = self.to[i];
                if self.mate[v].is_none() && (best_v.is_none() || degree_v[v] < degree_v[best_v.unwrap()]) {
                    best_v = Some(v);
                }
            }

            if let Some(best_v) = best_v {
                self.mate[best_v] = Some(u);
            }
        }
    }

    // O(m)
    fn initial_solution_karp_sipser(&mut self, graph: &BipartiteGraph) {
        // make csr format(right -> left)
        let mut start_r = vec![0; self.num_right_nodes + 1].into_boxed_slice();
        let mut to_r: Box<[usize]> = (0..graph.edges.len()).map(|_| 0).collect();

        for v in 1..=self.num_right_nodes {
            start_r[v] += start_r[v - 1] + graph.degree_right[v - 1];
        }

        let mut count = vec![0; self.num_right_nodes].into_boxed_slice();
        for edge in graph.edges.iter() {
            to_r[start_r[edge.v] + count[edge.v]] = edge.u;
            count[edge.v] += 1;
        }

        let mut degree_left = graph.degree_left.clone();
        let mut degree_right = graph.degree_right.clone();
        let mut used_left = BitVector::new(self.num_left_nodes);
        let mut used_right = BitVector::new(self.num_right_nodes);

        let mut que = SimpleQueue::with_capacity(self.num_left_nodes + self.num_right_nodes);
        let iter_left = degree_left.iter().enumerate().filter_map(|(u, &d)| (d == 1).then_some(u));
        let iter_right = degree_right.iter().enumerate().filter_map(|(v, &d)| (d == 1).then_some(self.num_left_nodes + v));
        que.extend(iter_left.chain(iter_right));

        // phase-1
        while let Some(node_id) = que.pop() {
            if node_id < self.num_left_nodes {
                let u = node_id;
                if used_left.get(u) || degree_left[u] != 1 {
                    continue;
                }

                let v = match self.neighbors(u).find(|&i| !used_right.get(self.to[i])) {
                    Some(i) => self.to[i],
                    None => continue,
                };

                self.mate[v] = Some(u);
                used_left.set(u, true);
                used_right.set(v, true);

                for i in start_r[v]..start_r[v + 1] {
                    let u2 = to_r[i];
                    if !used_left.get(u2) {
                        degree_left[u2] -= 1;
                        if degree_left[u2] == 1 {
                            que.push(u2);
                        }
                    }
                }
            } else {
                let v = node_id - self.num_left_nodes;
                if used_right.get(v) || degree_right[v] != 1 {
                    continue;
                }

                let u = match (start_r[v]..start_r[v + 1]).find(|&i| !used_left.get(to_r[i])) {
                    Some(i) => to_r[i],
                    None => continue,
                };

                self.mate[v] = Some(u);
                used_left.set(u, true);
                used_right.set(v, true);

                for i in self.neighbors(u) {
                    let v2 = self.to[i];
                    if !used_right.get(v2) {
                        degree_right[v2] -= 1;
                        if degree_right[v2] == 1 {
                            que.push(self.num_left_nodes + v2);
                        }
                    }
                }
            }
        }

        // phase-2 greedy
        let mut nodes: Vec<_> = (0..self.num_left_nodes).filter(|&u| !used_left.get(u)).collect();
        nodes.sort_unstable_by_key(|&u| degree_left[u]);

        for u in nodes {
            assert!(!used_left.get(u));
            let mut best_v = None;
            for i in self.neighbors(u) {
                let v = self.to[i];
                if self.mate[v].is_none() && (best_v.is_none() || degree_right[v] < degree_right[best_v.unwrap()]) {
                    best_v = Some(v);
                }
            }

            if let Some(best_v) = best_v {
                self.mate[best_v] = Some(u);
            }
        }
    }

    fn update_distances(&mut self) -> bool {
        // initialize
        self.distances.fill(0);
        for &u in self.mate.iter().flatten() {
            self.distances[u] = usize::MAX;
        }

        self.queue.reset();
        for (u, &d) in self.distances.iter().enumerate() {
            if d == 0 {
                self.queue.push(u);
            }
        }

        let mut found = false;
        while let Some(u1) = self.queue.pop() {
            for i in self.neighbors(u1) {
                let v = self.to[i];
                match self.mate[v] {
                    Some(u2) => {
                        // u1 -> v -> u2
                        if self.distances[u2] == usize::MAX {
                            self.distances[u2] = self.distances[u1] + 1;
                            self.queue.push(u2);
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

    fn dfs(&mut self, u: usize) -> bool {
        let now_dist = std::mem::replace(&mut self.distances[u], usize::MAX); // use node u

        for i in self.neighbors(u) {
            let v = self.to[i];
            let u2 = self.mate[v];
            if u2.is_none() || (self.distances[u2.unwrap()] == now_dist + 1 && self.dfs(u2.unwrap())) {
                // found an augmenting path
                self.mate[v] = Some(u);
                return true;
            }
        }
        false
    }

    #[inline(always)]
    pub fn neighbors(&self, u: usize) -> std::ops::Range<usize> {
        self.start[u]..self.start[u + 1]
    }
}
