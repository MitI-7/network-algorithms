use crate::data_structures::SkewHeap;
use crate::edge::weight::WeightEdge;
use crate::prelude::{Directed, EdgeId, Graph};
use crate::traits::{Bounded, IntNum, Zero};
use std::marker::PhantomData;

struct UnionFind {
    par: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        let mut par = Vec::with_capacity(n);
        for i in 0..n {
            par.push(i);
        }
        UnionFind { par }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.par[x] == x {
            x
        } else {
            let root = self.find(self.par[x]);
            self.par[x] = root;
            root
        }
    }

    fn unite(&mut self, x: usize, y: usize) -> bool {
        let a = self.find(x);
        let b = self.find(y);
        if a == b {
            false
        } else {
            self.par[a] = b;
            true
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Unvisited,   // =0
    Processing,  // =1
    Done,        // =2
}

struct Edge<W> {
    id: EdgeId,
    pub from: usize,
    pub to: usize,
    pub cost: W,
}

#[derive(Default)]
pub struct Tarjan<W> {
    num_nodes: usize,
    phantom_data: PhantomData<W>,
}

impl<W> Tarjan<W>
where
    W: IntNum + Zero + Bounded,
{
    pub fn solve(&mut self, graph: &Graph<Directed, (), WeightEdge<W>>, root: usize) -> Option<(W, Vec<EdgeId>)> {
        self.num_nodes = graph.num_nodes();
        let mut edges = Vec::with_capacity(graph.num_edges());
        for (i, edge) in graph.edges.iter().enumerate() {
            edges.push(Edge { id: EdgeId(i), from: edge.u.index(), to: edge.v.index(), cost: edge.data.weight });
        }

        let s =self.msa(&edges, root);
        if s.is_none() {
            return None;
        }
        Some((s.unwrap(), Vec::new()))
    }

    fn msa(&self, edges: &[Edge<W>], r: usize) -> Option<W> {
        let mut uf = UnionFind::new(self.num_nodes);
        let mut come: Vec<Option<Box<SkewHeap<W>>>> = Vec::with_capacity(self.num_nodes);
        come.resize(self.num_nodes, None);
        let mut used = vec![State::Unvisited; self.num_nodes];
        let mut from = vec![0; self.num_nodes];
        let mut from_cost = vec![W::zero(); self.num_nodes];
        used[r] = State::Done;

        // build initial heaps
        for (i, e) in edges.iter().enumerate() {
            let node = SkewHeap::new(e.cost, i);
            come[e.to] = SkewHeap::meld(come[e.to].take(), Some(node));
        }

        let mut total_cost = W::zero();
        for start in 0..self.num_nodes {
            if used[start] != State::Unvisited {
                continue;
            }
            let mut processing_nodes = Vec::new();
            let mut now = start;
            while used[now] != State::Done {
                used[now] = State::Processing;
                processing_nodes.push(now);

                if come[now].is_none() {
                    return None;
                }

                let mut heap = come[now].take().unwrap();
                heap.push_lazy();
                from[now] = uf.find(edges[heap.id].from);
                from_cost[now] = heap.v;
                come[now] = heap.pop();

                // ignore self loops
                if from[now] == now {
                    continue;
                }
                total_cost += from_cost[now];

                // cycle detected
                if used[from[now]] == State::Processing {
                    let mut p = now;
                    loop {
                        if let Some(ref mut h) = come[p] {
                            h.add -= from_cost[p];
                        }
                        if p != now {
                            uf.unite(p, now);
                            come[now] = SkewHeap::meld(come[now].take(), come[p].take());
                        }
                        p = uf.find(from[p]);
                        if p == now {
                            break;
                        }
                    }
                } else {
                    now = from[now];
                }
            }

            processing_nodes.iter().for_each(|&u| used[u] = State::Done);
        }

        Some(total_cost)
    }
}
