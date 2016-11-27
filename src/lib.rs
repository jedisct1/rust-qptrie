#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#[macro_use]
extern crate debug_unreachable;

mod iterator;
mod trie;

pub use iterator::{TrieIterator, TriePrefixIterator};
pub use trie::Trie;
