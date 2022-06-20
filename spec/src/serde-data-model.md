# The Serde Data Model

## Serde Data Model Types

The serde data model, as defined by the [Serde Book], contains 29 types, each referred to as "A Serde Data Type".

### 1 - `bool`

A type capable of expressing exclusively the values `true` or `false`.

### 2 - `i8`

A signed integer type, capable of expressing any value in the range of `-128..=127` (or `-(2^7)..=((2^7) - 1)`).

### 3 - `i16`

A signed integer type, capable of expressing any value in the range of `-32768..=32767` (or `-(2^15)..=((2^15) - 1)`).

### 4 - `i32`

A signed integer type, capable of expressing any value in the range of `-2147483648..=2147483647` (or `-(2^31)..=((2^31) - 1)`).

### 5 - `i64`

A signed integer type, capable of expressing any value in the range of `-9223372036854775808..=9223372036854775807` (or `-(2^63)..=((2^63) - 1)`).

### 6 - `i128`

A signed integer type, capable of expressing any value in the range of `-170141183460469231731687303715884105728..=170141183460469231731687303715884105727` (or `-(2^127)..=((2^127) - 1)`).

### 7 - `u8`

An unsigned integer type, capable of expressing any value in the range of `0..=255` (or `0..=((2^8) - 1)`).

### 8 - `u16`

An unsigned integer type, capable of expressing any value in the range of `0..=65535` (or `0..=((2^16) - 1)`).

### 9 - `u32`

An unsigned integer type, capable of expressing any value in the range of `0..=4294967295` (or `0..=((2^32) - 1)`).

### 10 - `u64`

An unsigned integer type, capable of expressing any value in the range of `0..=18446744073709551615` (or `0..=((2^64) - 1)`).

### 11 - `u128`

An unsigned integer type, capable of expressing any value in the range of `0..=340282366920938463463374607431768211456` (or `0..=((2^128) - 1)`).

### 12 - `f32`

A "binary32" type defined as defined in [IEEE 754-2008].

### 13 - `f64`

A "binary64" type defined as defined in [IEEE 754-2008].

### 14 - `char`

A four-byte type representing a [Unicode scalar value].

A Unicode scalar value is defined by [Unicode 14.0 Chapter 3] Section 9 - "Unicode Encoding Forms", Definition [D76](https://www.unicode.org/versions/latest/ch03.pdf#G7404):

> *Unicode scalar value*: Any Unicode code point except high-surrogate and low-surrogate code points.
>
> As a result of this definition, the set of Unicode scalar values consists of the ranges 0x0000_0000 to 0x0000_D7FF and 0x0000_E000 to 0x0010_FFFF inclusive.

[Unicode scalar value]: https://www.unicode.org/glossary/#unicode_scalar_value

### 15 - `string`

A type representing a variable quantity of bytes, which together represent a valid UTF-8 code point sequence,
as defined by [Unicode 14.0 Chapter 3] Section 9 - "Unicode Encoding Forms", Definition
[D92](https://www.unicode.org/versions/latest/ch03.pdf#G7404):

> *UTF-8 encoding form*: The Unicode encoding form that assigns each Unicode scalar
> value to an unsigned byte sequence of one to four bytes in length, as specified in
> Table 3-6 and Table 3-7.

This encoding form is stored using the "UTF-8 encoding scheme", as defined by [Unicode 14.0 Chapter 3]
Section 10 - "Unicode Encoding Schemes", Definition [D95](https://www.unicode.org/versions/latest/ch03.pdf#G28070):

> *UTF-8 encoding scheme*: The Unicode encoding scheme that serializes a UTF-8
> code unit sequence in exactly the same order as the code unit sequence itself.

### 16 - `byte array`

A type representing a variable quantity of bytes.

### 17 - `option`

A type representing zero or one Serde Data Type.

### 18 - `unit`

A type representing an anonymous value containing no data.

### 19 - `unit_struct`

A type representing a named value containing no data.

### 20 - `unit_variant`

A type representing a named, tagged union variant, containing no data.

### 21 - `newtype_struct`

A type representing a named value, containing exactly one anonymous Serde Data Type.

### 22 - `newtype_variant`

A type representing a named, tagged union variant, containing exactly one anonymous Serde Data Type.

### 23 - `seq`

A type representing a variable quantity of values of a single Serde Data Type, e.g. a "Homogeneous Array".

Values of each element of the `seq` may have differing values.

### 24 - `tuple`

A type representing a fixed quantity of values, each of any Serde Data Type, e.g. a "Heterogeneous Array".

Values of each element of the `tuple` may have differing values.

### 25 - `tuple_struct`

A type representing a named type specifcally containing exactly one `tuple` Serde Data Type

### 26 - `tuple_variant`

A type representing a named, tagged union variant, containing exactly one `tuple` Serde Data Type.

### 27 - `map`

A type representing a variable quantity of key-value pairs. All keys are values of a single Serde Data Type. All values are a values of a single Serde Data Type.

### 28 - `struct`

A type representing a fixed quantity of named values, each of any Serde Data Type.

Values of each element of the `tuple` may have differing values.

> NOTE: Similar to `tuple`s , `struct`s have a known number of members, however all members of a `struct` also have a known name.
>
> `struct`s are also similar to `map`s, in that each member has a name (as a key) and a value, however `struct`s always have a fixed number of members, and their names are always constant.

### 29 - `struct_variant`

A type representing a named, tagged union variant, containing exactly one `struct` Serde Data type

## Meta types

### Named and Anonymous types

The above discussion differentiates between "named types" and "anonymous types".

"named types" are used to describe types that are bound to a name within the data type they
are contained, such as a field of a `struct`.

"anonymous types" are used to describe types that are NOT bound to a name within the data type
they are contained, such as a single element of a `tuple`.


### `enum`s or Tagged Unions

In the Rust language, the `enum` type (also known as "tagged unions" in other languages) describes a type that has
a differing internal type based on a value known as a `discriminant`.

In the serde data model (as well as the rust language) `discriminants` are always of the type `u32`.

In the serde data model, the "internal type" of an `enum` can be one of any of the following:

* `unit_variant`
* `newtype_variant`
* `tuple_variant`
* `struct_variant`

## References

| Document Name             | Full Name                                                         | Version   |
| :---                      | :---                                                              | :---      |
| [Serde Book]              | The Serde Book                                                    | v1.0.137  |
| [Unicode 14.0 Chapter 3]  | The Unicode® Standard Core Specification, Chapter 3: Conformance  | v14.0     |
| [IEEE 754-2008]           | IEEE Standard for Floating-Point Arithmetic                       | 2008      |

[Serde Book]: https://serde.rs/
[Unicode 14.0 Chapter 3]: https://www.unicode.org/versions/Unicode14.0.0/ch03.pdf
[IEEE 754-2008]: https://standards.ieee.org/ieee/754/4211/
