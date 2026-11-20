# The Postcard Wire Specification

## Foreword

> If you think hard enough and long enough about serialization then you realize
> something remarkable: owwie ouch oof my thinking hurt
>
> David Koloski, author of [rkyv](https://rkyv.org/)

## Version and Stability

This document is version 1.1 of the Postcard Wire Specification, which describes
the postcard wire format.

Version 1.1 of the specification makes no changes to the postcard wire format
itself. This specification version splits the wire format definition into
layers, and clarifies and codifies behavior that was potentially unclear in
version 1.0 of the specification.

Version 1.0 of this specification remains available as an archive.

<!--
    TODO: Add link the archived 1.0 version once it is published,
    e.g. https://postcard.jamesmunns.com/v1.0/
-->

The postcard crate versions 1.x.y and postcard 2.x.y all implement the wire
format described by the 1.x specifications, and will each produce the same
[encoding] for the same [value].

The postcard wire format is stable. A change to the encoding of any value would
only be made in a new major version of this specification, and the postcard
crate would only adopt it in a new major version of the crate.

## Conventions

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this
document are to be interpreted as described in [BCP 14] ([RFC 2119],
[RFC 8174]) when, and only when, they appear in all capitals, as shown here.

The recommendations in [Implementation Considerations] also use these key words,
but are not tracked as requirements.

Terms such as [type], [value], [encoding], [message], [encoder], [decoder],
[atom], [composite], [element], [field], [variant], [header], and [schema] have
specific meanings in this specification, defined in the [Glossary].

Each requirement of this specification is labelled with a [tracey] identifier,
such as `postcard[atom.varint.encoding.zigzag]`. Implementations and tests refer
to requirements by these identifiers, in comments. The first part of each
identifier is the layer that the requirement belongs to: `conformance`, `atom`,
`comp` (composites), `schema`, or `serde`.

## Scope

This specification describes how a single message is encoded. It does not cover:

- Framing, such as separating messages in a stream with COBS.
- Integrity checks, such as a CRC or cryptographic digest appended to a message.
- Versioning of messages or schemas, or how schemas are exchanged.

Libraries such as `postcard` provide framing and integrity checks as API
features. They are not part of the specified wire format.

## Non Self-Describing Format

Postcard is not a "Self Describing Format": a message contains only the
encodings of its values, with no information about their types or names. The
encoder and decoder of a message are expected to use the same schema, often from
a shared Rust library of data types.

## Organization of This Specification

This specification describes the wire format in the following layers:

1. [Atoms] are types whose encoding is fully defined by this specification, such
   as integers, strings, and headers.
2. [Composites] are types that contain values of other types chosen by the user,
   such as [sequences][sequence], [aggregates][aggregate], and
   [tagged unions][tagged union].
3. [Schemas] are the types used by both the encoder and decoder of a message,
   and the requirements that are checked against those types.
4. Bindings describe how a library encodes its own types. [The Serde Binding]
   describes how each type of the [Serde Data Model] is encoded.
5. [Implementation Considerations] describes concerns that the wire format does
   not cover, but which matter to authors of libraries based on the postcard
   wire format.

Within the [postcard repository]:

1. The `postcard-core` crate encodes and decodes [Atoms].
2. The `postcard` crate implements [Composites] and the schema requirements, and
   uses `postcard-core` for [Atoms]. It also implements [The Serde Binding].
3. The `postcard-schema` crate describes the [Schema] of a Rust type as a value,
   which can be stored, sent, or compared at compile or runtime.
4. The `postcard-dyn` crate encodes and decodes messages using a schema known
   only at runtime, converting them to and from JSON values.

Each crate is checked only against the requirements of the layers it implements,
so neither crate's [tracey] coverage reaches 100% on its own. `postcard-core`
covers the conformance and atom requirements, and `postcard` covers the
composite, schema, and serde requirements. Together, they cover every
requirement. `postcard-schema` and `postcard-dyn` are not yet traced.

## Conformance

An [encoder] produces messages as a sequence of [bytes][byte], and a [decoder]
consumes them.

postcard[conformance.decode.reject]
When a decoder rejects a message, it MUST report an error, and MUST NOT produce
a value from it.

postcard[conformance.decode.truncated]
A decoder MUST reject a message if its input ends before the message is fully
decoded.

postcard[conformance.encode.valid]
A conforming encoder MUST only produce messages that meet all encoding
requirements of this specification.

postcard[conformance.decode.accept]
A conforming decoder MUST reject a message where this specification requires
it, and MUST accept every other message that a conforming encoder can produce
for the decoder's schema, unless the message exceeds one of the decoder's
[Resource Limits].

## References

| Document Name            | Full Name                                                        | Version            |
| :----------------------- | :--------------------------------------------------------------- | :----------------- |
| [Serde Book]             | The Serde Book                                                   | serde v1.0.229     |
| [Unicode 14.0 Chapter 3] | The Unicode® Standard Core Specification, Chapter 3: Conformance | v14.0              |
| [IEEE 754-2008]          | IEEE Standard for Floating-Point Arithmetic                      | 2008               |
| [BCP 14]                 | Key words for use in RFCs to Indicate Requirement Levels         | RFC 2119, RFC 8174 |

## License

The Postcard Wire Specification (this document) is licensed under the terms of
the [CC-BY-SA 4.0 license].

The full license text is in `spec/LICENSE-CC-BY-SA` in the
[postcard repository].

Portions of the [Serde Data Model] are included under the terms of the
[CC-BY-SA 4.0 license].

[Atoms]: #atoms
[BCP 14]: https://www.rfc-editor.org/info/bcp14
[CC-BY-SA 4.0 license]: https://creativecommons.org/licenses/by-sa/4.0/
[Composites]: #composites
[Glossary]: #glossary
[IEEE 754-2008]: https://standards.ieee.org/ieee/754/4211/
[Implementation Considerations]: #implementation-considerations
[RFC 2119]: https://www.rfc-editor.org/rfc/rfc2119
[RFC 8174]: https://www.rfc-editor.org/rfc/rfc8174
[Schema]: #schemas
[Schemas]: #schemas
[Resource Limits]: #schemas--resource-limits
[Serde Book]: https://serde.rs/
[Serde Data Model]: https://serde.rs/data-model.html
[The Serde Binding]: #the-serde-binding
[Unicode 14.0 Chapter 3]: https://www.unicode.org/versions/Unicode14.0.0/ch03.pdf
[decoder]: #glossary--decoder
[element]: #glossary--element
[encoder]: #glossary--encoder
[encoding]: #glossary--encoding
[field]: #glossary--field
[postcard repository]: https://github.com/jamesmunns/postcard
[tracey]: https://tracey.bearcove.eu/
[type]: #glossary--type
[value]: #glossary--value
[variant]: #glossary--variant
[message]: #glossary--message
[atom]: #glossary--atom
[composite]: #glossary--composite
[header]: #glossary--header
[byte]: #glossary--byte
