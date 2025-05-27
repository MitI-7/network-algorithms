use crate::data_structures::UnionFind;
use crate::edge::weight::WeightEdge;
use crate::prelude::{Directed, EdgeId, Graph};
use crate::traits::{Bounded, IntNum, Zero};
use std::marker::PhantomData;

/// ---------- Rollback-DSU ----------
#[derive(Clone)]
struct RollbackDsu {
    p: Vec<i32>,
    hist: Vec<(usize, i32)>,
}

impl RollbackDsu {
    fn new(n: usize) -> Self {
        Self { p: vec![-1; n], hist: vec![] }
    }

    fn find(&self, mut v: usize) -> usize {
        while self.p[v] >= 0 {
            v = self.p[v] as usize
        }
        v
    }

    fn same(&self, a: usize, b: usize) -> bool {
        self.find(a) == self.find(b)
    }

    fn time(&self) -> usize {
        self.hist.len()
    }

    fn join(&mut self, a: usize, b: usize) -> bool {
        let (mut x, mut y) = (self.find(a), self.find(b));
        if x == y {
            return false;
        }
        if self.p[x] > self.p[y] {
            std::mem::swap(&mut x, &mut y);
        }
        self.hist.push((y, self.p[y]));
        self.p[x] += self.p[y];
        self.p[y] = x as i32;
        true
    }

    fn rollback(&mut self, t: usize) {
        while self.hist.len() > t {
            let (v, old) = self.hist.pop().unwrap();
            let p = self.p[v] as usize;
            self.p[p] -= old;
            self.p[v] = old;
        }
    }
}

#[derive(Clone)]
struct Node<W> {
    left: Option<usize>,
    right: Option<usize>,
    from: usize,
    to: usize,
    w: W,
    lz: W,
    orig: usize, // ← 追加: 入力辺のインデックス
}

#[inline]
fn apply<W: IntNum + Zero + Bounded + std::ops::Neg<Output = W> + Default>(ns: &mut [Node<W>], i: usize, d: W) {
    ns[i].w -= d;
    ns[i].lz += d;
}

fn push<W: IntNum + Zero + Bounded + std::ops::Neg<Output = W> + Default>(ns: &mut [Node<W>], i: usize) {
    let lz = ns[i].lz;
    if lz != W::default() {
        let (l, r) = (ns[i].left, ns[i].right);
        ns[i].lz = W::default();
        if let Some(c) = l {
            apply(ns, c, lz);
        }
        if let Some(c) = r {
            apply(ns, c, lz);
        }
    }
}
fn merge<W: IntNum + Zero + Bounded + std::ops::Neg<Output = W> + Default>(ns: &mut Vec<Node<W>>, a: Option<usize>, b: Option<usize>) -> Option<usize> {
    match (a, b) {
        (None, None) => None,
        (Some(x), None) | (None, Some(x)) => Some(x),
        (Some(mut u), Some(mut v)) => {
            if ns[v].w < ns[u].w {
                std::mem::swap(&mut u, &mut v);
            }
            push(ns, u);
            let right = ns[u].right;
            ns[u].right = merge(ns, right, Some(v));
            /* swap children */
            let tmp = ns[u].left;
            ns[u].left = ns[u].right;
            ns[u].right = tmp;
            Some(u)
        }
    }
}
fn pop<W: IntNum + Zero + Bounded + std::ops::Neg<Output = W> + Default>(ns: &mut Vec<Node<W>>, root: &mut [Option<usize>], v: usize) {
    if let Some(r) = root[v] {
        push(ns, r);
        root[v] = merge(ns, ns[r].left, ns[r].right);
    }
}

#[derive(Default)]
pub struct Tarjan<W> {
    num_nodes: usize,
    phantom_data: PhantomData<W>,
}

impl<W> Tarjan<W>
where
    W: IntNum + Zero + Bounded + std::ops::Neg<Output = W> + Default,
{
    pub fn solve(&mut self, graph: &Graph<Directed, (), WeightEdge<W>>, root: usize) -> Option<(W, Vec<usize>)> {
        self.num_nodes = graph.num_nodes();
        let mut edges = Vec::with_capacity(graph.num_edges());
        for (i, edge) in graph.edges.iter().enumerate() {
            edges.push((edge.u.index(), edge.v.index(), edge.data.weight ));
        }

        self.min_arborescence(root, &mut edges)
    }

    fn min_arborescence(&self, root: usize, edges: &[(usize, usize, W)]) -> Option<(W, Vec<usize>)> {
        let mut ns = Vec::<Node<W>>::with_capacity(edges.len());
        let mut heap = vec![None; self.num_nodes];
        for (idx, &(u, v, w)) in edges.iter().enumerate() {
            ns.push(Node { left: None, right: None, from: u, to: v, w, lz: W::zero(), orig: idx });
            let id = ns.len() - 1;
            heap[v] = merge(&mut ns, heap[v], Some(id));
        }

        let mut cost = W::zero();
        let mut edge = vec![None::<usize>; self.num_nodes];
        let mut cycles = Vec::<(usize, usize)>::new();
        let mut dsu_c = UnionFind::new(self.num_nodes);
        let mut dsu_r = RollbackDsu::new(self.num_nodes);

        for u in 0..self.num_nodes {
            if u == root {
                continue;
            }
            let mut now = u;
            loop {
                let e = heap[now]?; // 到達不能
                edge[now] = Some(e);
                let w = ns[e].w;
                cost += w;
                apply(&mut ns, e, w);

                let fr = dsu_r.find(ns[e].from);
                if dsu_c.unite(now, fr) {
                    break;
                }

                /* contract new cycle */
                let t = dsu_r.time();
                let mut nxt = fr;
                while dsu_r.join(now, nxt) {
                    let rep = dsu_r.find(now);
                    heap[rep] = merge(&mut ns, heap[now], heap[nxt]);
                    now = rep;
                    nxt = dsu_r.find(ns[edge[nxt].unwrap()].from);
                }
                cycles.push((edge[now].unwrap(), t));

                /* remove self-loops */
                loop {
                    let idx = match heap[now] {
                        Some(x) => x,
                        None => break,
                    };
                    if dsu_r.same(ns[idx].from, now) {
                        pop(&mut ns, &mut heap, now);
                    } else {
                        break;
                    }
                }
            }
        }

        /* expand cycles */
        for &(e, t) in cycles.iter().rev() {
            let vr = dsu_r.find(ns[e].to);
            dsu_r.rollback(t);
            let vin = dsu_r.find(ns[edge[vr].unwrap()].to);
            let old = std::mem::replace(&mut edge[vr], Some(e));
            edge[vin] = old;
        }

        let mut idx_vec = Vec::with_capacity(self.num_nodes - 1);
        for u in 0..self.num_nodes {
            if u != root {
                idx_vec.push(ns[edge[u].unwrap()].orig);
            }
        }
        Some((cost, idx_vec))
    }
}
