pub struct BitVector {
    data: Box<[usize]>,
    size: usize,
}

impl BitVector {
    const BITS: usize = usize::BITS as usize;

    pub fn new(size: usize) -> Self {
        let num_blocks = size.div_ceil(Self::BITS);
        BitVector { data: vec![0; num_blocks].into_boxed_slice(), size }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.size
    }

    #[inline(always)]
    pub fn set(&mut self, index: usize, b: bool) {
        assert!(index < self.size);
        let block = index / Self::BITS;
        let offset = index % Self::BITS;
        if b {
            self.data[block] |= 1 << offset;
        } else {
            self.data[block] &= !(1 << offset);
        }
    }

    #[inline(always)]
    pub fn get(&self, index: usize) -> bool {
        assert!(index < self.size);
        let block = index / Self::BITS;
        let offset = index % Self::BITS;
        (self.data[block] >> offset) & 1 == 1
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.data.fill(0);
    }
}
