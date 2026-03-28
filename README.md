Doing the [1 Billion Row Challenge](https://1brc.dev/).

### Running
```
cargo run --release -- <measurements.txt>
```

See the [1brc](https://github.com/gunnarmorling/1brc) repository to know more
about creating the measurements.txt file.

## Status
Naive implementation: ~7min30sec

After some optimizations: 1 Billion rows in ~35seconds (single threaded)

On a laptop with Intel i5-1035g1 @ 2.0GHz.

## TODO
- better SIMD
- better hashing
- multithreading...?
- Reason about the correctness of all uses of 'unsafe'..;)
