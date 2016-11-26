#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

mod iterator;
mod trie;

pub use iterator::TrieIterator;
pub use trie::Trie;
