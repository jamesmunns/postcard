# The Serde Binding

This chapter describes how each type of the [Serde Data Model] is encoded.
`serde` calls encoding *serialization*, and decoding *deserialization*.

With `serde`, a type's `Serialize` and `Deserialize` implementations are its
[Schema]. For requirements checked against a schema, such as rejecting an
unknown [discriminant], a [decoder] is a serde `Deserializer` together with the
`Deserialize` implementation of the type being decoded.

## Tagged Unions

A Rust `enum` is a [Tagged Union]. In the Serde Data Model, a [variant] is
identified by its `variant_index`, which `serde_derive` sets to the variant's
position within the `enum`, starting at zero. Rust's own discriminant values,
such as `A = 5`, and the `#[repr]` of an `enum`, are not used.

postcard[serde.variant.discriminant]
The [Discriminant] of a serde variant MUST be its `variant_index`.

## Serde Data Model Types

Each section below quotes the [Serde Data Model]'s description of one or more of
its types, followed by the postcard type each is encoded as.

### Primitive Types

> - **14 primitive types**
>   - bool
>   - i8, i16, i32, i64, i128
>   - u8, u16, u32, u64, u128
>   - f32, f64
>   - char

postcard[serde.type.bool]
A serde `bool` MUST be encoded as a postcard [`bool`].

postcard[serde.type.i8]
A serde `i8` MUST be encoded as a postcard [`i8`].

postcard[serde.type.i16]
A serde `i16` MUST be encoded as a postcard [`i16`].

postcard[serde.type.i32]
A serde `i32` MUST be encoded as a postcard [`i32`].

postcard[serde.type.i64]
A serde `i64` MUST be encoded as a postcard [`i64`].

postcard[serde.type.i128]
A serde `i128` MUST be encoded as a postcard [`i128`].

postcard[serde.type.u8]
A serde `u8` MUST be encoded as a postcard [`u8`].

postcard[serde.type.u16]
A serde `u16` MUST be encoded as a postcard [`u16`].

postcard[serde.type.u32]
A serde `u32` MUST be encoded as a postcard [`u32`].

postcard[serde.type.u64]
A serde `u64` MUST be encoded as a postcard [`u64`].

postcard[serde.type.u128]
A serde `u128` MUST be encoded as a postcard [`u128`].

postcard[serde.type.f32]
A serde `f32` MUST be encoded as a postcard [`f32`].

postcard[serde.type.f64]
A serde `f64` MUST be encoded as a postcard [`f64`].

postcard[serde.type.char]
A serde `char` MUST be encoded as a postcard [`char`].

serde has no `usize` or `isize` type, and serializes the Rust `usize` and
`isize` types as `u64` and `i64`. The serde binding therefore does not use the
`usize` and `isize` [atoms][atom]: a Rust `usize` follows the requirements for
`u64`, including its [Maximum Encoded Length], on every platform. [Length]s are
still encoded as `varint(usize)`.

### `string`

> - **string**
>   - UTF-8 bytes with a length and no null terminator. May contain 0-bytes.
>   - When serializing, all strings are handled equally. When deserializing,
>     there are three flavors of strings: transient, owned, and borrowed. This
>     distinction is explained in [Understanding deserializer lifetimes] and is
>     a key way that Serde enabled efficient zero-copy deserialization.

postcard[serde.type.string]
A serde `string` MUST be encoded as a postcard [`string`].

All three flavors of a serde `string` are decoded from the same postcard
encoding.

### `byte array`

> - **byte array** - [u8]
>   - Similar to strings, during deserialization byte arrays can be transient,
>     owned, or borrowed.

postcard[serde.type.byte-array]
A serde `byte array` MUST be encoded as a postcard [`bytes`] value.

All three flavors of a serde `byte array` are decoded from the same postcard
encoding.

### `option`

> - **option**
>   - Either none or some value.

postcard[serde.type.option]
A serde `option` MUST be encoded as an [Option].

### `unit`

> - **unit**
>   - The type of `()` in Rust. It represents an anonymous value containing no
>     data.

postcard[serde.type.unit]
A serde `unit` MUST be encoded as a postcard [`unit`].

### `unit_struct`

> - **unit_struct**
>   - For example `struct Unit` or `PhantomData<T>`. It represents a named value
>     containing no data.

postcard[serde.type.unit-struct]
A serde `unit_struct` MUST be encoded as a postcard [`unit`].

### `unit_variant`

> - **unit_variant**
>   - For example the `E::A` and `E::B` in `enum E { A, B }`.

postcard[serde.type.unit-variant]
A serde `unit_variant` MUST be encoded as a [Tagged Union] whose variant's value
is a postcard [`unit`].

### `newtype_struct`

> - **newtype_struct**
>   - For example `struct Millimeters(u8)`.

