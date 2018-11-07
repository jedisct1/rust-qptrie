use super::iterator::TriePrefixIterator;
use super::node::{InternalNode, LeafNode, Node};
use super::sparse_array::SparseArray;
use std::usize;

use std::{cmp, mem};

const COMPLETE_KEY_NIBBLE: usize = 0;

/// A qp-trie.
#[derive(Clone, Debug)]
pub struct Trie<TK: PartialEq + AsRef<[u8]>, TV> {
    root: Option<Node<TK, TV>>,
    max_height: usize,
}

impl<TK: PartialEq + AsRef<[u8]>, TV> Default for Trie<TK, TV> {
    fn default() -> Self {
        Trie {
            root: None,
            max_height: usize::MAX,
        }
    }
}

impl<TK: PartialEq + AsRef<[u8]>, TV> Trie<TK, TV> {
    fn nibble(key: &[u8], index: usize) -> usize {
        let key_len = key.len();
        if index / 2 >= key_len {
            COMPLETE_KEY_NIBBLE
        } else {
            let nibble = key[index / 2];
            if (index & 1) == 0 {
                1 + (nibble >> 4) as usize
            } else {
                1 + (nibble & 0xf) as usize
            }
        }
    }

    fn find_closest_leaf_mut(
        root: &mut Node<TK, TV>,
        key: &[u8],
    ) -> (*mut LeafNode<TK, TV>, usize) {
        let mut height = 0;
        let mut t: *mut Node<TK, TV> = root;
        unsafe {
            while let Node::Internal(ref mut internal) = *t {
                let internal_index = internal.index;
                let nibble = Self::nibble(key, internal_index);
                t = internal.nibbles.get_or_head_mut(nibble);
                height += 1;
            }
            ((*t).as_mut_leaf(), height)
        }
    }

