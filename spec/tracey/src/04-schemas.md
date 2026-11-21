# Schemas

A schema is the [type] used by the [encoder] and [decoder] of a [message], along
with names for its types, [fields][field], and [variants][variant]. Names are
never encoded.

The requirements in this chapter are checked against a specific schema, rather
than against the [bytes][byte] alone ([Atoms]) or the shape of a [composite]
([Composites]).

## Types

postcard[schema.decode.types]
A decoder MUST decode each [value] as the type that the schema defines for it.

## Aggregates

postcard[schema.aggregate.decode.count]
A decoder MUST decode exactly as many [elements][element] of an [Aggregate] as
the schema defines.

## Tagged Unions

A schema defines a distinct [discriminant] for each variant of a [tagged union].

postcard[schema.tagged-union.decode.unknown-discriminant]
A decoder MUST reject a [Tagged Union] whose [Discriminant] does not identify a
variant that the schema defines.

## Value Constraints

A type may constrain acceptable values further, such as a non-zero integer, or a
collection with a maximum capacity. This specification does not define these
constraints. A value that does not meet them is reported as an error, either
while it is decoded or after.

## Resource Limits

A decoder may also limit the resources it uses to decode a message, such as the
number of elements it decodes, or how deeply composites nest (see [Security]).
A message that exceeds such a limit is reported as an error, in the same way as
a value that does not meet its type's constraints, even though a conforming
encoder can produce it. These limits are chosen by each decoder, and are not
defined by this specification.

[type]: #glossary--type
[Atoms]: #atoms
[Composites]: #composites
[Aggregate]: #composites--aggregate
[Tagged Union]: #composites--tagged-union
[Discriminant]: #atoms--headers--discriminant
[message]: #glossary--message
[decoder]: #glossary--decoder
[encoder]: #glossary--encoder
[variant]: #glossary--variant
[field]: #glossary--field
[composite]: #glossary--composite
[byte]: #glossary--byte
[value]: #glossary--value
[element]: #glossary--element
[The Serde Binding]: #the-serde-binding
[Security]: #implementation-considerations--security
