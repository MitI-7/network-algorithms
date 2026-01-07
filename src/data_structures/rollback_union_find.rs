#[derive(Clone)]
pub struct RollbackUnionFind {
    parent: Vec<isize>,
    history: Vec<(usize, isize)>,
}

impl RollbackUnionFind {
    pub fn new(num_nodes: usize) -> Self {
        Self { parent: vec![-1; num_nodes], history: vec![] }
    }

    pub fn find(&self, mut v: usize) -> usize {
        while self.parent[v] >= 0 {
            v = self.parent[v] as usize
        }
        v
    }

    pub fn same(&self, a: usize, b: usize) -> bool {
        self.find(a) == self.find(b)
    }

    pub fn time(&self) -> usize {
        self.history.len()
    }

    pub fn union(&mut self, a: usize, b: usize) -> bool {
        let (x, y) = (self.find(a), self.find(b));
        if x == y {
            return false;
        }
        // if self.parent[x] > self.parent[y] {
        //     std::mem::swap(&mut x, &mut y);
        // }
        self.history.push((y, self.parent[y]));
        self.parent[x] += self.parent[y];
        self.parent[y] = x as isize;
        true
    }

    pub fn rollback(&mut self, t: usize) {
        while self.history.len() > t {
            let (v, old) = self.history.pop().unwrap();
            let p = self.parent[v] as usize;
            self.parent[p] -= old;
            self.parent[v] = old;
        }
    }
}
