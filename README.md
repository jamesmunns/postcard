# Postcard

A (TODO), no_std, serde compatible message library for Rust.

## Design plans

1. usizes are [varint]s
2. enum variants are usizes
3. variable length data, like strings and vecs, are prefixed by their
length as a [varint]/usize, e.g. "hello" is [5, h, e, l, l, o]
4. [`heapless`] data structures are used as the first class
serialization/deserialization target.
5. there will be a `std` feature that allows you to either
auto-convert from heapless to non-heapless, or jump directly to that,
                    whatever is easier

[varint]: https://developers.google.com/protocol-buffers/docs/encoding
[`heapless`]: https://github.com/japaric/heapless
