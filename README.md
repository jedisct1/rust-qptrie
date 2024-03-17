A QP-Trie implementation for Rust
=================================

## [API documentation](https://docs.rs/qptrie)

A [qp-trie](http://dotat.at/prog/qp/) is a fast and compact associative array.

It is similar to a crit-bit trie, with a larger fan-out per internal node
to save memory and reduce lookup costs.

It supports the following operations at high speed:

* See whether a key is in the trie and retrieve an optional associated value
* Add a `(key, value)` pair to the trie
* Remove a key from the trie
* Find all keys matching a given prefix

This implementation uses 4 bits per index and doesn't require keys to be
zero-terminated.

## Example
```rust
use qptrie::Trie;

let mut trie = Trie::new();
trie.insert("key number one", 1);
trie.insert("key number two", 2);

for (k, v) in trie.prefix_iter(&"key").include_prefix() {
     println!("{} => {}", k, v);
}

trie.remove(&"key number one");

let v = trie.get(&"key number two").unwrap();
```

## Benchmarks

~500,000 4-bytes keys accessed in random order
([source](https://gist.github.com/ce89f94dda19ca426110c7f82405ad45)),
using `rustc 1.15.0-dev (d9aae6362 2016-12-08)`:

```text
test test::bench_btreemap_get    ... bench: 112,349,209 ns/iter (+/- 9,450,753)
test test::bench_btreemap_insert ... bench: 115,952,204 ns/iter (+/- 7,066,195)
test test::bench_hashmap_get     ... bench:  52,239,122 ns/iter (+/- 2,225,861)
test test::bench_hashmap_insert  ... bench:  60,889,965 ns/iter (+/- 27,314,557)
test test::bench_qptrie_get      ... bench:  51,843,861 ns/iter (+/- 18,878,702)
test test::bench_qptrie_insert   ... bench:  67,449,566 ns/iter (+/- 16,887,173)
```

qp-tries are more than twice as fast as Rust's `BTreeMap`, and roughly as
fast as Rust's excellent `HashMap` implementation while being more
compact and allowing range queries.
