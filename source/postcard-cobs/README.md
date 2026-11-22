# postcard-cobs

[COBS] framing for [`postcard`]:

- `Cobs`, `to_slice_cobs` and `to_vec_cobs` encode while serializing.
- `CobsAccumulator` decodes and deserializes messages from a stream in chunks.
- `from_bytes_cobs` and `take_from_bytes_cobs` decode in place, then deserialize.

With the `postcard-schema-v0_5` feature, buffers can be sized from a type's schema:

```rust
static ACC: CobsAccumulator<{ max_frame_len_of::<Msg>() }> = CobsAccumulator::new();
```

Versions of this crate before 0.3 are unrelated to current functionality.

[COBS]: https://en.wikipedia.org/wiki/Consistent_Overhead_Byte_Stuffing
[`postcard`]: https://docs.rs/postcard

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
