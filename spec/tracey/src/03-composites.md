# Composites

A composite is a [type] that contains [values] of other types chosen by the
user. For example, an [aggregate] containing a bool, a u8, and an f32.

A composite is encoded as its [Header][headers], if it has one, followed by the
values it contains.

## Aggregate

An aggregate is a type with a fixed number of [elements], each of which can be
of a different type.

postcard[comp.aggregate.encoding]
An aggregate MUST be encoded as its elements, in their order of definition.

As a result, the number of elements in an aggregate is not encoded, and an
aggregate with no elements is encoded as zero [bytes][byte].

## Sequence

A sequence is a type with a variable number of elements.

postcard[comp.sequence.encoding]
A sequence MUST be encoded as a [Length], containing the number of elements,
followed by each element in order.

postcard[comp.sequence.decode.count]
A [decoder] MUST decode exactly as many elements as the sequence's [Length]
contains.

See [Lengths] and [Nesting] for limiting the length of a sequence or the nesting
depth of composites.

## Tagged Union

A tagged union is a type with several [variants]. A value of a tagged union
contains a [discriminant], which identifies a variant, and a value of that
variant.

postcard[comp.tagged-union.encoding]
A tagged union MUST be encoded as a [Discriminant] identifying its variant,
followed by the value of that variant.

The [Schema] defines the discriminant of each variant.

## Option

An option is a type that contains zero or one value of another type.

postcard[comp.option.encoding]
An option MUST be encoded as a [`bool`], which is `true` if a value is present,
followed by the value if it is present.

[type]: #glossary--type
[values]: #glossary--value
[headers]: #atoms--headers
[Length]: #atoms--headers--length
[Discriminant]: #atoms--headers--discriminant
[`bool`]: #atoms--fixed-size-atoms--bool
[Schema]: #schemas
[elements]: #glossary--element
[Lengths]: #implementation-considerations--security--lengths
[Nesting]: #implementation-considerations--security--nesting
[variants]: #glossary--variant
[aggregate]: #composites--aggregate
[byte]: #glossary--byte
[decoder]: #glossary--decoder
[Schemas]: #schemas
[The Serde Binding]: #the-serde-binding