postcard[serde.type.newtype-struct]
A serde `newtype_struct` MUST be encoded as the value it contains.

### `newtype_variant`

> - **newtype_variant**
>   - For example the `E::N` in `enum E { N(u8) }`.

postcard[serde.type.newtype-variant]
A serde `newtype_variant` MUST be encoded as a [Tagged Union] whose variant's
value is the value it contains.

### `seq`

> - **seq**
>   - A variably sized heterogeneous sequence of values, for example `Vec<T>` or
>     `HashSet<T>`. When serializing, the length may or may not be known before
>     iterating through all the data. When deserializing, the length is
>     determined by looking at the serialized data. Note that a homogeneous Rust
>     collection like `vec![Value::Bool(true), Value::Char('c')]` may serialize
>     as a heterogeneous Serde seq, in this case containing a Serde bool
>     followed by a Serde char.

postcard[serde.type.seq]
A serde `seq` MUST be encoded as a [Sequence] of its [elements][element].

postcard[serde.encode.length-required]
When a `seq` or `map` is serialized without a known length, a serializer MUST
either report an error, or buffer its elements until its length is known.

Its [Length] is encoded before its elements, so it cannot be encoded until the
length is known.

### `tuple`

> - **tuple**
>   - A statically sized heterogeneous sequence of values for which the length
>     will be known at deserialization time without looking at the serialized
>     data, for example `(u8,)` or `(String, u64, Vec<T>)` or `[u64; 10]`.

postcard[serde.type.tuple]
A serde `tuple` MUST be encoded as an [Aggregate] of its elements.

### `tuple_struct`

> - **tuple_struct**
>   - A named tuple, for example `struct Rgb(u8, u8, u8)`.

postcard[serde.type.tuple-struct]
A serde `tuple_struct` MUST be encoded as an [Aggregate] of its elements.

### `tuple_variant`

> - **tuple_variant**
>   - For example the `E::T` in `enum E { T(u8, u8) }`.

postcard[serde.type.tuple-variant]
A serde `tuple_variant` MUST be encoded as a [Tagged Union] whose variant's
value is an [Aggregate] of its elements.

### `map`

> - **map**
>   - A variably sized heterogeneous key-value pairing, for example
>     `BTreeMap<K, V>`. When serializing, the length may or may not be known
>     before iterating through all the entries. When deserializing, the length
>     is determined by looking at the serialized data.

postcard[serde.type.map]
A serde `map` MUST be encoded as a [Sequence] of `(key, value)` [Aggregate]s.

This specification does not require the keys of a `map` to be unique or
ordered, and the type being decoded determines what happens to a repeated key.
For example, serde's `HashMap` and `BTreeMap` implementations keep the last
value for a repeated key, and `BTreeMap` serializes its pairs in key order.

### `struct`

> - **struct**
>   - A statically sized heterogeneous key-value pairing in which the keys are
>     compile-time constant strings and will be known at deserialization time
>     without looking at the serialized data, for example
>     `struct S { r: u8, g: u8, b: u8 }`.

postcard[serde.type.struct]
A serde `struct` MUST be encoded as an [Aggregate] of its [fields][field].

postcard[serde.decode.struct.seq]
A deserializer MUST present a serde `struct` or `struct_variant` to its visitor
as a sequence of its fields, in order, by calling `visit_seq`.

### `struct_variant`

> - **struct_variant**
>   - For example the `E::S` in `enum E { S { r: u8, g: u8, b: u8 } }`.

postcard[serde.type.struct-variant]
A serde `struct_variant` MUST be encoded as a [Tagged Union] whose variant's
value is an [Aggregate] of its fields.

## Human-Readable Formats

serde lets a type use a different representation in human-readable formats,
such as a string for an IP address.

postcard[serde.human-readable]
A serializer and a deserializer MUST return `false` from `is_human_readable`.

## Equivalent Encodings

Several serde types are encoded as the same postcard type, and so have the same
encoding. For example, a `tuple`, a `tuple_struct`, and a `struct` with the same
elements are each an [Aggregate], and a `map` and a `seq` of `(key, value)`
`tuple`s are each a [Sequence] of aggregates. See [Wire Compatibility] for the
postcard types that share an encoding.

## Serde Features That Need a Self-Describing Format

Postcard does not support serde features that use `deserialize_any`. See
[Serde Attributes] for the attributes that use it, and others that cause issues.

As postcard is not self-describing, deserializers do not support the following
hints:

postcard[serde.decode.any.unsupported]
A deserializer MUST report an error when asked to `deserialize_any`.

postcard[serde.decode.identifier.unsupported]
A deserializer MUST report an error when asked to `deserialize_identifier`.

postcard[serde.decode.ignored-any.unsupported]
A deserializer MUST report an error when asked to `deserialize_ignored_any`.

