const INF: i64 = 1_000_000_010;
type Index = usize;

struct UnionFind {
    par: Vec<Index>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        let mut par = Vec::with_capacity(n);
        for i in 0..n {
            par.push(i);
        }
        UnionFind { par }
    }

    fn find(&mut self, x: Index) -> Index {
        if self.par[x] == x { x } else {
            let root = self.find(self.par[x]);
            self.par[x] = root;
            root
        }
    }

    fn unite(&mut self, x: Index, y: Index) -> bool {
        let a = self.find(x);
        let b = self.find(y);
        if a == b { false } else { self.par[a] = b; true }
    }
}

#[derive(Clone)]
struct Heap {
    l: Option<Box<Heap>>, r: Option<Box<Heap>>,
    add: i64, v: i64, id: usize,
}

impl Heap {
    fn new(v: i64, id: usize) -> Box<Self> {
        Box::new(Heap { l: None, r: None, add: 0, v, id })
    }
}

fn lazy(a: &mut Box<Heap>) {
    if let Some(ref mut left) = a.l { left.add += a.add; }
    if let Some(ref mut right) = a.r { right.add += a.add; }
    a.v += a.add;
    a.add = 0;
}

fn meld(a: Option<Box<Heap>>, b: Option<Box<Heap>>) -> Option<Box<Heap>> {
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

fn pop(mut a: Option<Box<Heap>>) -> Option<Box<Heap>> {
    if let Some(mut node) = a {
        lazy(&mut node);
        meld(node.l.take(), node.r.take())
    } else { None }
}

pub struct Edge { pub from: Index, pub to: Index, pub cost: i64 }

pub fn msa(n: usize, r: Index, edges: &[Edge]) -> i64 {
    let mut uf = UnionFind::new(n);
    let mut come: Vec<Option<Box<Heap>>> = Vec::with_capacity(n);
    come.resize(n, None);
    let mut used = vec![0; n];
    let mut from = vec![0; n];
    let mut from_cost = vec![0; n];
    used[r] = 2;

    // build initial heaps
    for (i, e) in edges.iter().enumerate() {
        let node = Heap::new(e.cost, i);
        come[e.to] = meld(come[e.to].take(), Some(node));
    }

    let mut res = 0i64;
    for start in 0..n {
        if used[start] != 0 { continue; }
        let mut processing = Vec::new();
        let mut cur = start;
        while used[cur] != 2 {
            // mark as processing
            used[cur] = 1;
            processing.push(cur);
            // no incoming edges
            if come[cur].is_none() { return INF; }
            // take smallest incoming edge
            let mut heap = come[cur].take().unwrap();
            lazy(&mut heap);
            let eidx = heap.id;
            from[cur] = uf.find(edges[eidx].from);
            from_cost[cur] = heap.v;
            come[cur] = pop(Some(heap));

            // ignore self loops
            if from[cur] == cur { continue; }
            res += from_cost[cur];

            // cycle detected
            if used[from[cur]] == 1 {
                let mut p = cur;
                loop {
                    if let Some(ref mut h) = come[p] { h.add -= from_cost[p]; }
                    if p != cur {
                        uf.unite(p, cur);
                        come[cur] = meld(come[cur].take(), come[p].take());
                    }
                    p = uf.find(from[p]);
                    if p == cur { break; }
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

mod tests {
    use crate::algorithms::spanning_tree::tarjan::{msa, Edge, INF};

    #[test]
    fn test_msa() {
        let n: usize = 4;
        let m: usize = 4;
        let r: usize = 0;
        let mut edges = Vec::with_capacity(m);

        edges.push(Edge { from: 0, to: 1, cost: 10 });
        edges.push(Edge { from: 0, to: 2, cost: 10 });
        edges.push(Edge { from: 0, to: 3, cost: 3 });
        edges.push(Edge { from: 3, to: 2, cost: 4 });

        let cost = msa(n, r, &edges);
        if cost >= INF {
            println!("Impossible");
        } else {
            println!("{}", cost);
        }
    }

    #[test]
    fn test_msa2() {
        let n: usize = 4;
        let m: usize = 6;
        let r: usize = 0;
        let mut edges = Vec::with_capacity(m);

        edges.push(Edge { from: 0, to: 1, cost: 3 });
        edges.push(Edge { from: 0, to: 2, cost: 2 });
        edges.push(Edge { from: 2, to: 0, cost: 1 });
        edges.push(Edge { from: 2, to: 3, cost: 1 });
        edges.push(Edge { from: 3, to: 0, cost: 1 });
        edges.push(Edge { from: 3, to: 1, cost: 5 });

        let cost = msa(n, r, &edges);
        assert_eq!(cost, 6);
    }
}
