use crate::graph::{direction::Undirected, graph::Graph, ids::EdgeId};
use std::collections::VecDeque;

#[derive(Copy, Clone, Debug, PartialEq)]
enum NodeType {
    Unvisited,
    Even, // the number of edges in the unique path from the root node to node u in the alternating tree is even
    Odd,  // the number of edges in the unique path from the root node to node u in the alternating tree is odd
}

#[derive(Default)]
pub struct Blossom {
    mate: Box<[Option<usize>]>,
    parent: Box<[usize]>,
    component: Box<[usize]>,
    labels: Box<[NodeType]>,
    even_nodes: VecDeque<usize>,
    in_queue: Box<[bool]>,
    time_stamp: Box<[usize]>,
    time: usize,

    // csr
    start: Box<[usize]>,
    to: Box<[usize]>,
}

impl Blossom {
    pub fn solve(&mut self, graph: &Graph<Undirected, (), ()>) -> Vec<EdgeId> {
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
        for (i, e) in graph.edges().enumerate() {
            if self.mate[e.u.index()] == Some(e.v.index()) {
                matching.push(EdgeId(i));
            }
        }
        matching
    }

    fn preprocess(&mut self, graph: &Graph<Undirected, (), ()>) {
        let num_nodes = graph.num_nodes();

        self.mate = vec![None; num_nodes].into_boxed_slice();
        self.parent = vec![usize::MAX; num_nodes].into_boxed_slice();
        self.component = (0..num_nodes).collect();
        self.labels = vec![NodeType::Unvisited; num_nodes].into_boxed_slice();
        self.in_queue = vec![false; num_nodes].into_boxed_slice();
        self.time_stamp = vec![0; num_nodes].into_boxed_slice();

        self.start = vec![0; num_nodes + 1].into_boxed_slice();
        self.to = (0..graph.num_edges() * 2).map(|_| 0).collect();

        // make csr format
        let mut degree = vec![0; num_nodes];
        for e in graph.edges() {
            degree[e.u.index()] += 1;
            degree[e.v.index()] += 1;
        }

        for i in 1..=num_nodes {
            self.start[i] += self.start[i - 1] + degree[i - 1];
        }

        let mut count = vec![0; num_nodes].into_boxed_slice();
        for e in graph.edges() {
            self.to[self.start[e.u.index()] + count[e.u.index()]] = e.v.index();
            count[e.u.index()] += 1;

            self.to[self.start[e.v.index()] + count[e.v.index()]] = e.u.index();
            count[e.v.index()] += 1;
        }
    }

    fn find_augmenting_path(&mut self, root: usize) -> Option<usize> {
        self.component.iter_mut().enumerate().for_each(|(i, x)| *x = i);
        self.labels.fill(NodeType::Unvisited);
        self.in_queue.fill(false);
        self.even_nodes.clear();

        self.even_nodes.push_back(root);
        self.labels[root] = NodeType::Even;
        self.in_queue[root] = true;
        while let Some(u) = self.even_nodes.pop_front() {
            assert_eq!(self.labels[u], NodeType::Even);

            for i in self.neighbors(u) {
                let v = self.to[i];
                match self.labels[v] {
                    NodeType::Unvisited => {
                        self.parent[v] = u;
                        self.labels[v] = NodeType::Odd;

                        match self.mate[v] {
                            Some(w) => {
                                self.labels[w] = NodeType::Even;
                                self.parent[w] = v; // u(even) -> v(odd) -> w(even)
                                if !self.in_queue[w] {
                                    // TODO
                                    self.even_nodes.push_back(w);
                                    self.in_queue[w] = true;
                                }
                            }
                            None => {
                                // whenever an unmatched node has an odd label, the path joining the root node to this node is an augmenting path
                                return Some(v);
                            }
                        }
                    }
                    NodeType::Even => {
                        // find blossom
                        if self.component[u] != self.component[v] {
                            let base = self.find_base_of_blossom(u, v);
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

    fn find_base_of_blossom(&mut self, mut u: usize, mut v: usize) -> usize {
        self.time += 1;
        loop {
            if u != usize::MAX {
                // find base
                if self.time_stamp[u] == self.time {
                    assert_eq!(self.labels[u], NodeType::Even);
                    return u;
                }
                assert_ne!(self.time_stamp[u], self.time);
                self.time_stamp[u] = self.time;

                u = if let Some(x) = self.mate[u] {
                    self.component[self.parent[x]]
                } else {
                    usize::MAX
                }
            }
            std::mem::swap(&mut u, &mut v);
        }
    }

    fn contract(&mut self, entry: usize, mut peer: usize, base: usize) {
        assert_eq!(self.labels[entry], self.labels[peer]);

        let mut now = entry;
        while self.component[now] != base {
            self.parent[now] = peer;
            peer = self.mate[now].unwrap();

            self.component[now] = base;
            self.component[peer] = base;

            if self.labels[peer] == NodeType::Odd {
                self.labels[peer] = NodeType::Even;
                assert!(!self.in_queue[peer]);
                self.even_nodes.push_back(peer);
                self.in_queue[peer] = true;
            }

            now = self.parent[peer];
        }
    }

    #[inline(always)]
    pub fn neighbors(&self, u: usize) -> std::ops::Range<usize> {
        self.start[u]..self.start[u + 1]
    }
}
