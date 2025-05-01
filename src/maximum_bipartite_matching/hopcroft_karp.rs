use std::collections::VecDeque;

pub struct HopcroftKarp {
    num_left_nodes: usize,
    num_right_nodes: usize,
    edges: Vec<(usize, usize)>,

    start: Vec<usize>,
    inside_edge_list: Vec<usize>,
}

impl HopcroftKarp {
    pub fn new(num_left_nodes: usize, num_right_nodes: usize) -> Self {
        Self { num_left_nodes, num_right_nodes, edges: Vec::new(), start: Vec::new(), inside_edge_list: Vec::new() }
    }

    pub fn add_edge(&mut self, u: usize, v: usize) {
        self.edges.push((u, v));
    }

    pub fn solve(&mut self) -> Vec<(usize, usize)> {
        self.start.resize(self.num_left_nodes + 1, 0);
        self.inside_edge_list = (0..self.edges.len()).map(|_| 0).collect();

        let mut degree_u = vec![0; self.num_left_nodes];
        let mut degree_v = vec![0; self.num_right_nodes];
        for &(u, v) in self.edges.iter() {
            degree_u[u] += 1;
            degree_v[v] += 1;
        }

        for i in 1..=self.num_left_nodes {
            self.start[i] += self.start[i - 1] + degree_u[i - 1];
        }

        let mut counter = vec![0; self.num_left_nodes];
        for &(u, v) in self.edges.iter() {
            self.inside_edge_list[self.start[u] + counter[u]] = v;
            counter[u] += 1;
        }

        let mut left_match = vec![usize::MAX; self.num_right_nodes];

        let mut deg_u: Vec<_> = (0..self.num_left_nodes).map(|u| (degree_u[u], u)).collect();
        deg_u.sort_unstable();

        let mut num_match = 0;
        for (_, u) in deg_u {
            let mut best_v = usize::MAX;
            for &v in self.inside_edge_list[self.start[u]..self.start[u + 1]].iter() {
                if left_match[v] == usize::MAX && (best_v == usize::MAX || degree_v[v] < degree_v[best_v]) {
                    best_v = v;
                }
            }
            if best_v != usize::MAX {
                left_match[best_v] = u;
                num_match += 1;
            }
        }

        let mut dist = vec![0_usize; self.num_left_nodes];
        loop {
            dist.fill(0);
            for &u in left_match.iter() {
                if u != usize::MAX {
                    dist[u] = usize::MAX;
                }
            }

            // bfs
            let mut found = false;
            let mut queue = (0..self.num_left_nodes).filter(|&u| dist[u] == 0).collect::<VecDeque<_>>();
            while let Some(u1) = queue.pop_front() {
                for &v in self.inside_edge_list[self.start[u1]..self.start[u1 + 1]].iter() {
                    let u2 = left_match[v];
                    if u2 == usize::MAX {
                        found = true;
                        continue;
                    }

                    // u1 -> v -> u2
                    if dist[u2] == usize::MAX {
                        dist[u2] = dist[u1] + 1;
                        queue.push_back(u2);
                    }
                }
            }
            if !found {
                break;
            }

            // dfs
            for u in 0..self.num_left_nodes {
                if dist[u] == 0 && self.dfs(u, &mut left_match, &mut dist) {
                    num_match += 1;
                }
            }
        }

        left_match
            .iter()
            .enumerate()
            .filter_map(|(v, &u)| if u == usize::MAX { None } else { Some((u, v)) })
            .collect::<Vec<_>>()
    }

    fn dfs(&self, u: usize, left_match: &mut [usize], dist: &mut [usize]) -> bool {
        let now_dist = std::mem::replace(&mut dist[u], usize::MAX); // use node u

        for &v in self.inside_edge_list[self.start[u]..self.start[u + 1]].iter() {
            let u2 = left_match[v];
            if u2 == usize::MAX || (dist[u2] == now_dist + 1 && self.dfs(u2, left_match, dist)) {
                // found augmenting path
                left_match[v] = u;
                return true;
            }
        }

        false
    }
}
