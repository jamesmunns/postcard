# Atoms

An atom is a [type] whose [encoding] is fully defined by this specification. An
atom contains no values of types defined by the user.

## Atom Types

This specification defines the following atom types, where `P` is the pointer
width, in bits, of the platform encoding or decoding the value.

| Type     | Values                                   | Encoding                                                 |
| :---     | :---                                     | :---                                                     |
| `unit`   | The single value `()`                    | [Zero bytes](#atoms--fixed-size-atoms--unit)             |
| `bool`   | `true` or `false`                        | [One byte](#atoms--fixed-size-atoms--bool)               |
| `u8`     | Integers in `0..=(2^8 - 1)`              | [One byte](#atoms--fixed-size-atoms--u8)                 |
| `u16`    | Integers in `0..=(2^16 - 1)`             | [`varint(u16)`](#atoms--varint-encoded-integers)         |
| `u32`    | Integers in `0..=(2^32 - 1)`             | [`varint(u32)`](#atoms--varint-encoded-integers)         |
| `u64`    | Integers in `0..=(2^64 - 1)`             | [`varint(u64)`](#atoms--varint-encoded-integers)         |
| `u128`   | Integers in `0..=(2^128 - 1)`            | [`varint(u128)`](#atoms--varint-encoded-integers)        |
| `usize`  | Integers in `0..=(2^P - 1)`              | [`varint(usize)`](#atoms--usize-and-isize)               |
| `i8`     | Integers in `-(2^7)..=(2^7 - 1)`         | [One byte](#atoms--fixed-size-atoms--i8)                 |
| `i16`    | Integers in `-(2^15)..=(2^15 - 1)`       | [`varint(i16)`](#atoms--varint-encoded-integers)         |
| `i32`    | Integers in `-(2^31)..=(2^31 - 1)`       | [`varint(i32)`](#atoms--varint-encoded-integers)         |
| `i64`    | Integers in `-(2^63)..=(2^63 - 1)`       | [`varint(i64)`](#atoms--varint-encoded-integers)         |
| `i128`   | Integers in `-(2^127)..=(2^127 - 1)`     | [`varint(i128)`](#atoms--varint-encoded-integers)        |
| `isize`  | Integers in `-(2^(P-1))..=(2^(P-1) - 1)` | [`varint(isize)`](#atoms--usize-and-isize)               |
| `f32`    | IEEE 754 "binary32" values               | [Four bytes](#atoms--fixed-size-atoms--f32)              |
| `f64`    | IEEE 754 "binary64" values               | [Eight bytes](#atoms--fixed-size-atoms--f64)             |
| `bytes`  | Any number of bytes                      | [Length-prefixed](#atoms--length-prefixed-atoms--bytes)  |
| `string` | Any number of bytes that are valid UTF-8 | [Length-prefixed](#atoms--length-prefixed-atoms--string) |
| `char`   | A single Unicode scalar value            | [Length-prefixed](#atoms--length-prefixed-atoms--char)   |

[Headers], such as [lengths][length] and [discriminants][discriminant], are also
atoms. A header only appears in the encoding of another type, and is not used
directly in a schema.

## `varint`-Encoded Integers

Integers other than `u8` and `i8` are encoded in a variable-length format,
commonly known as [unsigned LEB128, or "varint"]. `varint(N)` refers to the
varint encoding of the integer type `N`, such as `varint(u16)`.

> postcard[atom.varint.encoding.continuation]
> All `varint(N)` types MUST be encoded as a variable length stream of bytes, as
> follows:
>
> * The most significant bit of each byte is a "continuation" flag: `1` if
>   another byte follows, and `0` on the last byte.
> * The seven least significant bits of each byte are data bits.

postcard[atom.varint.encoding.little-endian]
All `varint(N)` types MUST be encoded in "little endian" order, meaning that the
first byte contains the least significant seven data bits, and each following
byte contains the next seven more significant data bits.

postcard[atom.varint.types]
Every integer type other than `u8` and `i8` MUST be encoded as a `varint` of its
own type, such as `varint(u16)` for a `u16`, as listed in [Atom Types].

### Unsigned Integer Encoding

For example, the following 16-bit unsigned numbers would be encoded as follows:

| Dec   | Hex       | `varint` Encoded      | Length    |
| ---:  | :---      | :---                  | :---      |
| 0     | `0x00_00` | `[0x00]`              | 1         |
| 127   | `0x00_7F` | `[0x7F]`              | 1         |
| 128   | `0x00_80` | `[0x80, 0x01]`        | 2         |
| 16383 | `0x3F_FF` | `[0xFF, 0x7F]`        | 2         |
| 16384 | `0x40_00` | `[0x80, 0x80, 0x01]`  | 3         |
| 16385 | `0x40_01` | `[0x81, 0x80, 0x01]`  | 3         |
| 65535 | `0xFF_FF` | `[0xFF, 0xFF, 0x03]`  | 3         |

### Signed Integer Encoding

In [Two's Complement] form, every negative value has its most significant bit
set, so a negative value encoded directly would always take the maximum encoded
length.

postcard[atom.varint.encoding.zigzag]
A signed integer MUST be [Zigzag encoded] before it is encoded as a `varint`.
The zigzag value of a signed integer `n` is `2n` if `n` is zero or positive, and
`-2n - 1` if `n` is negative.

A decoder reverses this: a zigzag value `z` whose least significant bit is zero
decodes as `z / 2`, and one whose least significant bit is one decodes as
`-(z / 2) - 1`.

For example, the following 16-bit signed numbers would be encoded as follows:

| Dec       | Hex\*     | Zigzag (hex)      | `varint` Encoded      | Length |
| ---:      | :---      | :---              | :---                  | :---   |
| 0         | `0x00_00` | `0x00_00`         | `[0x00]`              | 1      |
| -1        | `0xFF_FF` | `0x00_01`         | `[0x01]`              | 1      |
| 1         | `0x00_01` | `0x00_02`         | `[0x02]`              | 1      |
| 63        | `0x00_3F` | `0x00_7E`         | `[0x7E]`              | 1      |
| -64       | `0xFF_C0` | `0x00_7F`         | `[0x7F]`              | 1      |
| 64        | `0x00_40` | `0x00_80`         | `[0x80, 0x01]`        | 2      |
| -65       | `0xFF_BF` | `0x00_81`         | `[0x81, 0x01]`        | 2      |
| 32767     | `0x7F_FF` | `0xFF_FE`         | `[0xFE, 0xFF, 0x03]`  | 3      |
| -32768    | `0x80_00` | `0xFF_FF`         | `[0xFF, 0xFF, 0x03]`  | 3      |

`*`: This column is represented as a sixteen bit, two's complement form

### Maximum Encoded Length

> postcard[atom.varint.max-len]
> As `varint`s encode seven data bits for every encoded byte, the maximum
> encoded length in bytes of a `varint(N)` is defined as `ceil(W / 7)`, where
> `W` is the width of `N` in bits.
>
> The following table expresses the maximum encoded length for each type:
>
> | Type      | Width (bits)   | Maximum encoded length (bytes) |
> | :---      | :---           | :---                           |
> | `u16`     | 16             | 3                              |
> | `i16`     | 16             | 3                              |
> | `u32`     | 32             | 5                              |
> | `i32`     | 32             | 5                              |
> | `u64`     | 64             | 10                             |
> | `i64`     | 64             | 10                             |
> | `u128`    | 128            | 19                             |
> | `i128`    | 128            | 19                             |
> | `usize`   | 16, 32, or 64  | 3, 5, or 10                    |
> | `isize`   | 16, 32, or 64  | 3, 5, or 10                    |

### Canonicalization

A `varint(N)` is canonical if it uses the fewest bytes that can encode its
value.

postcard[atom.varint.encode.canonical]
An encoder MUST always produce the canonical encoded form of a `varint(N)`.

postcard[atom.varint.decode.non-canonical]
A decoder MUST accept a non-canonical `varint(N)` that meets
[`atom.varint.decode.max-len`] and [`atom.varint.decode.max-value`].

postcard[atom.varint.decode.max-len]
A decoder MUST reject a `varint(N)` that is longer than the
[Maximum Encoded Length] of `N`.

postcard[atom.varint.decode.max-value]
A decoder MUST reject a `varint(N)` whose data bits form a value greater than
the maximum value of the unsigned type with the same width as `N`.

For example, the zigzag value of an `i16` cannot exceed the maximum value of a
`u16`.

For example, in the following `u16` encoded data:

| Data          | Encoded Form                  | Canonical?    | Accepted? |
| :---          | :---                          | :---          | :---      |
| 0             | `[0x00]`                      | Yes           | Yes       |
| 0             | `[0x80, 0x00]`                | No\*          | Yes       |
| 0             | `[0x80, 0x80, 0x00]`          | No\*          | Yes       |
| 0             | `[0x80, 0x80, 0x80, 0x00]`    | No\*          | No\*\*    |
| 65535         | `[0xFF, 0xFF, 0x03]`          | Yes           | Yes       |
| 131071        | `[0xFF, 0xFF, 0x07]`          | —             | No\*\*\*  |
| 65535         | `[0xFF, 0xFF, 0x83, 0x00]`    | No\*          | No\*\*    |

* \*: Contains excess encoding bytes
* \*\*: Exceeds the [Maximum Encoded Length] of the type
* \*\*\*: Exceeds the maximum value of the encoded type

## `usize` and `isize`

postcard[atom.usize.encoding]
A `usize` MUST be encoded as the unsigned integer type matching the pointer
width of the platform encoding it.

For example, on a platform with 32-bit pointers, `usize` maps to `u32`. On a
platform with 64-bit pointers, `usize` maps to `u64`.

A `usize` encoded on one platform can be decoded on a platform with a different
pointer width. As a result of [`atom.varint.decode.max-value`], a decoder
rejects a `varint(usize)` whose value exceeds the maximum value of its own
`usize`. For example, `0x1_0000_0000usize`, encoded on a 64-bit platform, is
rejected on a 32-bit platform.

postcard[atom.isize.encoding]
An `isize` MUST be encoded as the signed integer type matching the pointer
width of the platform encoding it.

In the same way, a decoder rejects a `varint(isize)` whose zigzag value exceeds
the maximum value of its own `usize`.

## Fixed-Size Atoms

### `unit`

postcard[atom.unit.encoding]
A `unit` MUST be encoded as zero bytes.

postcard[atom.unit.decode.empty]
A decoder MUST decode a `unit` by consuming zero bytes.

### `bool`

postcard[atom.bool.encoding]
A `bool` MUST be encoded as a single byte, with the value `0x00` for `false`,
and `0x01` for `true`.

postcard[atom.bool.decode.invalid]
A decoder MUST reject a `bool` encoded as any byte other than `0x00` or `0x01`.

### `u8`

postcard[atom.u8.encoding]
A `u8` MUST be encoded as a single byte containing its value.

postcard[atom.u8.decode.any]
A decoder MUST accept every value of a `u8`.

### `i8`

postcard[atom.i8.encoding]
An `i8` MUST be encoded as a single byte, in two's complement form.

postcard[atom.i8.decode.any]
A decoder MUST accept every value of an `i8`.

### `f32`

An `f32` is a "binary32" value, as defined in [IEEE 754-2008].

postcard[atom.f32.encoding]
An `f32` MUST be encoded as its binary32 bit pattern, as four bytes in
little-endian order.

For example, the float value `-32.005859375f32` would be bitwise represented as
`0xC200_0600u32`, and encoded as `[0x00, 0x06, 0x00, 0xC2]`.

postcard[atom.f32.decode.any]
A decoder MUST accept every bit pattern of an `f32`. For a bit pattern that is
not a NaN, it MUST produce the `f32` with that bit pattern.

postcard[atom.f32.decode.nan]
For a NaN bit pattern, a decoder MUST produce a NaN, and SHOULD produce the
`f32` with that bit pattern.

### `f64`

An `f64` is a "binary64" value, as defined in [IEEE 754-2008].

postcard[atom.f64.encoding]
An `f64` MUST be encoded as its binary64 bit pattern, as eight bytes in
little-endian order.

For example, the float value `-32.005859375f64` would be bitwise represented as
`0xC040_00C0_0000_0000u64`, and encoded as
`[0x00, 0x00, 0x00, 0x00, 0xC0, 0x00, 0x40, 0xC0]`.

postcard[atom.f64.decode.any]
A decoder MUST accept every bit pattern of an `f64`. For a bit pattern that is
not a NaN, it MUST produce the `f64` with that bit pattern.

postcard[atom.f64.decode.nan]
For a NaN bit pattern, a decoder MUST produce a NaN, and SHOULD produce the
`f64` with that bit pattern.

Some platforms cannot preserve every NaN bit pattern. For example, a signalling
NaN can become a quiet NaN when it is loaded into a floating point register,
and some languages represent every NaN with the same bit pattern.

## Length-Prefixed Atoms

A length-prefixed atom contains a [Length], followed by that many bytes.

### `bytes`

postcard[atom.bytes.encoding]
A `bytes` value MUST be encoded as a [Length], followed by its bytes.

### `string`

postcard[atom.string.encoding]
A `string` MUST be encoded as a [Length], followed by its UTF-8 bytes.

postcard[atom.string.decode.utf8]
A decoder MUST reject a `string` whose bytes are not well-formed UTF-8, as
defined by [Unicode 14.0 Chapter 3] Section 3.9, Definition D92 and Table 3-7.

### `char`

postcard[atom.char.encoding]
A `char` MUST be encoded in UTF-8 form, as a `string`.

postcard[atom.char.decode.one-scalar]
A decoder MUST reject a `char` whose `string` does not contain exactly one
Unicode scalar value.

A Unicode scalar value is defined by [Unicode 14.0 Chapter 3] Section 3.9,
Definition D76, as any code point other than a surrogate: `0x0000..=0xD7FF` or
`0xE000..=0x10FFFF`.

## Headers

A header is an atom whose value determines the number or type of the values that
follow it, in a [Composite][Composites] or a length-prefixed atom.

The `bool` that begins an [Option] is also considered a header, as it determines
whether a value follows it.

### Length

postcard[atom.length.encoding]
A length MUST be encoded as a `varint(usize)`, containing the number of
elements (for sequences), or bytes (for length-prefixed atoms), that follow it.

This specification does not limit a length, other than to the maximum value of
`usize`. See [Lengths].

### Discriminant

postcard[atom.discriminant.encoding]
A discriminant MUST be encoded as a `varint(u32)`.

[Atom Types]: #atoms--atom-types
[Composites]: #composites
[Headers]: #atoms--headers
[IEEE 754-2008]: https://standards.ieee.org/ieee/754/4211/
[unsigned LEB128, or "varint"]: https://en.wikipedia.org/wiki/Variable-length_quantity
[Length]: #atoms--headers--length
[Lengths]: #implementation-considerations--security--lengths
[Maximum Encoded Length]: #atoms--varint-encoded-integers--maximum-encoded-length
[Option]: #composites--option
[Two's Complement]: https://en.wikipedia.org/wiki/Two%27s_complement
[Unicode 14.0 Chapter 3]: https://www.unicode.org/versions/Unicode14.0.0/ch03.pdf
[Zigzag encoded]: https://en.wikipedia.org/wiki/Variable-length_quantity#Zigzag_encoding
[`atom.varint.decode.max-value`]: #postcard-atom.varint.decode.max-value
[encoding]: #glossary--encoding
[type]: #glossary--type
[`atom.varint.decode.max-len`]: #postcard-atom.varint.decode.max-len
