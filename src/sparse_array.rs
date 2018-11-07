type Bitmap = u32;

#[derive(Clone, Debug)]
pub struct SparseArray<TI> {
    array: Vec<TI>,
    bitmap: Bitmap,
}

impl<TI> Default for SparseArray<TI> {
    fn default() -> Self {
        SparseArray {
            array: vec![],
            bitmap: 0,
        }
    }
}

impl<TI> SparseArray<TI> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        debug_assert!(capacity <= Self::bitmap_size());
        SparseArray {
            array: Vec::with_capacity(capacity),
            bitmap: 0,
        }
    }

    #[inline]
    pub fn bitmap_size() -> usize {
        (0 as Bitmap).count_zeros() as usize
    }

    #[inline]
    pub fn has_sparse_index(&self, sparse_index: usize) -> bool {
        (self.bitmap & (1 << sparse_index)) != 0
    }

    #[inline]
    fn actual_index(&self, sparse_index: usize) -> usize {
        let mask = (1 << sparse_index) - 1;
        (self.bitmap & mask).count_ones() as usize
    }

    #[inline]
    pub fn get(&self, sparse_index: usize) -> Option<&TI> {
        if self.has_sparse_index(sparse_index) {
            Some(&self.array[self.actual_index(sparse_index)])
        } else {
            None
        }
    }

    #[inline]
    pub fn get_mut(&mut self, sparse_index: usize) -> Option<&mut TI> {
        if self.has_sparse_index(sparse_index) {
            let actual_index = self.actual_index(sparse_index);
            Some(&mut self.array[actual_index])
        } else {
            None
        }
    }

    #[inline]
    pub fn get_or_head(&self, sparse_index: usize) -> &TI {
        if self.has_sparse_index(sparse_index) {
            &self.array[self.actual_index(sparse_index)]
        } else {
            self.head()
        }
    }

    #[inline]
    pub fn get_or_head_mut(&mut self, sparse_index: usize) -> &mut TI {
        if self.has_sparse_index(sparse_index) {
            let actual_index = self.actual_index(sparse_index);
            &mut self.array[actual_index]
        } else {
            self.head_mut()
        }
    }

    pub fn set(&mut self, sparse_index: usize, item: TI) -> bool {
        let actual_index = self.actual_index(sparse_index);
        if !self.has_sparse_index(sparse_index) {
            debug_assert!(self.len() < Self::bitmap_size());
            self.bitmap |= 1 << sparse_index;
            self.array.insert(actual_index, item);
            true
        } else {
            self.array[actual_index] = item;
            false
        }
    }

    pub fn remove(&mut self, sparse_index: usize) {
        debug_assert!(self.has_sparse_index(sparse_index));
        self.bitmap &= !(1 << sparse_index);
        let actual_index = self.actual_index(sparse_index);
        self.array.remove(actual_index);
    }

    #[inline]
    pub fn head(&self) -> &TI {
        debug_assert!(!self.array.is_empty());
        &self.array[0]
    }

    #[inline]
    pub fn head_mut(&mut self) -> &mut TI {
        debug_assert!(!self.array.is_empty());
        &mut self.array[0]
    }

    #[inline]
    pub fn pop(&mut self) -> TI {
        debug_assert!(!self.array.is_empty());
        let sparse_index = self.bitmap.trailing_zeros();
        self.bitmap &= !(1 << sparse_index);
        self.array.remove(0)
    }

    #[inline]
    pub fn all(&self) -> &Vec<TI> {
        &self.array
    }

    #[inline]
    pub fn len(&self) -> usize {
        debug_assert_eq!(self.bitmap.count_ones() as usize, self.array.len());
        self.array.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.array.is_empty()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.bitmap = 0;
        self.array.clear();
    }
}
