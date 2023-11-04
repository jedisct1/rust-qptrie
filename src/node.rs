use super::sparse_array::SparseArray;

#[derive(Clone, Debug)]
pub struct InternalNode<TK: PartialEq + AsRef<[u8]>, TV> {
    pub index: usize,
    pub nibbles: SparseArray<Node<TK, TV>>,
}

#[derive(Clone, Debug)]
pub struct LeafNode<TK: PartialEq + AsRef<[u8]>, TV> {
    pub key: TK,
    pub val: TV,
}

#[derive(Clone, Debug, Default)]
pub enum Node<TK: PartialEq + AsRef<[u8]>, TV> {
    #[default]
    Empty,
    Internal(InternalNode<TK, TV>),
    Leaf(LeafNode<TK, TV>),
}



impl<TK: PartialEq + AsRef<[u8]>, TV> Node<TK, TV> {
    #[inline]
    pub fn is_internal(&self) -> bool {
        matches!(*self, Node::Internal(_))
    }

    #[inline]
    pub fn as_internal(&self) -> &InternalNode<TK, TV> {
        match *self {
            Node::Internal(ref internal) => internal,
            _ => unsafe { debug_unreachable!() },
        }
    }

    #[inline]
    pub fn as_mut_internal(&mut self) -> &mut InternalNode<TK, TV> {
        match *self {
            Node::Internal(ref mut internal) => internal,
            _ => unsafe { debug_unreachable!() },
        }
    }

    #[inline]
    pub fn is_leaf(&self) -> bool {
        matches!(*self, Node::Leaf(_))
    }

    #[inline]
    pub fn as_leaf(&self) -> &LeafNode<TK, TV> {
        match *self {
            Node::Leaf(ref leaf) => leaf,
            _ => unsafe { debug_unreachable!() },
        }
    }

    #[inline]
    pub fn as_mut_leaf(&mut self) -> &mut LeafNode<TK, TV> {
        match *self {
            Node::Leaf(ref mut leaf) => leaf,
            _ => unsafe { debug_unreachable!() },
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        matches!(*self, Node::Empty)
    }
}
