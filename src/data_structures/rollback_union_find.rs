#[derive(Clone)]
pub struct RollbackUnionFind {
    p: Vec<i32>,
    hist: Vec<(usize, i32)>,
}

impl RollbackUnionFind {
    pub fn new(num_nodes: usize) -> Self {
        Self { p: vec![-1; num_nodes], hist: vec![] }
    }

    pub fn find(&self, mut v: usize) -> usize {
        while self.p[v] >= 0 {
            v = self.p[v] as usize
        }
        v
    }

    pub fn same(&self, a: usize, b: usize) -> bool {
        self.find(a) == self.find(b)
    }

    pub fn time(&self) -> usize {
        self.hist.len()
    }

    pub fn join(&mut self, a: usize, b: usize) -> bool {
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

    pub fn rollback(&mut self, t: usize) {
        while self.hist.len() > t {
            let (v, old) = self.hist.pop().unwrap();
            let p = self.p[v] as usize;
            self.p[p] -= old;
            self.p[v] = old;
        }
    }
}
