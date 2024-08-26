# onebuck
[![Crate](https://img.shields.io/crates/v/onebuck.svg)](https://crates.io/crates/onebuck)

An efficient unordered dynamically-sized data structure.

## Time Complexity
| Method | Time |
| ------ | ---- |
| get    | `O(1)` |
| insert | `O(1)` |
| remove | `O(1)` |
| grow   | `O(k)` |
| shrink | `O(k)` |
- `k` - original capacity

## Memory Fragmentation
- Due to compaction on removal, this is essentially disregarded, resulting in incredibly fast iteration.

## Features
- `atomic` - uses `std::sync::Arc` instead of the default `std::rc::Rc` for thread safety.
- `clone` - allows `ValueIndex` to be cloneable, allowing for greater versatility.
- `get` (**default**) - Obtain a reference from the bucket at the indexed position.
