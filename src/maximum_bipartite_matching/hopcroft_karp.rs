use crate::data_structures::simple_queue::SimpleQueue;
use crate::maximum_bipartite_matching::bipartite_graph::BipartiteGraph;

#[derive(Default)]
pub struct HopcroftKarp {
    num_left_nodes: usize,
    num_right_nodes: usize,
    mate: Box<[Option<usize>]>, // mate[right_node] = Some(left_node)
    distances: Box<[usize]>,

    start: Box<[usize]>,
    to: Box<[usize]>,

    start_with_initial_matching: bool,
    initial_matching: Vec<usize>,
    queue: SimpleQueue<usize>,
}

impl HopcroftKarp {
    pub fn new_with_matching(matching: &[usize]) -> Self {
        Self { initial_matching: matching.to_vec(), ..Self::default() }
    }

    pub fn new_with_greedy() -> Self {
        Self { start_with_initial_matching: true, ..Self::default() }
    }

    pub fn solve(&mut self, graph: &BipartiteGraph) -> Vec<usize> {
        self.preprocess(graph);

        if self.start_with_initial_matching {
            self.initial_solution(&graph.degree_left, &graph.degree_right);
        }

        if !self.initial_matching.is_empty() {
            for &edge_id in self.initial_matching.iter() {
                let edge = &graph.edges[edge_id];
                self.mate[edge.v] = Some(edge.u);
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
    fn initial_solution(&mut self, degree_u: &[usize], degree_v: &[usize]) {
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
