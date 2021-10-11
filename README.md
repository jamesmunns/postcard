# Postcard

Postcard is a `#![no_std]` focused serializer and deserializer for Serde.

Postcard aims to be convenient for developers in constrained environments, while
allowing for flexibility to customize behavior as needed.

* [Documentation](https://docs.rs/postcard/)
* [Crates.io](https://crates.io/crates/postcard)
* [Changelog](./CHANGELOG.md)

## Design Goals

1. Design primarily for `#![no_std]` usage, in embedded or other constrained contexts
2. Support a maximal set of `serde` features, so `postcard` can be used as a drop in replacement
3. Avoid special differences in code between communication code written for a microcontroller or a desktop/server PC
4. Be resource efficient - memory usage, code size, developer time, and CPU time; in that order
5. Allow library users to customize the serialization and deserialization  behavior to fit their bespoke needs

## Variable Length Data

Variable length data (such as slices) are prefixed by their length.

Length is encoded as a [Varint]. This is done for two reasons: to minimize wasted bytes
on the wire when sending slices with items less than 127 items (typical for embedded),
and to reduce compatibility issues between 32-bit and 64-bit targets due to differing sizes
of `usize`.

Similarly, `enum` descriminants are encoded as varints, meaning that any enum with less than
127 variants will encode its discriminant as a single byte (rather than a `u32`).

Varints in `postcard` have a maximum value of the usize for that platform. In practice, this
means that 64-bit targets should not send messages with slices containing `(1 << 32) - 1` items
to 32-bit targets, which is uncommon in practice. Enum discriminants already have a fixed
maximum value of `(1 << 32) - 1` as currently defined in Rust. Varints larger than the current platform's
`usize` will cause the deserialization process to return an `Err`.

[Varint]: https://developers.google.com/protocol-buffers/docs/encoding

## Example - Serialization/Deserialization

Postcard can serialize and deserialize messages similar to other `serde` formats.
Using the default `heapless` feature to serialize to a `heapless::Vec<u8>`:

```rust
use core::ops::Deref;
use serde::{Serialize, Deserialize};
use postcard::{from_bytes, to_vec};
use heapless::Vec;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct RefStruct<'a> {
    bytes: &'a [u8],
    str_s: &'a str,
}
let message = "hElLo";
let bytes = [0x01, 0x10, 0x02, 0x20];
let output: Vec<u8, 11> = to_vec(&RefStruct {
    bytes: &bytes,
    str_s: message,
}).unwrap();

assert_eq!(
    &[0x04, 0x01, 0x10, 0x02, 0x20, 0x05, b'h', b'E', b'l', b'L', b'o',],
    output.deref()
);

let out: RefStruct = from_bytes(output.deref()).unwrap();
assert_eq!(
    out,
    RefStruct {
        bytes: &bytes,
        str_s: message,
    }
);
```

Or the optional `alloc` feature to serialize to an `alloc::vec::Vec<u8>`:
```rust
use core::ops::Deref;
use serde::{Serialize, Deserialize};
use postcard::{from_bytes, to_allocvec};
extern crate alloc;
use alloc::vec::Vec;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct RefStruct<'a> {
    bytes: &'a [u8],
    str_s: &'a str,
}
let message = "hElLo";
let bytes = [0x01, 0x10, 0x02, 0x20];
let output: Vec<u8> = to_allocvec(&RefStruct {
    bytes: &bytes,
    str_s: message,
}).unwrap();

assert_eq!(
    &[0x04, 0x01, 0x10, 0x02, 0x20, 0x05, b'h', b'E', b'l', b'L', b'o',],
    output.deref()
);

let out: RefStruct = from_bytes(output.deref()).unwrap();
assert_eq!(
    out,
    RefStruct {
        bytes: &bytes,
        str_s: message,
    }
);
```

## Example - Flavors

`postcard` supports a system called `Flavors`, which are used to modify the way
postcard serializes or processes serialized data. These flavors act as "plugins" or "middlewares"
during the serialization process, and can be combined to obtain complex protocol formats.

Here, we serialize the given data, while simultaneously encoding it using COBS (a "modification flavor"),
and placing the output in a byte slice (a "storage flavor").

Users of `postcard` can define their own Flavors that can be combined with existing Flavors.

```rust
use postcard::{
    serialize_with_flavor,
    flavors::{Cobs, Slice},
};

let mut buf = [0u8; 32];

let data: &[u8] = &[0x01, 0x00, 0x20, 0x30];
let buffer = &mut [0u8; 32];
let res = serialize_with_flavor::<[u8], Cobs<Slice>, &mut [u8]>(
    data,
    Cobs::try_new(Slice::new(buffer)).unwrap(),
).unwrap();

assert_eq!(res, &[0x03, 0x04, 0x01, 0x03, 0x20, 0x30, 0x00]);
```

## Setup - `Cargo.toml`

Don't forget to add [the `no-std` subset](https://serde.rs/no-std.html) of `serde` along with `postcard` to the `[dependencies]` section of your `Cargo.toml`!

``` toml
# Cargo.toml

[dependencies]
postcard = "0.7.2"

# By default, `serde` has the `std` feature enabled, which makes it unsuitable for embedded targets
# disabling default-features fixes this
serde = { version = "1.0.*", default-features = false }
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
