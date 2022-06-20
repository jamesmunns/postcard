# The Postcard Wire Format

Postcard is responsible for translating between items that exist as part of [The Serde Data Model] into a binary representation.

[The Serde Data Model]: ../serde-data-model.md

This is commonly referred to as **Serialization**, or converting from Serde Data Model elements to a binary representation; or **Deserialization**, or converting from a binary representation to Serde Data Model elements.

## Stability

The Postcard wire format is considered stable as of v1.0.0 and above of Postcard. Breaking changes to the wire format would
be considered a breaking change to the library, and would necessitate the library being revised to v2.0.0, along with a
new version of this wire format specification addressing the v2.0.0 wire format.

## Non Self-Describing Format

Postcard is **NOT** considered a "Self Describing Format", meaning that users (Serializers and Deserializers) of postcard data are expected to have a mutual understanding of the encoded data.

In practice this requires all systems sending or receiving postcard encoded data share a common schema, often as a common Rust data-type library.

Backwards/forwards compatibility between revisions of a postcard schema are considered outside of the scope of the postcard wire format, and must be considered by the end users, if compatible revisions to an agreed-upon schema are necessary.

## `varint` encoded integers

For reasons of portability and compactness, many integers are encoded into a variable length format, commonly known as ["leb" or "varint"] encoded.

["leb" or "varint"]: https://en.wikipedia.org/wiki/Variable-length_quantity

For the remainder of this document, these variable length encoded values will be referred to as `varint(N)`, where `N` represents the encoded Serde Data Model type, such as `u16` (`varint(u16)`) or `i32` (`varint(i32)`).

Conceptually, all `varint(N)` types encode data in a similar way when considering a stream of bytes:

* The most significant bit of each stream byte is used as a "continuation" flag.
    * If the flag is `1`, then this byte is NOT the last byte that comprises this varint
    * If the flag is `0`, then this byte IS the last byte that comprises this varint

All `varint(N)` types are encoded in "little endian" order, meaning that the first byte will contain the least significant seven data bits.

Specifically, the following types are encoded as `varint`s in postcard:

| Type      | Varint Type       |
| :---      | :---              |
| `u16`     | `varint(u16)`     |
| `i16`     | `varint(i16)`     |
| `u32`     | `varint(u32)`     |
| `i32`     | `varint(i32)`     |
| `u64`     | `varint(u64)`     |
| `i64`     | `varint(i64)`     |
| `u128`    | `varint(u128)`    |
| `i128`    | `varint(i128)`    |

As `u8` and `i8` types always fit into a single byte, they are encoded as-is rather than encoded using a `varint`.

Additionally the following two types are not part of the Serde Data Model, but are used within the context of postcard:

| Type      | Varint Type       |
| :---      | :---              |
| `usize`   | `varint(usize)`   |
| `isize`   | `varint(isize)`   |

See the section [isize and usize] below for more details on how these types are used.

[isize and usize]: #isize-and-usize

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

