# The Postcard Schema Specification

## Version and Stability

This document is a draft of version 0.1 of the Postcard Schema Specification,
which describes how the `postcard-schema` crate identifies schemas. It is not
yet stable.

It builds on the [Postcard Wire Specification], which defines how messages are
encoded. This specification does not change the wire format.

## Conventions

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this
document are to be interpreted as described in [BCP 14] ([RFC 2119], [RFC 8174])
when, and only when, they appear in all capitals, as shown here.

Each requirement of this specification is labelled with a [tracey] identifier,
such as `schema[key.hash.fnv1a-128]`. The `postcard-schema` crate refers to
requirements by these identifiers, in comments.

Terms such as aggregate, sequence, tagged union, and `varint(N)` have the
meanings defined in the [Postcard Wire Specification].

## Scope

This version of the specification only covers [Keys]: the 8-byte hash of a list
of strings and schemas, chosen by an application; and [Schema Tables]: how
schemas are sent.

TODO: define the schema data model
TODO: define how a schema is encoded on the wire

## Schemas

A schema in `postcard-schema` describes a type in the terms of the
[Serde Data Model]: it records the serde calls that the type's `Serialize`
implementation makes.

Each kind of schema is encoded as a postcard type, as described by the serde
binding of the [Postcard Wire Specification]:

| Kind                                   | Encoded as                                          |
|---                                     |---                                                  |
| `Bool`, `I8` to `I128`, `U8` to `U128` | the atom of the same name                           |
| `F32`, `F64`, `Char`                   | the atom of the same name                           |
| `String { .. }`                        | `string`                                            |
| `ByteArray { .. }`                     | `bytes`                                             |
| `Unit`                                 | `unit`                                              |
| `Option(T)`                            | an option of `T`                                    |
| `Seq { element, .. }`                  | a sequence of `element`                             |
| `Tuple(T, ...)`                        | an aggregate of the `T`s                            |
| `Map { key, val, .. }`                 | a sequence of aggregates of `key` and `val`         |
| `Struct { name, data }`                | `data`, as below                                    |
| `Enum { name, variants }`              | a tagged union, with each variant's `data` as below |

The `data` of a struct or variant is one of:

| Data                   | Encoded as               |
|---                     |---                       |
| `Unit`                 | `unit`                   |
| `Newtype(T)`           | `T`                      |
| `Tuple(T, ...)`        | an aggregate of the `T`s |
| `Struct(name: T, ...)` | an aggregate of the `T`s |

Names (of structs, enums, fields, and variants) and metadata like maximum
lengths do not affect how the values themselves are encoded on the wire.

## References

| Document Name                 | Full Name                                                | Version            |
|---                            |---                                                       |---                 |
| [Postcard Wire Specification] | The Postcard Wire Specification                          | 1.1                |
| [Serde Data Model]            | The Serde Book: Serde data model                         | serde v1.0.229     |
| [FNV]                         | The FNV Non-Cryptographic Hash Algorithm                 | draft-eastlake-fnv |
| [MurmurHash3]                 | MurmurHash3, `fmix64`                                    |                    |
| [BCP 14]                      | Key words for use in RFCs to Indicate Requirement Levels | RFC 2119, RFC 8174 |

## License

The Postcard Schema Specification (this document) is licensed under the terms of
the [CC-BY-SA 4.0 license]. The full license text is in
`spec/tracey/LICENSE-CC-BY-SA` in the [postcard repository].

[Keys]: #keys
[Schema Tables]: #schema-tables
[Postcard Wire Specification]: https://postcard.rs/wire/1.1/spec.html
[Serde Data Model]: https://serde.rs/data-model.html
[FNV]: https://datatracker.ietf.org/doc/draft-eastlake-fnv/
[MurmurHash3]: https://github.com/aappleby/smhasher/blob/master/src/MurmurHash3.cpp
[BCP 14]: https://www.rfc-editor.org/info/bcp14
[RFC 2119]: https://www.rfc-editor.org/rfc/rfc2119
[RFC 8174]: https://www.rfc-editor.org/rfc/rfc8174
[CC-BY-SA 4.0 license]: https://creativecommons.org/licenses/by-sa/4.0/
[postcard repository]: https://github.com/jamesmunns/postcard
[tracey]: https://tracey.bearcove.eu/
