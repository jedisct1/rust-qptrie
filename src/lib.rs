//! A qp-trie implementation for Rust
//!
//! A [qp-trie](http://dotat.at/prog/qp/) is a fast and compact associative array.
//! It is similar to a crit-bit trie, with a larger fan-out per internal node to save memory and
//! reduce lookup costs.
//!
//! It supports the following operations at high speed:
//!
//! * See whether a key is in the trie and retrieve an optional associated value
//! * Add a `(key, value)` pair to the trie
//! * Remove a key from the trie
//! * Find all keys matching a given prefix
//!
//! This implementation uses 4 bits per index and doesn't require keys to be zero-terminated.
//!
//! # Example
//! ```
//! use qptrie::Trie;
//!
//! let mut trie = Trie::new();
//! trie.insert("key number one", 1);
//! trie.insert("key number two", 2);
//!
//! for (k, v) in trie.prefix_iter(&"key").include_prefix() {
//!     println!("{} => {}", k, v);
//! }
//!
//! trie.remove(&"key number one");
//!
//! let v = trie.get(&"key number two").unwrap();
//! ```
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

#[macro_use]
extern crate debug_unreachable;

mod iterator;
mod node;
mod sparse_array;
#[cfg(test)]
mod test;
mod trie;

pub use iterator::TriePrefixIterator;
pub use trie::Trie;