    #[allow(dead_code)]
    fn find_closest_leaf<'t>(root: &'t Node<TK, TV>, key: &[u8]) -> (&'t LeafNode<TK, TV>, usize) {
        let mut height = 0;
        let mut t: &Node<TK, TV> = root;
        while let Node::Internal(ref internal) = *t {
            let internal_index = internal.index;
            let nibble = Self::nibble(key, internal_index);
            t = internal.nibbles.get_or_head(nibble);
            height += 1;
        }
        (t.as_leaf(), height)
    }

    fn find_exact_leaf_mut(root: &mut Node<TK, TV>, key: &[u8]) -> Option<*mut LeafNode<TK, TV>> {
        let mut t: *mut Node<TK, TV> = root;
        unsafe {
            while let Node::Internal(ref mut internal) = *t {
                let internal_index = internal.index;
                let nibble = Self::nibble(key, internal_index);
                t = match internal.nibbles.get_mut(nibble) {
                    None => return None,
                    Some(t) => t,
                };
            }
            let leaf = (*t).as_mut_leaf();
            if key != leaf.key.as_ref() {
                return None;
            }
            Some(leaf)
        }
    }

    fn find_exact_leaf<'t>(root: &'t Node<TK, TV>, key: &[u8]) -> Option<&'t LeafNode<TK, TV>> {
        let mut t: &Node<TK, TV> = root;
        while let Node::Internal(ref internal) = *t {
            let internal_index = internal.index;
            let nibble = Self::nibble(key, internal_index);
            t = match internal.nibbles.get(nibble) {
                None => return None,
                Some(t) => t,
            };
        }
        let leaf = t.as_leaf();
        if key != leaf.key.as_ref() {
            return None;
        }
        Some(leaf)
    }

    fn new_internal_for_shorter_index(
        t: *mut Node<TK, TV>,
        leaf_key: &TK,
        index: usize,
        key: TK,
        val: TV,
    ) {
        let mut new_internal = InternalNode {
            nibbles: SparseArray::with_capacity(2),
            index,
        };
        let orig_nibble = Self::nibble(leaf_key.as_ref(), index);
        let new_nibble = Self::nibble(key.as_ref(), index);
        let new_leaf = Node::Leaf(LeafNode { key, val });
        debug_assert!(orig_nibble != new_nibble);
        let orig_node = unsafe { mem::replace(&mut *t, Node::Empty) };
        new_internal.nibbles.set(orig_nibble, orig_node);
        new_internal.nibbles.set(new_nibble, new_leaf);
        unsafe { *t = Node::Internal(new_internal) };
    }

    fn replace_leaf_with_internal_node(
        t: *mut Node<TK, TV>,
        leaf: &LeafNode<TK, TV>,
        index: usize,
        key: TK,
        val: TV,
    ) {
        let mut new_internal = InternalNode {
            nibbles: SparseArray::with_capacity(2),
            index,
        };
        let orig_nibble = Self::nibble(leaf.key.as_ref(), index);
        let new_nibble = Self::nibble(key.as_ref(), index);
        let new_leaf = Node::Leaf(LeafNode { key, val });
        debug_assert!(orig_nibble != new_nibble);
        let orig_node = unsafe { mem::replace(&mut *t, Node::Empty) };
        new_internal.nibbles.set(orig_nibble, orig_node);
        new_internal.nibbles.set(new_nibble, new_leaf);
        unsafe { *t = Node::Internal(new_internal) };
    }

    /// Creates a new, empty qp-trie.
    pub fn new() -> Self {
        Self::default()
    }

    /// Refuses to insert nodes that would make the trie height greater than `max_height`.
    pub fn max_height(mut self, max_height: usize) -> Self {
        self.max_height = max_height;
        self
    }

    /// Returns `true` if the trie is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    #[doc(hidden)]
    #[inline]
    pub fn root(&self) -> Option<&Node<TK, TV>> {
        self.root.as_ref()
    }

    /// Inserts a new node with the key `key`.
    pub fn insert(&mut self, key: TK, val: TV) -> bool {
        if self.root.is_none() {
            let leaf = LeafNode { key, val };
            self.root = Some(Node::Leaf(leaf));
            return true;
        }
        let (leaf, height) = unsafe {
            let closest = Self::find_closest_leaf_mut(self.root.as_mut().unwrap(), key.as_ref());
            (&mut *closest.0, closest.1)
        };
        let leaf_key = &leaf.key;
        let mut i = 0;
        let mut x = 0;
        let key_len = key.as_ref().len();
        let leaf_key_len = leaf_key.as_ref().len();
        let min_keys_len = cmp::min(key_len, leaf_key_len);
        while i < min_keys_len {
            x = key.as_ref()[i] ^ leaf_key.as_ref()[i];
            if x != 0 {
                break;
            }
            i += 1;
        }
        if x == 0 {
            if key_len == leaf_key_len {
                leaf.val = val;
                return false;
            }
            x = 0xff;
        }
        let mut index = i * 2;
        if (x & 0xf0) == 0 {
            index += 1;
        }
        let mut t: *mut Node<TK, TV> = self.root.as_mut().unwrap();
        loop {
            match *unsafe { &mut *t } {
                Node::Leaf(ref leaf) => {
                    if height >= self.max_height {
                        return false;
                    }
                    Self::replace_leaf_with_internal_node(t, leaf, index, key, val);
                    return true;
                }
                Node::Internal(ref mut internal) => {
                    if internal.index <= index {
                        let new_nibble = Self::nibble(key.as_ref(), internal.index);
                        if let Some(t_next) = internal.nibbles.get_mut(new_nibble) {
                            t = t_next;
                            continue;
                        }
                        let new_leaf = Node::Leaf(LeafNode { key, val });
                        internal.nibbles.set(new_nibble, new_leaf);
                        return true;
                    }
                    if height >= self.max_height {
                        return false;
                    }
                    Self::new_internal_for_shorter_index(t, leaf_key, index, key, val);
                    return true;
                }
                _ => unsafe { debug_unreachable!() },
            }
        }
    }

    /// Returns the value associated with the key `key`, or `None` if the key is not present in the trie.
    pub fn get(&self, key: &TK) -> Option<&TV> {
        let root = match self.root.as_ref() {
            None => return None,
            Some(root) => root,
        };
        let leaf = match Self::find_exact_leaf(root, key.as_ref()) {
            None => return None,
            Some(leaf) => leaf,
        };
        Some(&leaf.val)
    }

    /// Returns a mutable value associated with the key `key`, or `None` if the key is not present in the trie.
    pub fn get_mut(&mut self, key: &TK) -> Option<&mut TV> {
        let root = match self.root.as_mut() {
            None => return None,
            Some(root) => root,
        };
        let leaf = match Self::find_exact_leaf_mut(root, key.as_ref()) {
            None => return None,
            Some(leaf) => leaf,
        };
        Some(unsafe { &mut (*leaf).val })
    }

    /// Removes the node associated with the key `key`.
    ///
    /// Returns `true` if the key was found, or `false` if the operation was a no-op.
    pub fn remove(&mut self, key: &TK) -> bool {
        if self.root.is_none() {
            return false;
        }
        let mut t: *mut Node<TK, TV> = self.root.as_mut().unwrap();
        let (leaf, parent, nibble) = unsafe {
            let mut parent = None;
            let mut nibble = 0;
            while let Node::Internal(ref mut internal) = *t {
                let internal_index = internal.index;
                nibble = Self::nibble(key.as_ref(), internal_index);
                parent = Some(&mut *t);
                t = match internal.nibbles.get_mut(nibble) {
                    None => return false,
                    Some(t) => t,
                };
            }
            (&mut *t, parent, nibble)
        };
        if key.as_ref()[..] != leaf.as_leaf().key.as_ref()[..] {
            return false;
        }
        let parent: &mut Node<TK, TV> = match parent {
            None => {
                self.root = None;
                return true;
            }
            Some(parent) => parent,
        };
        parent.as_mut_internal().nibbles.remove(nibble);
        debug_assert!(!parent.as_internal().nibbles.is_empty());
        if parent.as_internal().nibbles.len() == 1 {
            *parent = parent.as_mut_internal().nibbles.pop();
        }
        true
    }

    #[doc(hidden)]
    pub fn prefix_find_next<'t>(
        &self,
        prefix: &TK,
        todo: &mut Vec<&'t Node<TK, TV>>,
        include_prefix: bool,
    ) -> Option<&'t LeafNode<TK, TV>> {
        let prefix_len = prefix.as_ref().len();
        while let Some(t) = todo.pop() {
            match *t {
                Node::Leaf(ref leaf) => {
                    if leaf.key.as_ref().starts_with(prefix.as_ref()) {
                        if include_prefix || leaf.key.as_ref() != prefix.as_ref() {
                            return Some(leaf);
                        }
                    }
                }
                Node::Internal(ref internal) => {
                    if internal.index / 2 >= prefix_len {
                        for node in internal.nibbles.all().iter().rev() {
                            todo.push(node);
                        }
                    } else {
                        let nibble = Self::nibble(prefix.as_ref(), internal.index);
                        if let Some(node) = internal.nibbles.get(nibble) {
                            todo.push(node)
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        None
    }

    /// Creates a new iterator over all the nodes whose key includes `prefix` as a prefix.
    pub fn prefix_iter<'t>(&'t self, prefix: &'t TK) -> TriePrefixIterator<TK, TV> {
        TriePrefixIterator::new(self, prefix, false)
    }
}
