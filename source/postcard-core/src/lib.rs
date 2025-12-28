//! # Postcard Core
//!
//! `postcard-core` contains the minimal logic necessary for encoding data to and decoding data from
//! the postcard wire format. It is not coupled to any specific serialization or deserialization
//! framework, and is intended to be usable when writing a custom postcard serializer or deserializer.
//!
//! For an example of usage of this crate, see the [`postcard2`] crate, which uses `postcard-core`
//! to implement a [`serde`] compatible serializer/deserializer.
//!
//! ## Primitive items
//!
//! The following are all "primitive" data types. They are directly serialized and deserialized.
//!
//! | #     | Type                  | Serializer                        | Deserializer                      |
//! | :---  | :---                  | :---                              | :---                              |
//! | 1     | [bool]                | [`ser::try_push_bool()`]          | [`de::try_take_bool()`]           |
//! | 2     | [i8]                  | [`ser::try_push_i8()`]            | [`de::try_take_i8()`]             |
//! | 3     | [i16]                 | [`ser::try_push_i16()`]           | [`de::try_take_i16()`]            |
//! | 4     | [i32]                 | [`ser::try_push_i32()`]           | [`de::try_take_i32()`]            |
//! | 5     | [i64]                 | [`ser::try_push_i64()`]           | [`de::try_take_i64()`]            |
//! | 6     | [i128]                | [`ser::try_push_i128()`]          | [`de::try_take_i128()`]           |
//! | 7     | [u8]                  | [`ser::try_push_u8()`]            | [`de::try_take_u8()`]             |
//! | 8     | [u16]                 | [`ser::try_push_u16()`]           | [`de::try_take_u16()`]            |
//! | 9     | [u32]                 | [`ser::try_push_u32()`]           | [`de::try_take_u32()`]            |
//! | 10    | [u64]                 | [`ser::try_push_u64()`]           | [`de::try_take_u64()`]            |
//! | 11    | [u128]                | [`ser::try_push_u128()`]          | [`de::try_take_u128()`]           |
//! | 12    | [f32]                 | [`ser::try_push_f32()`]           | [`de::try_take_f32()`]            |
//! | 13    | [f64]                 | [`ser::try_push_f64()`]           | [`de::try_take_f64()`]            |
//! | 14    | [char]                | [`ser::try_push_char()`]          | [`de::try_take_char()`]           |
//! | 30    | [isize]               | [`ser::try_push_isize()`]         | [`de::try_take_isize()`]          |
//! | 31    | [usize]               | [`ser::try_push_usize()`]         | [`de::try_take_usize()`]          |
//!
//! ## Enum Variants
//!
//! The following are all [`Tagged Union`] items. The listed serializer and deserializer allow for pushing or
//! taking ONLY the discriminant. For serialization, the "body" must then be written if necessary. For
//! deserialization, the discriminant must be used to determine if and how to take the remainder of the
//! tagged union item.
//!
//! | #     | Type                  | Serializer                        | Deserializer                      |
//! | :---  | :---                  | :---                              | :---                              |
//! | 17a   | [option] - `None`     | [`ser::try_push_option_none()`]   | [`de::try_take_option_discrim()`] |
//! | 17b   | [option] - `Some`     | [`ser::try_push_option_some()`]   | [`de::try_take_option_discrim()`] |
//! | 20    | [unit variant]        | [`ser::try_push_discriminant()`]  | [`de::try_take_discriminant()`]   |
//! | 22    | [newtype variant]     | [`ser::try_push_discriminant()`]  | [`de::try_take_discriminant()`]   |
//! | 26    | [tuple variant]       | [`ser::try_push_discriminant()`]  | [`de::try_take_discriminant()`]   |
//! | 29    | [struct variant]      | [`ser::try_push_discriminant()`]  | [`de::try_take_discriminant()`]   |
//!
//! ## Length prefixed items
//!
//! The following are all length-prefixed items. They begin with a `varint(usize)`, which determines the number
//! of items in the item. This many items must then be serialized or deserialized.
//!
//! For [string] and [byte array], "temp" variants are provided that yield borrowed items that are not required
//! to live for the entire lifetime of the Flavor's buffer. This is useful when then yielded item is immediately
//! going to be converted into an owned item, e.g. a `Vec<u8>` or `String`.
//!
//! This is an implementation detail only - there is no functional different in deserialization, but may
//! influence the Flavor interactions. See [`de::Flavor::try_take_n()`] and [`de::Flavor::try_take_n_temp()`]
//! for more details.
//!
//! [string] and [byte array] are also unique in that the provide *borrowed* views to the deserialized data.
//!
//! | #     | Type                  | Serializer                        | Deserializer                      |
//! | :---  | :---                  | :---                              | :---                              |
//! | 15a   | [string]              | [`ser::try_push_str()`]           | [`de::try_take_str()`]            |
//! | 15b   | [string] (temp)       | [`ser::try_push_str()`]           | [`de::try_take_str_temp()`]       |
//! | 16a   | [byte array]          | [`ser::try_push_bytes()`]         | [`de::try_take_bytes()`]          |
//! | 16b   | [byte array] (temp)   | [`ser::try_push_bytes()`]         | [`de::try_take_bytes_temp()`]     |
//! | 23    | [seq]                 | [`ser::try_push_length()`]        | [`de::try_take_length()`]         |
//! | 27    | [map]                 | [`ser::try_push_length()`]        | [`de::try_take_length()`]         |
//!
//! ## Omitted items
//!
//! The following items are not encoded to the wire, and therefore have no serializer and deserializers.
//!
//! | #     | Type                  | Serializer                        | Deserializer                      |
//! | :---  | :---                  | :---                              | :---                              |
//! | 18    | [unit]                | N/A                               | N/A                               |
//! | 19    | [unit struct]         | N/A                               | N/A                               |
//!
//! ## Unheadered Aggregates
//!
//! The following items contain any number of subitems, however the "container" item itself does not require
//! any data to be written to or read from the wire. This means that only their child items, if any, need to
//! be serialized or deserialized.
//!
//! | #     | Type                  | Serializer                        | Deserializer                      |
//! | :---  | :---                  | :---                              | :---                              |
//! | 21    | [newtype struct]      | N/A                               | N/A                               |
//! | 24    | [tuple]               | N/A                               | N/A                               |
//! | 25    | [tuple struct]        | N/A                               | N/A                               |
//! | 28    | [struct]              | N/A                               | N/A                               |
//!
//! ## Schema
//!
//! | #     | Type                  | Serializer                        | Deserializer                      |
//! | :---  | :---                  | :---                              | :---                              |
//! | 32    | [schema]              | TODO: Schema                      | TODO: Schema                      |
//!
//! [`postcard2`]: https://github.com/jamesmunns/postcard/tree/main/source/postcard2
//! [`serde`]: https://serde.rs
//! [bool]: https://postcard.jamesmunns.com/wire-format#1---bool
//! [i8]: https://postcard.jamesmunns.com/wire-format#2---i8
//! [i16]: https://postcard.jamesmunns.com/wire-format#3---i16
//! [i32]: https://postcard.jamesmunns.com/wire-format#4---i32
//! [i64]: https://postcard.jamesmunns.com/wire-format#5---i64
//! [i128]: https://postcard.jamesmunns.com/wire-format#6---i128
//! [isize]: https://postcard.jamesmunns.com/wire-format#isize-and-usize
//! [u8]: https://postcard.jamesmunns.com/wire-format#7---u8
//! [u16]: https://postcard.jamesmunns.com/wire-format#8---u16
//! [u32]: https://postcard.jamesmunns.com/wire-format#9---u32
//! [u64]: https://postcard.jamesmunns.com/wire-format#10---u64
//! [u128]: https://postcard.jamesmunns.com/wire-format#11---u128
//! [usize]: https://postcard.jamesmunns.com/wire-format#isize-and-usize
//! [f32]: https://postcard.jamesmunns.com/wire-format#12---f32
//! [f64]: https://postcard.jamesmunns.com/wire-format#13---f64
//! [char]: https://postcard.jamesmunns.com/wire-format#14---char
//! [string]: https://postcard.jamesmunns.com/wire-format#15---string
//! [byte array]: https://postcard.jamesmunns.com/wire-format#16---byte-array
//! [option]: https://postcard.jamesmunns.com/wire-format#17---option
//! [unit]: https://postcard.jamesmunns.com/wire-format#18---unit
//! [unit struct]: https://postcard.jamesmunns.com/wire-format#19---unit_struct
//! [unit variant]: https://postcard.jamesmunns.com/wire-format#20---unit_variant
//! [newtype struct]: https://postcard.jamesmunns.com/wire-format#21---newtype_struct
//! [newtype variant]: https://postcard.jamesmunns.com/wire-format#22---newtype_variant
//! [seq]: https://postcard.jamesmunns.com/wire-format#23---seq
//! [tuple]: https://postcard.jamesmunns.com/wire-format#24---tuple
//! [tuple struct]: https://postcard.jamesmunns.com/wire-format#25---tuple_struct
//! [tuple variant]: https://postcard.jamesmunns.com/wire-format#26---tuple_variant
//! [map]: https://postcard.jamesmunns.com/wire-format#27---map
//! [struct]: https://postcard.jamesmunns.com/wire-format#28---struct
//! [struct variant]: https://postcard.jamesmunns.com/wire-format#29---struct_variant
//! [schema]: #
//! [`Tagged Union`]: https://postcard.jamesmunns.com/wire-format#tagged-unions

#![cfg_attr(not(test), no_std)]

pub mod de;
pub mod ser;
pub mod varint;
