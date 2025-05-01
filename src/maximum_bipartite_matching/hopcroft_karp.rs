use crate::maximum_bipartite_matching::bipartite_graph::BipartiteGraph;
use std::collections::VecDeque;

#[derive(Default)]
pub struct HopcroftKarp {
    num_left_nodes: usize,
    num_right_nodes: usize,
    left_match: Box<[Option<usize>]>,

    start: Box<[usize]>,
    to: Box<[usize]>,
}

impl HopcroftKarp {
    pub fn solve(&mut self, graph: &BipartiteGraph) -> Vec<(usize, usize)> {
        self.preprocess(graph);

        let mut dist = vec![0_usize; self.num_left_nodes].into_boxed_slice();
        loop {
            dist.fill(0);
            for &u in self.left_match.iter().flatten() {
                dist[u] = usize::MAX;
            }

            // bfs
            let mut found = false;
            let mut unmatched_nodes = (0..self.num_left_nodes).filter(|&u| dist[u] == 0).collect::<VecDeque<_>>();
            while let Some(u1) = unmatched_nodes.pop_front() {
                for i in self.neighbors(u1) {
                    let v = self.to[i];
                    match self.left_match[v] {
                        Some(u2) => {
                            // u1 -> v -> u2
                            if dist[u2] == usize::MAX {
                                dist[u2] = dist[u1] + 1;
                                unmatched_nodes.push_back(u2);
                            }
                        }
                        None => {
                            found = true;
                        }
                    }
                }
            }
            if !found {
                break;
            }

            // dfs
            for u in 0..self.num_left_nodes {
                if dist[u] == 0 {
                    self.dfs(u, &mut dist);
                }
            }
        }

        self.left_match.iter().enumerate().filter_map(|(v, &u)| u.map(|u| (u, v))).collect::<Vec<_>>()
    }

    fn preprocess(&mut self, graph: &BipartiteGraph) {
        self.num_left_nodes = graph.num_left_nodes();
        self.num_right_nodes = graph.num_right_nodes();

        self.start = vec![0; self.num_left_nodes + 1].into_boxed_slice();
        self.to = (0..graph.edges.len()).map(|_| 0).collect();
        self.left_match = vec![None; self.num_right_nodes].into_boxed_slice();

        // make csr format
        let mut degree_u = vec![0; self.num_left_nodes];
        let mut degree_v = vec![0; self.num_right_nodes];
        for e in graph.edges.iter() {
            degree_u[e.u] += 1;
            degree_v[e.v] += 1;
        }

        for i in 1..=self.num_left_nodes {
            self.start[i] += self.start[i - 1] + degree_u[i - 1];
        }

        let mut count = vec![0; self.num_left_nodes].into_boxed_slice();
        for e in graph.edges.iter() {
            self.to[self.start[e.u] + count[e.u]] = e.v;
            count[e.u] += 1;
        }

        // make initial matching(greedy)
        let mut deg_u: Vec<_> = (0..self.num_left_nodes).map(|u| (degree_u[u], u)).collect();
        deg_u.sort_unstable();

        for (_, u) in deg_u {
            let mut best_v = usize::MAX;
            for i in self.neighbors(u) {
                let v = self.to[i];
                if self.left_match[v].is_none() && (best_v == usize::MAX || degree_v[v] < degree_v[best_v]) {
                    best_v = v;
                }
            }
            if best_v != usize::MAX {
                self.left_match[best_v] = Some(u);
            }
        }
    }

    fn dfs(&mut self, u: usize, dist: &mut [usize]) -> bool {
        let now_dist = std::mem::replace(&mut dist[u], usize::MAX); // use node u

        for i in self.neighbors(u) {
            let v = self.to[i];
            let u2 = self.left_match[v];
            if u2.is_none() || (dist[u2.unwrap()] == now_dist + 1 && self.dfs(u2.unwrap(), dist)) {
                // found augmenting path
                self.left_match[v] = Some(u);
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
