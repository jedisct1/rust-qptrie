
use iterator::TrieIterator;
use std::{mem, str};

type Bitmap = u16;

#[derive(Copy, Clone, Debug)]
struct FlagsIndex(u32);

impl FlagsIndex {
    #[inline]
    fn new(flags: u8, index: usize) -> FlagsIndex {
        debug_assert!(flags & 0x3 == flags);
        FlagsIndex(((index as u32) << 2) | (flags as u32))
    }

    #[inline]
    fn flags_get(&self) -> u8 {
        (self.0 & 0x3) as u8
    }

    #[inline]
    fn index_get(&self) -> usize {
        (self.0 >> 2) as usize
    }
}

#[derive(Debug)]
pub struct Leaf<TK: PartialEq + AsRef<[u8]>, TV> {
    key: TK,
    val: TV,
}

#[derive(Debug)]
pub struct Branch<TK: PartialEq + AsRef<[u8]>, TV> {
    twigs: Vec<Node<TK, TV>>,
    flags_index: FlagsIndex,
    bitmap: Bitmap,
}

#[derive(Debug)]
pub enum Node<TK: PartialEq + AsRef<[u8]>, TV> {
    Leaf(Leaf<TK, TV>),
    Branch(Branch<TK, TV>),
    Empty,
}

#[derive(Default, Debug)]
pub struct Trie<TK: PartialEq + AsRef<[u8]>, TV> {
    root: Option<Node<TK, TV>>,
}

impl<TK: PartialEq + AsRef<[u8]>, TV> Branch<TK, TV> {
    #[inline]
    fn twigoff(&self, b: Bitmap) -> usize {
        (self.bitmap & (b - 1)).count_ones() as usize
    }
}

impl<TK: PartialEq + AsRef<[u8]>, TV> Node<TK, TV> {
    #[inline]
    fn flags_index_get(&self) -> (u8, usize) {
        let branch = match *self {
            Node::Branch(ref branch) => branch,
            _ => unsafe { debug_unreachable!() },
        };
        let flags_index = branch.flags_index;
        (flags_index.flags_get(), flags_index.index_get())
    }

    #[inline]
    fn is_branch(&self) -> bool {
        match *self {
            Node::Branch(_) => true,
            _ => false,
        }
    }

    #[inline]
    fn twigbit(&self, key: &[u8]) -> Bitmap {
        let key_len = key.len();
        let (flags, index) = self.flags_index_get();
        let i = index as usize;
        if i >= key_len {
            return 1;
        }
        Node::<TK, TV>::nibbit(key[i], flags)
    }

    #[inline]
    fn has_twig(&self, bit: Bitmap) -> bool {
        let branch = match *self {
            Node::Branch(ref branch) => branch,
            _ => unsafe { debug_unreachable!() },
        };
        (branch.bitmap & bit) != 0
    }

    #[inline]
    fn twigoff(&self, b: Bitmap) -> usize {
        match *self {
            Node::Branch(ref branch) => branch.twigoff(b),
            _ => unsafe { debug_unreachable!() },
        }
    }

    #[inline]
    fn twig(&self, i: usize) -> &Node<TK, TV> {
        let branch = match *self {
            Node::Branch(ref branch) => branch,
            _ => unsafe { debug_unreachable!() },
        };
        &branch.twigs[i]
    }

    #[inline]
    fn twig_mut(&mut self, i: usize) -> &mut Node<TK, TV> {
        let branch = match *self {
            Node::Branch(ref mut branch) => branch,
            _ => unsafe { debug_unreachable!() },
        };
        &mut branch.twigs[i]
    }

    #[inline]
    fn twigoff_max(&self, b: Bitmap) -> (usize, usize) {
        let branch = match *self {
            Node::Branch(ref branch) => branch,
            _ => unsafe { debug_unreachable!() },
        };
        let off = self.twigoff(b);
        let max = branch.bitmap.count_ones() as usize;
        (off, max)
    }

