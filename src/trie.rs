use iterator::TriePrefixIterator;
use node::{Node, InternalNode, LeafNode};
use sparse_array::SparseArray;
use std::usize;

use std::{cmp, mem};

#[derive(Clone, Debug)]
pub struct Trie<TK: PartialEq + AsRef<[u8]>, TV> {
    root: Option<Node<TK, TV>>,
    max_height: usize,
}

const COMPLETE_KEY_NIBBLE: usize = 0;

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
        if index / 2 >= key.len() {
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

    fn find_closest_leaf_mut(root: &mut Node<TK, TV>,
                             key: &[u8])
                             -> (*mut LeafNode<TK, TV>, usize) {
        let mut height = 0;
        let mut t: *mut Node<TK, TV> = root;
        unsafe {
            while let Node::Internal(ref mut internal) = *t {
                let internal_index = internal.index;
                let nibble = Self::nibble(&key, internal_index);
                t = internal.nibbles.get_or_head_mut(nibble);
                height += 1;
            }
            debug_assert!((*t).is_leaf());
            ((*t).as_mut_leaf(), height)
        }
    }

    fn find_closest_leaf<'t>(root: &'t Node<TK, TV>, key: &[u8]) -> (&'t LeafNode<TK, TV>, usize) {
        let mut height = 0;
        let mut t: &Node<TK, TV> = root;
        while let Node::Internal(ref internal) = *t {
            let internal_index = internal.index;
            let nibble = Self::nibble(&key, internal_index);
            t = internal.nibbles.get_or_head(nibble);
            height += 1;
        }
        debug_assert!(t.is_leaf());
        (t.as_leaf(), height)
    }

    fn new_internal_for_shorter_index(t: *mut Node<TK, TV>,
                                      leaf_key: &TK,
                                      index: usize,
                                      key: TK,
                                      val: TV) {
        let mut new_internal = InternalNode {
            nibbles: SparseArray::with_capacity(2),
            index: index,
        };
        let orig_nibble = Self::nibble(leaf_key.as_ref(), index);
        let new_nibble = Self::nibble(key.as_ref(), index);
        let new_leaf = Node::Leaf(LeafNode {
            key: key,
            val: val,
        });
        debug_assert!(orig_nibble != new_nibble);
        let orig_node = unsafe { mem::replace(&mut *t, Node::Empty) };
        new_internal.nibbles.set(orig_nibble, orig_node);
        new_internal.nibbles.set(new_nibble, new_leaf);
        unsafe { *t = Node::Internal(new_internal) };
    }

    fn replace_leaf_with_internal_node(t: *mut Node<TK, TV>,
                                       leaf: &LeafNode<TK, TV>,
                                       index: usize,
                                       key: TK,
                                       val: TV) {
        let mut new_internal = InternalNode {
            nibbles: SparseArray::with_capacity(2),
            index: index,
        };
        let orig_nibble = Self::nibble(leaf.key.as_ref(), index);
        let new_nibble = Self::nibble(key.as_ref(), index);
        let new_leaf = Node::Leaf(LeafNode {
            key: key,
            val: val,
        });
        debug_assert!(orig_nibble != new_nibble);
        let orig_node = unsafe { mem::replace(&mut *t, Node::Empty) };
        new_internal.nibbles.set(orig_nibble, orig_node);
        new_internal.nibbles.set(new_nibble, new_leaf);
        unsafe { *t = Node::Internal(new_internal) };
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn max_height(mut self, max_height: usize) -> Self {
        self.max_height = max_height;
        self
    }

    #[inline]
    pub fn root(&self) -> Option<&Node<TK, TV>> {
        self.root.as_ref()
    }

    pub fn insert(&mut self, key: TK, val: TV) -> bool {
        if self.root.is_none() {
            let leaf = LeafNode {
                key: key,
                val: val,
            };
            self.root = Some(Node::Leaf(leaf));
            return true;
        }
        let (mut leaf, height) = unsafe {
            let closest = Self::find_closest_leaf_mut(self.root.as_mut().unwrap(), key.as_ref());
            (&mut *closest.0, closest.1)
        };
        let leaf_key = &leaf.key;
        let mut i = 0;
        let mut x = 0;
        let key_len = key.as_ref().len();
        let min_keys_len = cmp::min(key_len, leaf_key.as_ref().len());
        while i < min_keys_len {
            x = key.as_ref()[i] ^ leaf_key.as_ref()[i];
            if x != 0 {
                break;
            }
            i += 1;
        }
        if x == 0 {
            if key_len == leaf_key.as_ref().len() {
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
                        let new_leaf = Node::Leaf(LeafNode {
                            key: key,
                            val: val,
                        });
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

    pub fn get(&self, key: &TK) -> Option<&TV> {
        let (leaf, _) = Self::find_closest_leaf(self.root.as_ref().unwrap(), key.as_ref());
        if key.as_ref()[..] != leaf.key.as_ref()[..] {
            None
        } else {
            Some(&leaf.val)
        }
    }

    pub fn prefix_find(&self, prefix: &TK, include_prefix: bool) -> Vec<&LeafNode<TK, TV>> {
        let t = match self.root.as_ref() {
            None => return vec![],
            Some(root) => root,
        };
        let mut found = vec![];
        let mut todo = vec![t];
        while let Some(leaf) = self.prefix_find_next(&prefix, &mut todo, include_prefix) {
            found.push(leaf);
        }
        found
    }

    pub fn prefix_find_next<'t>(&self,
                                prefix: &TK,
                                todo: &mut Vec<&'t Node<TK, TV>>,
                                include_prefix: bool)
                                -> Option<&'t LeafNode<TK, TV>> {
        let prefix_len = prefix.as_ref().len();
        while let Some(t) = todo.pop() {
            match *t {
                Node::Leaf(ref leaf) => {
                    if leaf.key.as_ref().starts_with(prefix.as_ref()) {
                        if include_prefix || leaf.key.as_ref() != prefix.as_ref() {
                            return Some(&leaf);
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

    pub fn prefix_iter<'t>(&'t self, key: &'t TK) -> TriePrefixIterator<TK, TV> {
        TriePrefixIterator::new(&self, key, false)
    }
}
