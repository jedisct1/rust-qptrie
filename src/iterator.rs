use trie::Node;

pub struct TrieIterator<'s, TK: 's + PartialEq + AsRef<[u8]>, TV: 's> {
    t: &'s Node<TK, TV>,
    key: &'s [u8],
    gt: bool,
}

impl<'s, TK: 's + PartialEq + AsRef<[u8]>, TV: 's> TrieIterator<'s, TK, TV> {
    #[inline]
    pub fn new(t: &'s Node<TK, TV>, key: &'s [u8], gt: bool) -> Self {
        TrieIterator {
            t: t,
            key: key,
            gt: gt,
        }
    }
}

impl<'s, TK: PartialEq + AsRef<[u8]>, TV> TrieIterator<'s, TK, TV> {
    #[inline]
    pub fn different(mut self) -> Self {
        self.gt = true;
        self
    }
}

impl<'s, TK: PartialEq + AsRef<[u8]>, TV> Iterator for TrieIterator<'s, TK, TV> {
    type Item = (&'s TK, &'s TV);

    fn next(&mut self) -> Option<Self::Item> {
        let res = if self.gt {
            self.t.next_gt(self.key)
        } else {
            self.t.next_ge(self.key)
        };
        match res {
            None => None,
            Some((key, val)) => {
                self.key = key.as_ref();
                self.gt = true;
                Some((key, val))
            }
        }
    }
}

pub struct TriePrefixIterator<'s, TK: 's + PartialEq + AsRef<[u8]>, TV: 's> {
    t: &'s Node<TK, TV>,
    prefix: &'s [u8],
    key: &'s [u8],
    gt: bool,
}

impl<'s, TK: 's + PartialEq + AsRef<[u8]>, TV: 's> TriePrefixIterator<'s, TK, TV> {
    #[inline]
    pub fn new(t: &'s Node<TK, TV>, key: &'s [u8], gt: bool) -> Self {
        TriePrefixIterator {
            t: t,
            prefix: key,
            key: key,
            gt: gt,
        }
    }
}

impl<'s, TK: PartialEq + AsRef<[u8]>, TV> TriePrefixIterator<'s, TK, TV> {
    #[inline]
    pub fn different(mut self) -> Self {
        self.gt = true;
        self
    }
}

impl<'s, TK: PartialEq + AsRef<[u8]>, TV> Iterator for TriePrefixIterator<'s, TK, TV> {
    type Item = (&'s TK, &'s TV);

    fn next(&mut self) -> Option<Self::Item> {
        let res = if self.gt {
            self.t.next_gt(self.key)
        } else {
            self.t.next_ge(self.key)
        };
        match res {
            None => None,
            Some((key, _)) if !key.as_ref().starts_with(self.prefix) => None,
            Some((key, val)) => {
                self.key = key.as_ref();
                self.gt = true;
                Some((key, val))
            }
        }
    }
}
