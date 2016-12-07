#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#[macro_use]
extern crate debug_unreachable;

mod iterator;
mod node;
mod sparse_array;
mod trie;

pub use iterator::TriePrefixIterator;
pub use trie::Trie;
