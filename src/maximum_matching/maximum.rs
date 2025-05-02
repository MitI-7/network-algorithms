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
    base: Box<[usize]>,
    node_type: Box<[NodeType]>,
    even_nodes: VecDeque<usize>,
    in_queue: Box<[bool]>,
    book: Box<[usize]>,
    book_mark: usize,

    // csr
    start: Box<[usize]>,
    to: Box<[usize]>,
}

impl Blossom {
    pub fn solve(&mut self, graph: &Graph) -> usize {
        let num_nodes = graph.num_nodes();

        self.mate = vec![None; num_nodes].into_boxed_slice();
        self.parent = vec![0; num_nodes].into_boxed_slice();
        self.base = (0..num_nodes).collect();
        self.node_type = vec![NodeType::Unvisited; num_nodes].into_boxed_slice();
        self.in_queue = vec![false; num_nodes].into_boxed_slice();
        self.book = vec![0; num_nodes].into_boxed_slice();

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

        let mut num_matches = 0;
        for u in 0..num_nodes {
            if self.mate[u].is_none() && self.find_augmenting_path(u) {
                num_matches += 1;
            }
        }

        num_matches
    }

    fn find_augmenting_path(&mut self, s: usize) -> bool {
        self.base.iter_mut().enumerate().for_each(|(i, x)| *x = i);
        self.node_type.fill(NodeType::Unvisited);
        self.in_queue.fill(false);
        self.even_nodes.clear();

        self.even_nodes.push_back(s);
        self.node_type[s] = NodeType::Even;
        self.in_queue[s] = true;
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
                                self.augment(v);
                                return true;
                            }
                        }
                    }
                    NodeType::Even => {
                        // find blossom
                        if self.base[u] != self.base[v] {
                            let lca = self.get_lca(u, v);
                            self.contract(u, v, lca);
                            self.contract(v, u, lca);
                        }
                    }
                    NodeType::Odd => continue,
                }
            }
        }

        false
    }

    fn augment(&mut self, mut u: usize) {
        while let Some(p) = Some(self.parent[u]) {
            let prev = self.mate[p];
            self.mate[p] = Some(u);
            self.mate[u] = Some(p);
            if let Some(x) = prev {
                u = x;
            } else {
                break;
            }
        }
    }

    fn get_lca(&mut self, mut u: usize, mut v: usize) -> usize {
        self.book_mark += 1;
        loop {
            if u != usize::MAX {
                // find lca
                if self.book[u] == self.book_mark {
                    return u;
                }
                self.book[u] = self.book_mark;

                u = if let Some(m) = self.mate[u] { self.base[self.parent[m]] } else { usize::MAX }
            }
            std::mem::swap(&mut u, &mut v);
        }
    }

    fn contract(&mut self, mut u: usize, mut v: usize, lca: usize) {
        while self.base[u] != lca {
            self.parent[u] = v;
            v = self.mate[u].unwrap();
            if self.node_type[v] == NodeType::Odd {
                self.node_type[v] = NodeType::Even;
                self.even_nodes.push_back(v);
                self.in_queue[v] = true;
            }
            self.base[u] = lca;
            self.base[v] = lca;
            u = self.parent[v];
        }
    }

    #[inline(always)]
    pub fn neighbors(&self, u: usize) -> std::ops::Range<usize> {
        self.start[u]..self.start[u + 1]
    }
}
