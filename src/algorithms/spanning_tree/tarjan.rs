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

#[derive(Clone)]
struct Heap<W> {
    l: Option<Box<Heap<W>>>,
    r: Option<Box<Heap<W>>>,
    add: W,
    v: W,
    id: usize,
}

impl<W> Heap<W>
where
    W: IntNum + Zero,
{
    fn new(v: W, id: usize) -> Box<Self> {
        Box::new(Heap { l: None, r: None, add: W::zero(), v, id })
    }
}

fn lazy<W: IntNum + Zero>(a: &mut Box<Heap<W>>) {
    if let Some(ref mut left) = a.l {
        left.add += a.add;
    }
    if let Some(ref mut right) = a.r {
        right.add += a.add;
    }
    a.v += a.add;
    a.add = W::zero();
}

fn meld<W: IntNum + Zero>(a: Option<Box<Heap<W>>>, b: Option<Box<Heap<W>>>) -> Option<Box<Heap<W>>> {
    match (a, b) {
        (None, x) => x,
        (x, None) => x,
        (Some(mut xh), Some(mut yh)) => {
            if xh.v + xh.add > yh.v + yh.add {
                return meld(Some(yh), Some(xh));
            }
            lazy(&mut xh);
            xh.r = meld(xh.r.take(), Some(yh));
            std::mem::swap(&mut xh.l, &mut xh.r);
            Some(xh)
        }
    }
}

fn pop<W: IntNum + Zero>(mut a: Option<Box<Heap<W>>>) -> Option<Box<Heap<W>>> {
    if let Some(mut node) = a {
        lazy(&mut node);
        meld(node.l.take(), node.r.take())
    } else {
        None
    }
}

struct Edge<W> {
    id: EdgeId,
    pub from: usize,
    pub to: usize,
    pub cost: W,
}

#[derive(Default)]
pub struct Tarjan<W> {
    phantom_data: PhantomData<W>,
}

impl<W> Tarjan<W>
where
    W: IntNum + Zero + Bounded,
{
    pub fn solve(&mut self, graph: &Graph<Directed, (), WeightEdge<W>>, root: usize) -> Option<(W, Vec<EdgeId>)> {
        let mut edges = Vec::with_capacity(graph.num_edges());
        for (i, edge) in graph.edges.iter().enumerate() {
            edges.push(Edge { id: EdgeId(i), from: edge.u.index(), to: edge.v.index(), cost: edge.data.weight });
        }

        Some((self.msa(graph.num_nodes(), &edges, root), Vec::new()))
    }

    fn msa(&self, n: usize, edges: &[Edge<W>], r: usize) -> W {
        let mut uf = UnionFind::new(n);
        let mut come: Vec<Option<Box<Heap<W>>>> = Vec::with_capacity(n);
        come.resize(n, None);
        let mut used = vec![0; n];
        let mut from = vec![0; n];
        let mut from_cost = vec![W::zero(); n];
        used[r] = 2;

        // build initial heaps
        for (i, e) in edges.iter().enumerate() {
            let mut node = Heap::new(e.cost, i);
            come[e.to] = meld(come[e.to].take(), Some(node));
        }

        let mut res = W::zero();
        for start in 0..n {
            if used[start] != 0 {
                continue;
            }
            let mut processing = Vec::new();
            let mut cur = start;
            while used[cur] != 2 {
                // mark as processing
                used[cur] = 1;
                processing.push(cur);
                // no incoming edges
                if come[cur].is_none() {
                    return W::max_value();
                }
                // take smallest incoming edge
                let mut heap = come[cur].take().unwrap();
                lazy(&mut heap);
                let eidx = heap.id;
                from[cur] = uf.find(edges[eidx].from);
                from_cost[cur] = heap.v;
                come[cur] = pop(Some(heap));

                // ignore self loops
                if from[cur] == cur {
                    continue;
                }
                res += from_cost[cur];

                // cycle detected
                if used[from[cur]] == 1 {
                    let mut p = cur;
                    loop {
                        if let Some(ref mut h) = come[p] {
                            h.add -= from_cost[p];
                        }
                        if p != cur {
                            uf.unite(p, cur);
                            come[cur] = meld(come[cur].take(), come[p].take());
                        }
                        p = uf.find(from[p]);
                        if p == cur {
                            break;
                        }
                    }
                } else {
                    cur = from[cur];
                }
            }
            // mark processed nodes as done
            for &u in &processing {
                used[u] = 2;
            }
        }
        res
    }
}
