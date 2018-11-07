use super::node::Node;
use super::Trie;

/// An iterator over keys matching a prefix.
#[derive(Clone, Debug)]
pub struct TriePrefixIterator<'t, TK: 't + PartialEq + AsRef<[u8]>, TV: 't> {
    trie: &'t Trie<TK, TV>,
    prefix: &'t TK,
    todo: Vec<&'t Node<TK, TV>>,
    include_prefix: bool,
}

impl<'t, TK: 't + PartialEq + AsRef<[u8]>, TV: 't> TriePrefixIterator<'t, TK, TV> {
    pub fn new(trie: &'t Trie<TK, TV>, key: &'t TK, include_prefix: bool) -> Self {
        let todo = match trie.root() {
            None => vec![],
            Some(root) => vec![root],
        };
        TriePrefixIterator {
            trie,
            prefix: key,
            todo,
            include_prefix,
        }
    }
}

impl<'t, TK: PartialEq + AsRef<[u8]>, TV> TriePrefixIterator<'t, TK, TV> {
    /// If a key equal to the prefix itself is found, include it in the results.
    #[inline]
    pub fn include_prefix(mut self) -> Self {
        self.include_prefix = true;
        self
    }
}

impl<'t, TK: PartialEq + AsRef<[u8]>, TV> Iterator for TriePrefixIterator<'t, TK, TV> {
    type Item = (&'t TK, &'t TV);

    fn next(&mut self) -> Option<Self::Item> {
        match self
            .trie
            .prefix_find_next(self.prefix, &mut self.todo, self.include_prefix)
        {
            None => None,
            Some(leaf) => {
                self.include_prefix = false;
                Some((&leaf.key, &leaf.val))
            }
        }
    }
}