    #[inline]
    fn nibbit(k: u8, flags: u8) -> Bitmap {
        let mask = ((flags.wrapping_sub(2)) ^ 0x0f) & 0xff;
        let shift = (2 - flags) << 2;
        (1 as Bitmap) << ((k & mask) >> shift)
    }

    pub fn next_ge<'s>(self: &'s Node<TK, TV>, key: &[u8]) -> Option<(&'s TK, &'s TV)> {
        if self.is_branch() {
            let (s, m) = self.twigoff_max(self.twigbit(key));
            for s in s..m {
                if let ret @ Some(_) = self.twig(s).next_ge(key) {
                    return ret;
                }
            }
            return None;
        }
        let leaf = match *self {
            Node::Leaf(ref leaf) => leaf,
            _ => unsafe { debug_unreachable!() },
        };
        Some((&leaf.key, &leaf.val))
    }

    pub fn next_gt<'s>(self: &'s Node<TK, TV>, key: &[u8]) -> Option<(&'s TK, &'s TV)> {
        if self.is_branch() {
            let (s, m) = self.twigoff_max(self.twigbit(key));
            for s in s..m {
                if let ret @ Some(_) = Self::next_gt(self.twig(s), key) {
                    return ret;
                }
            }
            return None;
        }
        let leaf = match *self {
            Node::Leaf(ref leaf) => leaf,
            _ => unsafe { debug_unreachable!() },
        };
        if leaf.key.as_ref() == key {
            None
        } else {
            Some((&leaf.key, &leaf.val))
        }
    }
}

impl<TK: PartialEq + AsRef<[u8]>, TV> Trie<TK, TV> {
    pub fn get(&self, key: &TK) -> Option<&TV> {
        let key = key.as_ref();
        if self.root.is_none() {
            return None;
        }
        if key.len() == 0 {
            return None;
        }
        let mut t = self.root.as_ref().unwrap();
        while t.is_branch() {
            let b = t.twigbit(key);
            if !t.has_twig(b) {
                return None;
            }
            t = t.twig(t.twigoff(b));
        }
        let leaf = match *t {
            Node::Leaf(ref leaf) => leaf,
            _ => unsafe { debug_unreachable!() },
        };
        if leaf.key.as_ref() != key {
            return None;
        }
        Some(&leaf.val)
    }

    pub fn insert(&mut self, key: TK, val: TV) -> bool {
        let key_len = match key.as_ref().len() {
            0 => panic!("key cannot be empty"),
            key_len if key_len > 0xffffff => panic!("key is too long"),
            key_len => key_len,
        };
        if self.root.is_none() {
            let new_node = Node::Leaf(Leaf {
                key: key,
                val: val,
            });
            self.root = Some(new_node);
            return true;
        }
        let mut t: *mut Node<TK, TV> = self.root.as_mut().unwrap();
        let t = unsafe {
            while (&*t).is_branch() {
                let b = (&*t).twigbit(key.as_ref());
                let i = if (&*t).has_twig(b) {
                    (&*t).twigoff(b)
                } else {
                    0
                };
                t = (&mut *t).twig_mut(i);
            }
            &mut *t
        };
        let leaf = match *t {
            Node::Leaf(ref mut leaf) => leaf,
            _ => unsafe { debug_unreachable!() },
        };
        let leaf_key = &leaf.key;
        let mut i = 0;
        let mut x = 0;
        let (mut k1, mut k2) = (0, 0);
        let leaf_key_len = leaf_key.as_ref().len();
        while i <= key_len {
            k1 = if i < key_len { key.as_ref()[i] } else { 0 };
            k2 = if i < leaf_key_len {
                leaf_key.as_ref()[i]
            } else {
                0
            };
            x = k1 ^ k2;
            if x != 0 {
                break;
            }
            i += 1;
        }
        if x == 0 {
            leaf.val = val;
            return false;
        }
        let f = if (x & 0xf0) != 0 { 1 } else { 2 };
        let mut t: *mut Node<TK, TV> = self.root.as_mut().unwrap();
        let (t, grow_branch) = unsafe {
            let mut grow_branch = false;
            while (&*t).is_branch() {
                let (flags, index) = (&*t).flags_index_get();
                if i == index && f == flags {
                    grow_branch = true;
                    break;
                }
                if (i == index && f < flags) || i < index {
                    break;
                }
                let b = (&*t).twigbit(key.as_ref());
                debug_assert!((&*t).has_twig(b));
                t = (&mut *t).twig_mut((&*t).twigoff(b));
            }
            (&mut *t, grow_branch)
        };
        let new_node = Node::Leaf(Leaf {
            key: key,
            val: val,
        });
        let b1 = Node::<TK, TV>::nibbit(k1, f);
        if grow_branch {
            Self::_grow_branch(t, b1, new_node);
        } else {
            let b2 = Node::<TK, TV>::nibbit(k2, f);
            Self::_new_branch(t, b1, b2, f, i, new_node);
        }
        true
    }

