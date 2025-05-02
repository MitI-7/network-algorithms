use crate::maximum_matching::graph::Graph;
use std::collections::VecDeque;

#[derive(Copy, Clone, Debug, PartialEq)]
enum NodeType {
    Unvisited,
    Even,
    Odd,
}

#[derive(Default)]
pub struct Blossom {
    mate: Box<[Option<usize>]>,
    parent: Box<[usize]>,
    label: Box<[usize]>,
    node_type: Box<[NodeType]>,
    even_nodes: VecDeque<usize>,
    in_queue: Box<[bool]>,
    time_stamp: Box<[usize]>,
    time: usize,

    // csr
    start: Box<[usize]>,
    to: Box<[usize]>,
}

impl Blossom {
    pub fn solve(&mut self, graph: &Graph) -> Vec<usize> {
        self.preprocess(graph);

        for root in 0..graph.num_nodes() {
            if self.mate[root].is_some() {
                continue;
            }
            if let Some(u) = self.find_augmenting_path(root) {
                self.augmentation(root, u);
            }
        }

        let mut matching = Vec::new();
        for (i, e) in graph.edges.iter().enumerate() {
            if self.mate[e.u] == Some(e.v) {
                matching.push(i);
            }
        }
        matching
    }

    fn preprocess(&mut self, graph: &Graph) {
        let num_nodes = graph.num_nodes();

        self.mate = vec![None; num_nodes].into_boxed_slice();
        self.parent = vec![usize::MAX; num_nodes].into_boxed_slice();
        self.label = (0..num_nodes).collect();
        self.node_type = vec![NodeType::Unvisited; num_nodes].into_boxed_slice();
        self.in_queue = vec![false; num_nodes].into_boxed_slice();
        self.time_stamp = vec![0; num_nodes].into_boxed_slice();

        self.start = vec![0; num_nodes + 1].into_boxed_slice();
        self.to = (0..graph.num_edges() * 2).map(|_| 0).collect();

        // make csr format
        let mut degree = vec![0; num_nodes];
        for e in graph.edges.iter() {
            degree[e.u] += 1;
            degree[e.v] += 1;
        }

        for i in 1..=num_nodes {
            self.start[i] += self.start[i - 1] + degree[i - 1];
        }

        let mut count = vec![0; num_nodes].into_boxed_slice();
        for e in graph.edges.iter() {
            self.to[self.start[e.u] + count[e.u]] = e.v;
            count[e.u] += 1;

            self.to[self.start[e.v] + count[e.v]] = e.u;
            count[e.v] += 1;
        }
    }

    fn find_augmenting_path(&mut self, root: usize) -> Option<usize> {
        self.label.iter_mut().enumerate().for_each(|(i, x)| *x = i);
        self.node_type.fill(NodeType::Unvisited);
        self.in_queue.fill(false);
        self.even_nodes.clear();

        self.even_nodes.push_back(root);
        self.node_type[root] = NodeType::Even;
        self.in_queue[root] = true;
        while let Some(u) = self.even_nodes.pop_front() {
            assert_eq!(self.node_type[u], NodeType::Even);

            for i in self.neighbors(u) {
                let v = self.to[i];
                match self.node_type[v] {
                    NodeType::Unvisited => {
                        self.parent[v] = u;
                        self.node_type[v] = NodeType::Odd;

                        match self.mate[v] {
                            Some(w) => {
                                self.node_type[w] = NodeType::Even;
                                self.parent[w] = v; // u -> v -> w
                                if !self.in_queue[w] {
                                    self.even_nodes.push_back(w);
                                    self.in_queue[w] = true;
                                }
                            }
                            None => {
                                // find an augmenting path
                                return Some(v);
                            }
                        }
                    }
                    NodeType::Even => {
                        // find blossom
                        if self.label[u] != self.label[v] {
                            let base = self.get_base(u, v);
                            self.contract(u, v, base);
                            self.contract(v, u, base);
                        }
                    }
                    NodeType::Odd => continue,
                }
            }
        }

        None
    }

    fn augmentation(&mut self, root: usize, end: usize) {
        let mut now = end;
        while now != root {
            let p = self.parent[now];
            let next = self.mate[p];
            (self.mate[p], self.mate[now]) = (Some(now), Some(p));

            if p == root {
                break;
            }
            now = next.unwrap();
        }
    }

    fn get_base(&mut self, mut u: usize, mut v: usize) -> usize {
        self.time += 1;
        loop {
            if u != usize::MAX {
                // find base
                if self.time_stamp[u] == self.time {
                    return u;
                }
                assert_ne!(self.time_stamp[u], self.time);
                self.time_stamp[u] = self.time;

                u = if let Some(x) = self.mate[u] { self.label[self.parent[x]] } else { usize::MAX }
            }
            std::mem::swap(&mut u, &mut v);
        }
    }

    fn contract(&mut self, entry: usize, mut bridge_even: usize, base: usize) {
        let mut now = entry;
        while self.label[now] != base {
            self.parent[now] = bridge_even;

            bridge_even = self.mate[now].unwrap();
            if self.node_type[bridge_even] == NodeType::Odd {
                self.node_type[bridge_even] = NodeType::Even;
                assert!(!self.in_queue[bridge_even]);
                self.even_nodes.push_back(bridge_even);
                self.in_queue[bridge_even] = true;
            }
            self.label[now] = base;
            self.label[bridge_even] = base;

            now = self.parent[bridge_even];
        }
    }

    #[inline(always)]
    pub fn neighbors(&self, u: usize) -> std::ops::Range<usize> {
        self.start[u]..self.start[u + 1]
    }
}
