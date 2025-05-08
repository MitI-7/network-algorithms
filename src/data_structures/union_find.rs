use std::collections::HashSet;

pub struct UnionFind {
    num_nodes: usize,
    set_size: usize,
    leaders: HashSet<usize>,
    parent: Vec<isize>,
    next: Vec<usize>,
}

#[allow(dead_code)]
impl UnionFind {
    pub fn new(num_nodes: usize) -> Self {
        Self {
            num_nodes,
            set_size: num_nodes,
            leaders: (0..num_nodes).collect(),
            parent: vec![-1; num_nodes],
            next: (0..num_nodes).collect(),
        }
    }

    pub fn same_set(&mut self, u: usize, v: usize) -> bool {
        assert!(u < self.num_nodes && v < self.num_nodes);
        self.find_root(u) == self.find_root(v)
    }

    pub fn unite(&mut self, mut u: usize, mut v: usize) -> bool {
        assert!(u < self.num_nodes && v < self.num_nodes);
        u = self.find_root(u);
        v = self.find_root(v);
        if u == v {
            return false;
        }

        if self.parent[u] > self.parent[v] {
            std::mem::swap(&mut u, &mut v);
        }

        self.parent[u] += self.parent[v];
        self.parent[v] = u as isize;
        self.next.swap(v, u);
        self.set_size -= 1;
        self.leaders.remove(&v);

        true
    }

    pub fn size(&mut self, u: usize) -> usize {
        let p = self.find_root(u);
        (-self.parent[p]) as usize
    }

    pub fn leader(&mut self, u: usize) -> usize {
        self.find_root(u)
    }

    pub fn group(&mut self, u: usize) -> Vec<usize> {
        let mut group = Vec::with_capacity(self.size(u));
        let mut now = self.find_root(u);
        for _ in 0..group.capacity() {
            group.push(now);
            now = self.next[now];
        }
        group
    }

    fn find_root(&mut self, u: usize) -> usize {
        if self.parent[u] < 0 {
            u
        } else {
            let root = self.find_root(self.parent[u] as usize);
            self.parent[u] = root as isize;
            root
        }
    }
}

mod tests {
    use super::UnionFind;

    #[test]
    fn test() {
        let mut uf = UnionFind::new(4);
        assert!(!uf.same_set(0, 1));
        uf.unite(0, 1);
        uf.unite(2, 3);
        assert!(uf.same_set(0, 1));
        assert!(!uf.same_set(1, 2));
        uf.unite(0, 2);
        assert!(uf.same_set(1, 3));
    }
}