## Serde Attributes

This section is informative, and contains no requirements. The attributes were
checked with serde_derive 1.0.229, apart from those that only change the
generated code.

### Attributes Believed to Cause Issues With Postcard

The following attributes can make encoding and decoding disagree, or cause an
error.

#### `#[serde(tag = "...")]` on an enum, and `#[serde(untagged)]` on an enum or a variant

Decoding uses `deserialize_any`, which reports an error.

#### `#[serde(tag = "...")]` on a struct

The tag is encoded as an extra `string` before the fields, but not expected when
decoding, so the fields decode as the wrong values.

#### `#[serde(tag = "...", content = "...")]`, `#[serde(field_identifier)]`, and `#[serde(variant_identifier)]`

Decoding uses `deserialize_identifier`, which reports an error.

#### `#[serde(other)]`

On an enum without `tag`, a discriminant that the decoding type does not define
decodes as the `other` variant. The value of the unknown variant is not decoded,
so the values that follow it decode as the wrong values. On an enum with `tag`,
or with `tag` and `content`, decoding reports an error, as above.

#### `#[serde(flatten)]`

Encoding serializes a `map` with an unknown length, which reports an error.

#### `#[serde(skip_serializing)]` on a field, and `#[serde(skip_serializing_if = "...")]`

The field is not encoded, but is decoded, so the [message] fails to decode, or
decodes as the wrong value using the [bytes][byte] that follow it. For
`skip_serializing_if`, this happens whenever the condition holds.

#### `#[serde(skip_deserializing)]` on a field

The field is encoded, but not decoded, so the fields after it decode as the
wrong values.

#### `#[serde(skip)]` or `#[serde(skip_deserializing)]` on a variant

An [encoder] uses each variant's position among all of the variants, while a
decoder leaves out the skipped variant. A variant after a skipped one fails to
decode, or decodes as the wrong variant.

### Attributes Believed Not to Cause Issues With Postcard

The following attributes are believed not to cause errors, or a mismatch
between encoding and decoding:

- `serde(transparent)`
- `serde(from = "...")`, `serde(try_from = "...")`, `serde(into = "...")`
- `serde(deny_unknown_fields)`
- `serde(default)`, `serde(default = "...")`
- `serde(rename = "...")`, `serde(rename_all = "...")`,
  `serde(rename_all_fields = "...")`
- `serde(alias = "...")`
- `serde(skip)`, on a field
- `serde(skip_serializing)`, on a variant
- `serde(with = "...")`, `serde(serialize_with = "...")`,
  `serde(deserialize_with = "...")`, when they encode and decode the same type
- `serde(bound = "...")`, `serde(crate = "...")`, `serde(expecting = "...")`,
  `serde(remote = "...")`, `serde(getter = "...")`, and `serde(borrow)`, which
  only change the generated code

[Schema]: #schemas
[Wire Compatibility]: #implementation-considerations--interoperability--schema-evolution--wire-compatibility
[Serde Data Model]: https://serde.rs/data-model.html
[Tagged Union]: #composites--tagged-union
[Aggregate]: #composites--aggregate
[Sequence]: #composites--sequence
[`unit`]: #atoms--fixed-size-atoms--unit
[`string`]: #atoms--length-prefixed-atoms--string
[Discriminant]: #atoms--headers--discriminant
[`bool`]: #atoms--fixed-size-atoms--bool
[`i8`]: #atoms--fixed-size-atoms--i8
[`i16`]: #atoms--varint-encoded-integers
[`i32`]: #atoms--varint-encoded-integers
[`i64`]: #atoms--varint-encoded-integers
[`i128`]: #atoms--varint-encoded-integers
[`u8`]: #atoms--fixed-size-atoms--u8
[`u16`]: #atoms--varint-encoded-integers
[`u32`]: #atoms--varint-encoded-integers
[`u64`]: #atoms--varint-encoded-integers
[`u128`]: #atoms--varint-encoded-integers
[`f32`]: #atoms--fixed-size-atoms--f32
[`f64`]: #atoms--fixed-size-atoms--f64
[`char`]: #atoms--length-prefixed-atoms--char
[`bytes`]: #atoms--length-prefixed-atoms--bytes
[Option]: #composites--option
[Length]: #atoms--headers--length
[Understanding deserializer lifetimes]: https://serde.rs/lifetimes.html
[Maximum Encoded Length]: #atoms--varint-encoded-integers--maximum-encoded-length
[Serde Attributes]: #the-serde-binding--serde-attributes
[decoder]: #glossary--decoder
[variant]: #glossary--variant
[atom]: #glossary--atom
[element]: #glossary--element
[field]: #glossary--field
[message]: #glossary--message
[byte]: #glossary--byte
[encoder]: #glossary--encoder
[Composites]: #composites
[Schemas]: #schemas
