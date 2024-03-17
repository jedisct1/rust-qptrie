#![doc = include_str!("../README.md")]

#[macro_use]
extern crate debug_unreachable;

mod iterator;
mod node;
mod sparse_array;
#[cfg(test)]
mod test;
mod trie;

pub use self::iterator::TriePrefixIterator;
pub use self::trie::Trie;
