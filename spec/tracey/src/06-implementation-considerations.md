# Implementation Considerations

This chapter describes concerns that the wire format does not cover. It contains
no requirements.

<!--
    TODO: Decide whether any of the SHOULDs below become tracked requirements,
    e.g. in a small spec of their own, so that each implementation can show it
    addresses them. Concrete limits (such as postcard's) belong in each
    library's own documentation.
-->

## Security

Decoders often decode messages from untrusted sources.

The limits recommended below are [Resource Limits]: a message that exceeds one
is rejected, even if it would otherwise be considered valid.

### Lengths

A decoder SHOULD NOT allocate memory based on a [Length] before checking it
against the bytes remaining, or against its own limit.

Checking a length against the bytes remaining does not limit the time spent
decoding. An element can be encoded as zero bytes, such as a [`unit`] or an
empty [Aggregate], so a short message can contain a sequence of many such
elements. A decoder SHOULD limit the number of elements it decodes from an
untrusted message.

<!--
    TODO: postcard caps preallocation at the bytes remaining
    (`SeqAccess::size_hint`), but decoding a `Vec<()>` with a length near
    `u64::MAX` loops that many times, from about ten bytes of input. Decide
    whether postcard should limit this, e.g. with an element budget in the
    deserializer.
-->

### Nesting

A schema that contains itself, such as a tree, can nest without limit. A decoder
that recurses for each level of nesting can overflow its stack. A decoder SHOULD
limit how deeply it nests when decoding untrusted messages.

<!--
    TODO: postcard has no nesting limit, and serde decoders recurse. Decide
    whether postcard should add one (and what the default would be).
-->

### Malformed Messages

A decoder that panics or aborts on a malformed message, instead of reporting an
error (see [Conformance]), can be made to crash by any sender.

### Hash Collisions

A map decoded into a hash table uses keys chosen by the sender, who can choose
keys whose hashes collide, so that decoding takes excessive time. The type being
decoded chooses the hasher, not this specification. A decoder SHOULD use a
hasher that the sender cannot predict, such as the `RandomState` hasher used by
default in Rust's `HashMap`.

## Interoperability

### Comparing Encoded Messages

As decoders accept non-canonical varints (see [Canonicalization]), equal values
can have different encodings. Floats are encoded bit for bit, so values that
compare equal, such as `0.0` and `-0.0`, also have different encodings.
A collection encoded in an unspecified order, such as a `HashMap` encoded in
iteration order, can also encode equal values differently. Comparing or hashing
encoded messages does not compare their values.

### Message Boundaries

A message does not contain its own length. A decoder MAY stop at the end of a
message, without decoding the bytes that follow it, so messages can be
concatenated. The application decides whether bytes left after a message are an
error.

### Schema Evolution

Adding, removing, or reordering fields or variants generally changes how a
schema's values are encoded, so the new schema is no longer wire compatible with
the previous one. Users who need compatible revisions of a schema are
responsible for keeping them compatible.

#### Wire Compatibility

Two schemas are wire compatible if they are identical once all names are
removed. Messages of one can be decoded as the other, subject to any
[Value Constraints].

Schemas that differ only by replacing a type with another type of the same
encoding are also wire compatible, for the values whose encodings match. For
example, the following types have the same encoding as each other:

* `bytes`, and a [Sequence] of `u8`.
* `string`, and `bytes` containing the same bytes. `bytes` decode as a `string`
  only if they are valid UTF-8.
* `char`, and a `string` containing it. A `string` decodes as a `char` only if
  it contains exactly one Unicode scalar value.
* Unsigned integers of different widths, such as `u16` and `u32`, other than
  `u8`. A value decodes as the narrower type only if it fits. The same applies
  to signed integers other than `i8`.
* `unit`, and an [Aggregate] with no elements.
* An aggregate with one element, and that element.
* `bool`, and a [Tagged Union] whose variants with discriminants 0 and 1 each
  contain `unit`.
* An [Option], and a tagged union whose variant with discriminant 0 contains
  `unit`, and whose variant with discriminant 1 contains the option's value.

[Conformance]: #the-postcard-wire-specification--conformance
[Length]: #atoms--headers--length
[`unit`]: #atoms--fixed-size-atoms--unit
[Canonicalization]: #atoms--varint-encoded-integers--canonicalization
[Aggregate]: #composites--aggregate
[Sequence]: #composites--sequence
[Option]: #composites--option
[Tagged Union]: #composites--tagged-union
[Value Constraints]: #schemas--value-constraints
[Resource Limits]: #schemas--resource-limits
