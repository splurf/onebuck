# onebuck
An efficient unordered dynamically-sized data structure.

## Time Complexity
| Method | Time |
| ------ | ---- |
| insert | $O(1)$ |
| remove | $O(1)$ |
| grow   | $O(k)$ |
| shrink | $O(k)$ |
- $k \to \text{original capacity}$ 

## Memory Fragmentation
- Due to compaction on removal, this is essentially disregarded, resulting in incredibly fast iteration.