    fn _new_branch(t: &mut Node<TK, TV>,
                   b1: Bitmap,
                   b2: Bitmap,
                   f: u8,
                   i: usize,
                   new_node: Node<TK, TV>) {
        let twigs: Vec<Node<TK, TV>> = Vec::with_capacity(2);
        let mut new_t = Branch {
            twigs: twigs,
            flags_index: FlagsIndex::new(f, i),
            bitmap: b1 | b2,
        };
        let t_save = mem::replace(t, Node::Empty);
        if new_t.twigoff(b1) == 0 {
            new_t.twigs.push(new_node);
            new_t.twigs.push(t_save);
        } else {
            new_t.twigs.push(t_save);
            new_t.twigs.push(new_node);
        }
        *t = Node::Branch(new_t);
    }

    fn _grow_branch(t: &mut Node<TK, TV>, b1: Bitmap, new_node: Node<TK, TV>) {
        debug_assert!(!t.has_twig(b1));
        let branch = match *t {
            Node::Branch(ref mut branch) => branch,
            _ => unsafe { debug_unreachable!() },
        };
        let s = branch.twigoff(b1);
        branch.twigs.insert(s, new_node);
        branch.bitmap |= b1;
    }

    pub fn remove(&mut self, key: &TK) -> Option<TV> {
        if self.root.is_none() || key.as_ref().len() == 0 {
            return None;
        }
        let mut t: *mut Node<TK, TV> = self.root.as_mut().unwrap();
        let (t, p, b): (&mut Node<TK, TV>, _, _) = unsafe {
            let mut b = 0;
            let mut p = None;
            while (&*t).is_branch() {
                b = (&*t).twigbit(key.as_ref());
                if !(&*t).has_twig(b) {
                    return None;
                }
                p = Some(t);
                t = (&mut *t).twig_mut((&*t).twigoff(b));
            }
            (&mut *t, p, b)
        };
        match *t {
            Node::Leaf(ref leaf) => {
                if leaf.key != *key {
                    return None;
                }
            }
            _ => unsafe { debug_unreachable!() },
        }
        let val = match mem::replace(t, Node::Empty) {
            Node::Leaf(leaf) => leaf.val,
            _ => unsafe { debug_unreachable!() },
        };
        let t2: &mut Node<TK, TV> = match p {
            None => {
                return Some(val);
            }
            Some(t2) => unsafe { &mut *t2 },
        };
        let (s, m) = t.twigoff_max(b);
        if m == 2 {
            *t2 = mem::replace(t2.twig_mut(1 - s), Node::Empty);
        } else {
            let branch = match *t2 {
                Node::Branch(ref mut branch) => branch,
                _ => unsafe { debug_unreachable!() },
            };
            branch.twigs.remove(s);
            branch.twigs.shrink_to_fit();
            branch.bitmap &= !b;
        }
        Some(val)
    }

    pub fn prefix_iter<'s>(&'s self, key: &'s TK) -> TrieIterator<TK, TV> {
        TrieIterator::new(self.root.as_ref().unwrap(), key.as_ref(), false)
    }
}
