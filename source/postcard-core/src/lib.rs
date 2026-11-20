//! # Postcard Core
//!
//! `postcard-core` contains the minimal logic necessary for encoding data to and decoding data from
//! the postcard wire format. It is not coupled to any specific serialization or deserialization
//! framework, and is intended to be usable when writing a custom postcard serializer or deserializer.
//!
//! For an example of usage of this crate, see the [`postcard2`] crate, which uses `postcard-core`
//! to implement a [`serde`] compatible serializer/deserializer.
//!
//! `postcard-core` implements the atoms of the [Postcard Wire Specification]: the types
//! whose encoding the specification fully defines.
//!
//! Encoders/decoders for composites, such as [`postcard2`], build on top of
//! these functions. Each function refers to the requirements it implements with
//! a [tracey] identifier, such as `postcard[atom.varint.encoding.zigzag]`.
//!
//! ## Atoms
//!
//! The following table shows the mapping of an atom to the encoding function,
//! decoding function, and [tracey] requirement identifier.
//!
//! | Atom     | Encode                    | Decode                   | Requirements                                        |
//! | :---     | :---                      | :---                     | :---                                                |
//! | `unit`   | [`ser::try_push_unit()`]  | [`de::try_take_unit()`]  | `postcard[atom.unit.*]`                             |
//! | `bool`   | [`ser::try_push_bool()`]  | [`de::try_take_bool()`]  | `postcard[atom.bool.*]`                             |
//! | `u8`     | [`ser::try_push_u8()`]    | [`de::try_take_u8()`]    | `postcard[atom.u8.*]`                               |
//! | `u16`    | [`ser::try_push_u16()`]   | [`de::try_take_u16()`]   | `postcard[atom.varint.*]`                           |
//! | `u32`    | [`ser::try_push_u32()`]   | [`de::try_take_u32()`]   | `postcard[atom.varint.*]`                           |
//! | `u64`    | [`ser::try_push_u64()`]   | [`de::try_take_u64()`]   | `postcard[atom.varint.*]`                           |
//! | `u128`   | [`ser::try_push_u128()`]  | [`de::try_take_u128()`]  | `postcard[atom.varint.*]`                           |
//! | `usize`  | [`ser::try_push_usize()`] | [`de::try_take_usize()`] | `postcard[atom.varint.*]`, `postcard[atom.usize.*]` |
//! | `i8`     | [`ser::try_push_i8()`]    | [`de::try_take_i8()`]    | `postcard[atom.i8.*]`                               |
//! | `i16`    | [`ser::try_push_i16()`]   | [`de::try_take_i16()`]   | `postcard[atom.varint.*]`                           |
//! | `i32`    | [`ser::try_push_i32()`]   | [`de::try_take_i32()`]   | `postcard[atom.varint.*]`                           |
//! | `i64`    | [`ser::try_push_i64()`]   | [`de::try_take_i64()`]   | `postcard[atom.varint.*]`                           |
//! | `i128`   | [`ser::try_push_i128()`]  | [`de::try_take_i128()`]  | `postcard[atom.varint.*]`                           |
//! | `isize`  | [`ser::try_push_isize()`] | [`de::try_take_isize()`] | `postcard[atom.varint.*]`, `postcard[atom.isize.*]` |
//! | `f32`    | [`ser::try_push_f32()`]   | [`de::try_take_f32()`]   | `postcard[atom.f32.*]`                              |
//! | `f64`    | [`ser::try_push_f64()`]   | [`de::try_take_f64()`]   | `postcard[atom.f64.*]`                              |
//! | `bytes`  | [`ser::try_push_bytes()`] | [`de::try_take_bytes()`] | `postcard[atom.bytes.encoding]`                     |
//! | `string` | [`ser::try_push_str()`]   | [`de::try_take_str()`]   | `postcard[atom.string.*]`                           |
//! | `char`   | [`ser::try_push_char()`]  | [`de::try_take_char()`]  | `postcard[atom.char.*]`                             |
//!
//! `bytes` and `string` values can also be decoded with [`de::try_take_bytes_temp()`] and
//! [`de::try_take_str_temp()`], which yield borrowed data that is not required to live for the
//! entire lifetime of the Flavor's buffer. This is useful when the yielded data is immediately
//! going to be converted into owned data, e.g. a `Vec<u8>` or `String`.
//!
//! This is an implementation detail only: there is no difference in what is decoded, but it
//! may influence the Flavor interactions. See [`de::Flavor::try_take_n()`] and
//! [`de::Flavor::try_take_n_temp()`] for more details. The non-temp functions provide *borrowed*
//! views of the decoded data.
//!
//! ## Headers
//!
//! Headers are atoms whose value determines the number or type of the values that follow
//! them, in a composite or a `bytes` or `string` value.
//!
//! | Header        | Encode                                                           | Decode                            | Requirements                           |
//! | :---          | :---                                                             | :---                              | :---                                   |
//! | Length        | [`ser::try_push_length()`]                                       | [`de::try_take_length()`]         | `postcard[atom.length.encoding]`       |
//! | Discriminant  | [`ser::try_push_discriminant()`]                                 | [`de::try_take_discriminant()`]   | `postcard[atom.discriminant.encoding]` |
//! | Option `bool` | [`ser::try_push_option_none()`], [`ser::try_push_option_some()`] | [`de::try_take_option_discrim()`] | `postcard[atom.bool.*]`                |
//!
//! ## Composites
//!
//! `postcard-core` does not encode composites itself, as it only defines the minimal building
//! blocks for encoding and decoding.
//!
//! An encoder for composites encodes headers with the functions below, followed by the values
//! they contain. Composites are defined by the `postcard[comp.*]` requirements of the specification.
//!
//! | Composite    | Encode                                                                             | Decode                                                      |
//! | :---         | :---                                                                               | :---                                                        |
//! | Aggregate    | Each element in schema order                                                       | Each element in schema order                                |
//! | Sequence     | [`ser::try_push_length()`], then each element                                      | [`de::try_take_length()`], then each element                |
//! | Tagged union | [`ser::try_push_discriminant()`], then the variant's value                         | [`de::try_take_discriminant()`], then the variant's value   |
//! | Option       | [`ser::try_push_option_none()`] or [`ser::try_push_option_some()`], then the value | [`de::try_take_option_discrim()`], then the value if `true` |
//!
//! ## Schema
//!
//! | Item     | Encode       | Decode       |
//! | :---     | :---         | :---         |
//! | [schema] | TODO: Schema | TODO: Schema |
//!
//! [`postcard2`]: https://github.com/jamesmunns/postcard/tree/main/source/postcard2
//! [`serde`]: https://serde.rs
//! [Postcard Wire Specification]: https://github.com/jamesmunns/postcard/tree/main/spec
//! [tracey]: https://github.com/bearcove/tracey
//! [schema]: #

#![cfg_attr(not(test), no_std)]
#![warn(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

// postcard[impl conformance.decode.accept]
pub mod de;
// postcard[impl conformance.encode.valid]
pub mod ser;
pub mod varint;