Signed integers are typically "natively" encoded using a [Two's Compliment] form, meaning that the most significant bit is used
to offset the value by a large negative shift. If this form was used directly for encoding signed integer values, it would
have the negative effect that negative values would ALWAYS take the maximum encoded length to store on the wire.

[Two's Compliment]: https://en.wikipedia.org/wiki/Two%27s_complement

For this reason, signed integers, when encoded as a `varint`, are first [Zigzag encoded]. Zigzag encoding stores the sign bit in the
LEAST significant bit of the integer, rather than the MOST significant bit.

[Zigzag encoded]: https://en.wikipedia.org/wiki/Variable-length_quantity#Zigzag_encoding

This means that signed integers of low absolute magnitude (e.g. 1, -1) can be encoded using a much smaller space.

For example, the following 16-bit signed numbers would be encoded as follows:

| Dec       | Hex\*     | Zigzag (hex)      | `varint` Encoded      | Length    |
| ---:      | :---      | :---              | :---                  | :---      |
| 0         | `0x00_00` | `0x00_00`         | `[0x00]`              | 1         |
| -1        | `0xFF_FF` | `0x00_01`         | `[0x01]`              | 1         |
| 1         | `0x00_01` | `0x00_02`         | `[0x02]`              | 1         |
| 63        | `0x00_3F` | `0x00_7E`         | `[0x7E]`              | 1         |
| -64       | `0xFF_C0` | `0x00_7F`         | `[0x7F]`              | 1         |
| 64        | `0x00_40` | `0x00_80`         | `[0x80, 0x01]`        | 2         |
| -65       | `0xFF_BF` | `0x00_81`         | `[0x81, 0x01]`        | 2         |
| 32767     | `0x7F_FF` | `0xFF_FE`         | `[0xFF, 0xFF, 0x02]`  | 3         |
| -32768    | `0x80_00` | `0xFF_FF`         | `[0xFF, 0xFF, 0x03]`  | 3         |

`*`: This column is represented as a sixteen bit, two's compliment form

### Maximum Encoded Length

As the values that an integer type (e.g. `u16`, `u32`) are limited to the expressible range of the type,
the maximum encoded length of these types are knowable ahead of time. Postcard uses this information to
limit the number of bytes it will process when decoding a `varint`.

As `varint`s encode seven data bits for every encoded byte, the maximum encoded length can be stated
as follows:

```
bits_per_byte = 8
enc_bits_per_byte = 7
encoded_max = ceil((len_bytes * bits_per_byte) / enc_bits_per_byte)
```

The following table expresses the maximum encoded length for each type:

| Type      | Varint Type       | Type length (bytes)   | Varint length max (bytes) |
| :---      | :---              | :---                  | :---                      |
| `u16`     | `varint(u16)`     | 2                     | 3                         |
| `i16`     | `varint(i16)`     | 2                     | 3                         |
| `u32`     | `varint(u32)`     | 4                     | 5                         |
| `i32`     | `varint(i32)`     | 4                     | 5                         |
| `u64`     | `varint(u64)`     | 8                     | 10                        |
| `i64`     | `varint(i64)`     | 8                     | 10                        |
| `u128`    | `varint(u128)`    | 16                    | 19                        |
| `i128`    | `varint(i128)`    | 16                    | 19                        |

### Canonicalization

The postcard wire format does NOT enforce [canonicalization], however values are still required to fit within the [Maximum Encoded Length] of the data type, and to contain no data that exceeds the maximum value of the integer type.

[Maximum Encoded Length]: #maximum-encoded-length

In this context, an encoded form would be considered canonical if it is encoded with no excess encoding bytes necessary to encode the value, and with the excess encoding bits all containing `0`s.

For example in the following `u16` encoded data:

| Value (`u16`) | Encoded Form                  | Canonical?    | Accepted? |
| :---          | :---                          | :---          | :---      |
| 0             | `[0x00]`                      | Yes           | Yes       |
| 0             | `[0x80, 0x00]`                | No\*          | Yes       |
| 0             | `[0x80, 0x80, 0x00]`          | No\*          | Yes       |
| 0             | `[0x80, 0x80, 0x80, 0x00]`    | No\*          | No\*\*    |
| 65535         | `[0xFF, 0xFF, 0x03]`          | Yes           | Yes       |
| 131071        | `[0xFF, 0xFF, 0x07]`          | No\*\*\*      | No\*\*\*  |
| 65535         | `[0xFF, 0xFF, 0x83, 0x00]`    | No\*          | No\*\*    |

* \*: Contains excess encoding bytes
* \*\*: Exceeds the [Maximum Encoded Length] of the type
* \*\*\*: Exceeds the maximum value of the encoded type

[canonicalization]: https://en.wikipedia.org/wiki/Canonicalization

## `isize` and `usize`

The Serde Data Model does not address platform-specific sized integers, and instead supports them by mapping to an integer type matching
the platform's bit width.

For example, on a platform with 32-bit pointers, `usize` will map to `u32`, and `isize` will map to `i32`. On a platform with 64-bit pointers, `usize` will map to `u64`, and `isize` will map to `i64`.

As these types are all `varint` encoded on the wire, two platforms of dissimilar pointer-widths will be able to interoperate without compatibility problems, as long as the value encoded in these types do not exceed the maximum encodable value of the smaller platform. If this occurs, for example sending `0x1_0000_0000usize` from a 64-bit target (as a `u64`), when decoding on a 32-bit platform, the value will fail to decode, as it exceeds the maximum value of a `usize` (as a `u32`).

## Variable Quantities

Several Serde Data Model types, such as `seq` and `string` contain a variable quantity of data elements.

Variable quantities are prefixed by a `varint(usize)`, encoding the count of subsequent data elements, followed by the encoded data elements.

## Tagged Unions

Tagged unions consist of two parts: The tag, or discriminant, and the value matching with that discriminant.

Tagged unions in postcard are encoded as a `varint(u32)` containing the discriminant, followed by the encoded value matching that discriminant.

[Tagged Union]: #tagged-unions

## Serde Data Model Types

The following describes how each of the Serde Data Model types are encoded in the Postcard Wire Format.

### 1 - `bool`

A `bool` is stored as a single byte, with the value of `0x00` for `false`, and `0x01` as `true`.

All other values are considered an error.

### 2 - `i8`

An `i8` is stored as a single byte, in two's compliment form.

All values are considered valid.

### 3 - `i16`

An `i16` is stored as a `varint(i16)`.

### 4 - `i32`

An `i32` is stored as a `varint(i32)`.

### 5 - `i64`

An `i64` is stored as a `varint(i64)`.

### 6 - `i128`

An `i128` is stored as a `varint(i128)`.

### 7 - `u8`

An `u8` is stored as a single byte.

All values are considered valid.

### 8 - `u16`

A `u16` is stored as a `varint(u16)`.

### 9 - `u32`

A `u32` is stored as a `varint(u32)`.

### 10 - `u64`

A `u64` is stored as a `varint(u64)`.

### 11 - `u128`

A `u128` is stored as a `varint(u128)`.

### 12 - `f32`

An `f32` will be bitwise converted into a `u32`, and encoded as a little-endian array of four bytes.

For example, the float value `-32.005859375f32` would be bitwise represented as `0xc200_0600u32`, and encoded as `[0x00, 0x06, 0x00, 0xc2]`.

> NOTE: `f32` values are NOT converted to `varint` form, and are always encoded as four bytes on the wire.

### 13 - `f64`

An `f64` will be bitwise converted into a `u64`, and encoded as a little-endian array of eight bytes.

For example, the float value `-32.005859375f64` would be bitwise represented as `0xc040_00c0_0000_0000u64`, and encoded as `[0x00, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x40, 0xc0]`.

> NOTE: `f64` values are NOT converted to `varint` form, and are always encoded as eight bytes on the wire.

### 14 - `char`

A `char` will be encoded in UTF-8 form, and encoded as a `string`.

### 15 - `string`

A `string` is encoded with a `varint(usize)` containing the length, followed by the array of bytes, each encoded as a single `u8`.

### 16 - `byte array`

A `byte array` is encoded with a `varint(usize)` containing the length, followed by the array of bytes, each encoded as a single `u8`.

### 17 - `option`

An `option` is encoded in one of two ways, depending in its value.

If an option has the value of `None`, it is encoded as the single byte `0x00`, with no following data.

If an option has the value of `Some`, it is encoded as the single byte `0x01`, followed by exactly one encoded Serde Data Type.

### 18 - `unit`

The `unit` type is NOT encoded to the wire, meaning that it occupies zero bytes.

### 19 - `unit_struct`

The `unit_struct` type is NOT encoded to the wire, meaning that it occupies zero bytes.

### 20 - `unit_variant`

A `unit_variant` is an instance of a [Tagged Union], consisting of a `varint(u32)` discriminant, with no additional encoded data.

### 21 - `newtype_struct`

A `newtype_struct` is encoded as the Serde Data Type it contains, with no additional data preceding or following it.

### 22 - `newtype_variant`

A `newtype_variant` is an instance of a [Tagged Union], consisting of a `varint(u32)` discriminant, followed by the encoded representation of the Serde Data Type it contains.

### 23 - `seq`

A `seq` is encoded with a `varint(usize)` containing the number of elements of the `seq`, followed by the array of elements, each encoded as an individual Serde Data Type.

### 24 - `tuple`

A `tuple` is encoded as the elements that comprise it, in their order of definition (left to right).

As `tuple`s have a known size, their length is not encoded on the wire.

### 25 - `tuple_struct`

A `tuple_struct` is encoded as a `tuple` consisting of the elements contained by the `tuple_struct`.

### 26 - `tuple_variant`

A `tuple_variant` is an instance of a [Tagged Union], consisting of a `varint(u32)` discriminant, followed by a `tuple` consisting of the elements contained by the `tuple_variant`.

### 27 - `map`

A `map` is encoded with a `varint(usize)` containing the number of (key, value) elements of the `map`, followed by the array of (key, value) pairs, each encoded as a `tuple` of `(key, value)`.

### 28 - `struct`

A `struct` is encoded as the elements that comprise it, in their order of definition (top to bottom).

As `struct`s have a known number of elements with known names, their length and field names are not encoded on the wire.

### 29 - `struct_variant`

A `struct_variant` is an instance of a [Tagged Union], consisting of a `varint(u32)` discriminant, followed by a `struct` consisting of the elements contained by the `struct_variant`.
