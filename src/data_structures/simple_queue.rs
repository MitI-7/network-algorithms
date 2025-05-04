#[derive(Default)]
pub struct SimpleQueue<T> {
    data: Vec<T>,
    head: usize,
}

impl<T> SimpleQueue<T> {
    pub fn new() -> Self {
        Self { data: Vec::new(), head: 0 }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self { data: Vec::with_capacity(cap), head: 0 }
    }

    #[inline]
    pub fn push(&mut self, value: T) {
        self.data.push(value);
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.head < self.data.len() {
            let value = std::mem::replace(&mut self.data[self.head], unsafe { std::mem::MaybeUninit::zeroed().assume_init() });
            self.head += 1;
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn peek(&self) -> Option<&T> {
        self.data.get(self.head)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.head >= self.data.len()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data.len() - self.head
    }

    #[inline]
    pub fn reset(&mut self) {
        self.data.clear();
        self.head = 0;
    }
}

impl<T> Extend<T> for SimpleQueue<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.data.extend(iter);
    }
}
