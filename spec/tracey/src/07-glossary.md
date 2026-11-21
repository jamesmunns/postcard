# Glossary

These words have specific meanings throughout this specification.

## Aggregate

A [Composite][composite] with a fixed number of [elements][element], each of
which can be of a different type, such as a struct or tuple. See
[Aggregate][aggregates].

## Atom

A [type][type] whose [encoding][encoding] is fully defined by this
specification. An atom contains no values of types defined by the user. See
[Atoms][atoms].

## Byte

An octet: eight bits.

## Composite

A type that contains values of other types chosen by the user: an aggregate, a
sequence, a tagged union, or an option. See [Composites][composites].

## Decoder

Something that decodes a message, and either produces a value or rejects the
message. In the [Serde Binding][serde binding], a decoder is a serde
`Deserializer` together with the `Deserialize` implementation of the type being
decoded.

## Discriminant

The [Header][header] of a tagged union: a `varint(u32)` that identifies one of
its [variants][variant]. See [Discriminant][discriminants].

## Element

A value contained in an aggregate or a sequence.

## Encoder

Something that produces the encoding of a value. In the
[Serde Binding][serde binding], a serde `Serializer` is an encoder.

## Encoding

The bytes that represent a value. A value that is *encoded as* another type has
the same encoding as the corresponding value of that type.

## Field

A named element, such as one in a `struct`.

## Header

An atom whose value determines the number or type of the values that follow it,
in a composite or a length-prefixed atom: a length, a discriminant, or the
`bool` of an option. See [Headers][headers].

## Length

The [Header][header] of a sequence or a length-prefixed atom: a
`varint(usize)` containing the number of elements, or bytes, that follow it. See
[Length][length header].

## Length-Prefixed Atom

An atom that contains a [Length][length], followed by that many bytes: a
`bytes` value, a `string`, or a `char`. See
[Length-Prefixed Atoms][length-prefixed atoms].

## Message

The encoding of a single value of a schema.

## Option

A [Composite][composite] that contains zero or one value of another type. See
[Option][options].

## Schema

The [type][type] used by the encoder and decoder of a message, along with
names for its types, fields, and variants. Names are never encoded. See
[Schemas][schemas].

## Sequence

A [Composite][composite] with a variable number of [elements][element]. See
[Sequence][sequences].

## Tagged Union

A [Composite][composite] with several [variants][variant]. Each of its
values contains a [Discriminant][discriminant] that identifies a variant, and
a value of that variant. Known in Rust as an `enum`. See
[Tagged Union][tagged unions].

## Type

A set of possible values, and how each value is encoded. Atoms, composites, and
schemas are all types. A requirement that refers to a type applies to every
value of that type.

## Value

A single instance of a type.

## Variant

One of the alternative types of a tagged union.

## `varint`

The variable-length encoding of an integer, written `varint(N)` for an integer
type `N`. See [`varint`-Encoded Integers][`varint`-encoded integers].

[composite]: #glossary--composite
[element]: #glossary--element
[aggregates]: #composites--aggregate
[type]: #glossary--type
[encoding]: #glossary--encoding
[atoms]: #atoms
[composites]: #composites
[serde binding]: #the-serde-binding
[header]: #glossary--header
[variant]: #glossary--variant
[discriminants]: #atoms--headers--discriminant
[headers]: #atoms--headers
[length header]: #atoms--headers--length
[length]: #atoms--headers--length
[length-prefixed atoms]: #atoms--length-prefixed-atoms
[options]: #composites--option
[schemas]: #schemas
[sequences]: #composites--sequence
[discriminant]: #atoms--headers--discriminant
[tagged unions]: #composites--tagged-union
[`varint`-encoded integers]: #atoms--varint-encoded-integers
