# Keys

As `postcard` is not a self-describing format, it is often useful for two
entities (e.g. a client and server) to be able to quickly cross-check that they
have a common understanding of message schemas, protocol versions, or the
meaning of a method.

This chapter describes postcard-schema "keys": an 8-byte hash of a list of
[items] (strings and schemas) that express information relevant to an
application. Each application chooses its items, and the [options] that control
which parts of each item are considered during hashing.

Keys borrow the idea of [content addressing] from tools such as Git and IPFS,
and apply it to types and APIs: a key is derived from what it identifies, so a
change that affects the key input (see [What Changes a Key]) produces a
different key. Unlike those tools, keys use a short, non-cryptographic hash, and
they hash a **type**'s postcard structure, rather than the **value** of a
specific piece of data.

## Design Considerations

1. Keys should be "misuse resistant", and are not a security boundary. Keys
   should never be used as a reason to skip input validation steps, or as an
   acceptable justification for unsafe code. A key can always be falsified by
   a malicious attacker.
2. The process of generating keys must be deterministic across platforms, so
   that clients and servers built at different times, or built on or for
   different targets, agree on the key produced from the same input.
3. Keys do not allow for Semantic Versioning (SemVer). SemVer lets an API change
   compatibly, for example by adding an enum variant, but hashing the schema
   causes any such change to produce a different key.

## How Keys Work

postcard-schema defines how [items] and [options] are converted to bytes to be
hashed, called the [key input], and how the key input is hashed into the key.

For example, an RPC framework may choose to use the following as the items of
its application-specific keys:

- the method name, e.g. "get_temperature_value"
- the schema of the request type
- the schema of the response type

This would allow the client and server of the RPC framework to use these keys
both to uniquely identify the method being called, and to ensure they have a
common understanding of the request and response types being exchanged.

```text
   the application chooses             postcard-schema defines
  ┌────────────────────────────┐      ┌─────────────────────────────┐
  │ options                    │      │ the key input:              │
  │ items, in order:           │─────▶│   how items become bytes    │─────▶  8-byte key
  │   "get_temperature_value"  │      │ the hash:                   │
  │   schema of the request    │      │   how those bytes are mixed │
  │   schema of the response   │      └─────────────────────────────┘
  └────────────────────────────┘
```

### What the Application Chooses

schema[key.items]
A key MUST be computed from a set of [options] and an ordered list of zero or
more [items].

This specification does not define the use of any particular items. Both sides
of an application need to choose the same items, in the same order, with the
same options.

schema[key.item]
An item MUST be either a string or a schema.

schema[key.options]
There MUST be exactly two options:

* The **names option**: include the names of fields and variants. Enabled by
  default.
* The **maximum lengths option**: include the maximum length of strings, byte
  arrays, sequences, and maps. Disabled by default.

### What Changes a Key

A schema is part of the key input by its [postcard type], such as an aggregate,
a sequence, or a tagged union, not by how the type is written in Rust. A change
to a type changes the key only if it changes that postcard type, or a name or
maximum length that the options include. Some different postcard types share an
encoding on the wire (see [Wire Compatibility]), but still produce different
keys. For example:

| Change                                                                        | Changes the key?                     |
| ------                                                                        | ----------------                     |
| adding, removing, reordering, or changing items                               | yes                                  |
| the name of a field or variant                                                | only with the names option           |
| the maximum length of a string, byte array, sequence or map                   | only with the maximum lengths option |
| the name of a struct or enum                                                  | no                                   |
| a newtype struct -> its field, a unit struct -> `()`, a tuple struct -> tuple | no                                   |
| `Vec<u8>` -> a byte array, a map -> a sequence of `(key, value)` tuples       | no                                   |
| a struct with named fields -> a tuple of the same types                       | only without the names option        |
| `u16` -> `u32`, `Option<T>` -> an enum with variants `None` and `Some(T)`     | yes                                  |

<!-- NOTE: End of "informal discussion", start of "impl details" -->

## Key Input

The [key input] is a header followed by the [items], each preceded by an item
tag:

```text
●──[ domain ]──[ options ]──[ count ]──┬───────────────────────────────────────┬──▶
   |----------- header -------------|  │   ┌──( 0x00 )──[   string    ]───┐    │
                                       └─▶─┤  | tag  |  |---- item ---|   ├──┐ │
                                         ▲ └──( 0x01 )──[    type     ]───┘  │ │
                                         │                                   │ │
                                         └───────────── count times ◀────────┴─┘
                                           |----------- items ------------|
```

schema[key.input.postcard]
The key input MUST be the postcard encoding of a `KeyInput`, as given by the
[Serde binding] of the [Postcard Wire Specification]:

```rust
struct KeyInput {
    domain: [u8; 22],
    options: u64,
    items: Vec<Item>,
}

enum Item {
    String(String),               // 0x00
    Schema(Type),                 // 0x01
}

enum Type {
    Bool,                         // 0x00
    I8,                           // 0x01
    U8,                           // 0x02
    I16,                          // 0x03
    I32,                          // 0x04
    I64,                          // 0x05
    I128,                         // 0x06
    U16,                          // 0x07
    U32,                          // 0x08
    U64,                          // 0x09
    U128,                         // 0x0A
    F32,                          // 0x0B
    F64,                          // 0x0C
    Char,                         // 0x0D
    String(MaxLen),               // 0x0E
    Unit,                         // 0x0F
    Option(Box<Type>),            // 0x10
    Sequence(MaxLen, Box<Type>),  // 0x11
    Aggregate(Vec<Element>),      // 0x12
    TaggedUnion(Vec<Element>),    // 0x13
    Schema,                       // 0x14
}

struct Element {
    name: Name,
    ty: Type,
}
```

The comments give each variant's discriminant, which postcard writes as one
byte: the item tag, or the type's tag. `Name` and `MaxLen` depend on the
[options], as below.

Postcard gives every `KeyInput` value a different encoding, so different items
or options never produce the same key input.

schema[key.input.domain]
The domain MUST be the array of 22 ASCII bytes `postcard-schema/key/v3`.

This array is not length prefixed.

schema[key.input.options]
The options MUST be written as a `varint(u64)`, with bit 0 (`0x01`) set if the
names option is enabled, bit 1 (`0x02`) set if the maximum lengths option is
enabled, and all other bits clear.

Options added in future will each use a new bit and be disabled by default, so
existing keys keep their values.

schema[key.input.item]
The items MUST be the application's items, in order: a string as `Item::String`,
and a schema as `Item::Schema` with its `Type`.

schema[key.input.name]
When the names option is enabled, `Name` MUST be `Option<String>`: the name of
a named element, or `None` for an unnamed one. When it is disabled, `Name` MUST
be `()`.

schema[key.input.max-len]
When the maximum lengths option is enabled, `MaxLen` MUST be `Option<u64>`:
the maximum length, or `None` if there is none. When it is disabled, `MaxLen`
MUST be `()`.

Postcard writes `()` as no bytes, so a disabled option adds nothing to the key
input.

A maximum length counts bytes for a string or a byte array, elements for a
sequence, and entries for a map.

### Normalization

Each kind of schema is written as the `Type` of its own postcard type (see
[Schemas]), except for the kinds below, which are written as another `Type`. No
other equivalences, such as those in [Wire Compatibility], are applied.

schema[key.input.bytes]
A `ByteArray` MUST be written as a `Type::Sequence` of `Type::U8`, with the byte
array's maximum length.

schema[key.input.map]
A `Map` MUST be written as a `Type::Sequence`, with the map's maximum length, of
`Type::Aggregate`s of two unnamed elements, the key type and then the value
type.

schema[key.input.tuple]
A `Tuple` MUST be written as a `Type::Aggregate` of unnamed elements.

schema[key.input.struct]
A `Struct` MUST be written as its `Data`: `Data::Unit` as `Type::Unit`,
`Data::Newtype(T)` as `T`, `Data::Tuple` as a `Type::Aggregate` of unnamed
elements, and `Data::Struct` as a `Type::Aggregate` of elements named after its
fields.

schema[key.input.enum]
An `Enum` MUST be written as a `Type::TaggedUnion` with one element per variant,
in order, named after the variant, whose type is the variant's `Data` written as
for a `Struct`. A variant is identified by its position; no discriminant is
written.

## Hash

schema[key.hash.fnv1a-128]
The key input MUST be hashed with 128-bit [FNV-1a], with offset basis
`0x6c62272e07bb014262b821756295c58d` and prime
`0x0000000001000000000000000000013B`.

schema[key.hash.fold]
The 128-bit hash MUST be folded to 64 bits by XORing its upper 64 bits with its
lower 64 bits.

schema[key.hash.fmix64]
The folded value `k` MUST then be mixed with the `fmix64` function of
[MurmurHash3], using wrapping 64-bit arithmetic.

As pseudocode:

```text
k ^= k >> 33
k *= 0xff51afd7ed558ccd
k ^= k >> 33
k *= 0xc4ceb9fe1a85ec53
k ^= k >> 33
```

`fmix64` spreads every input bit across the whole key, to reduce the chance of
collisions when a key is shortened to a prefix, or folded by XOR.

schema[key.hash.output]
The key MUST be the mixed value as 8 bytes, least significant byte first.

## Examples

An application that keys on a path and one schema, with the default options:
the path `ping`, and this struct:

```rust
struct Ping {
    seq: u32,
    data: Vec<u8>,
}
```

Key input:

```text
70 6f 73 74 63 61 72 64 2d 73 63 68 65 6d 61    domain: "postcard-schema/key/v3"
2f 6b 65 79 2f 76 33
01                                              options: names
02                                              2 items
00 04 70 69 6e 67                               a string, length 4: "ping"
01                                              a schema
12 02                                           an aggregate of 2 elements
01 03 73 65 71 08                               "seq": u32
01 04 64 61 74 61 11 02                         "data": a sequence of u8
```

Key:

```text
key: 94 a1 8b 70 c2 aa 4b 3f
```

The same application, for the path `cmd` and this enum:

```rust
enum Cmd {
    Stop,
    Go(u8),
}
```

Key input:

```text
70 6f 73 74 63 61 72 64 2d 73 63 68 65 6d 61    domain: "postcard-schema/key/v3"
2f 6b 65 79 2f 76 33
01                                              options: names
02                                              2 items
00 03 63 6d 64                                  a string, length 3: "cmd"
01                                              a schema
13 02                                           a tagged union of 2 variants
01 04 53 74 6f 70 0f                            "Stop": unit
01 02 47 6f 02                                  "Go": u8
```

Key:

```text
key: 22 11 ad 35 85 38 eb 76
```

One item, the schema of a `BTreeMap<String, u8>`, with the maximum lengths
option enabled and the names option disabled:

Key input:

```text
70 6f 73 74 63 61 72 64 2d 73 63 68 65 6d 61    domain: "postcard-schema/key/v3"
2f 6b 65 79 2f 76 33
02                                              options: maximum lengths
01                                              1 item
01                                              a schema
11 00                                           a sequence, no maximum length
12 02                                           of aggregates of 2 elements
0e 00                                           a string, no maximum length
02                                              u8
```

Key:

```text
key: c1 19 b9 32 34 91 d6 a0
```

An RPC framework that keys on a method name and the schemas of its request and
response, with both options enabled: the method `get_temperature_value`, and
these types:

```rust
struct Read {
    channel: u8,
}

type Response = Vec<MaxLenString<16>>;
```

Key input:

```text
70 6f 73 74 63 61 72 64 2d 73 63 68 65 6d 61    domain: "postcard-schema/key/v3"
2f 6b 65 79 2f 76 33
03                                              options: names, maximum lengths
03                                              3 items
00 15                                           a string, length 21:
67 65 74 5f 74 65 6d 70 65 72 61 74 75 72 65    "get_temperature_value"
5f 76 61 6c 75 65
01                                              a schema
12 01                                           an aggregate of 1 element
01 07 63 68 61 6e 6e 65 6c 02                   "channel": u8
01                                              a schema
11 00                                           a sequence, no maximum length
0e 01 10                                        of strings, maximum length 16
```

Key:

```text
key: 90 13 41 cc 15 df a3 cb
```

## Versions

This specification defines version 3 of keys. Earlier versions were defined only
by their implementations:

| Version | Used by                                         | Differences from version 3                                      |
|---      |---                                              |---                                                              |
| 1       | early postcard-rpc                              | FNV-1a 64; type names are hashed, so `String` and `&str` differ |
| 2       | postcard-schema 0.1 to 0.2, postcard-schema-ng  | FNV-1a 64; no lengths or counts; only `ByteArray` is normalized |
| 3       | postcard-schema 0.5, this specification         |                                                                 |

[content addressing]: https://en.wikipedia.org/wiki/Content-addressable_storage
[Schemas]: #the-postcard-schema-specification--schemas
[postcard type]: https://postcard.rs/wire/1.1/spec.html#glossary--type
[Postcard Wire Specification]: https://postcard.rs/wire/1.1/spec.html
[Serde binding]: https://postcard.rs/wire/1.1/spec.html#the-serde-binding
[What Changes a Key]: #keys--how-keys-work--what-changes-a-key
[Wire Compatibility]: https://postcard.rs/wire/1.1/spec.html#implementation-considerations--interoperability--schema-evolution--wire-compatibility
[items]: #glossary--item
[options]: #glossary--options
[key input]: #glossary--key-input
[FNV-1a]: https://datatracker.ietf.org/doc/draft-eastlake-fnv/
[MurmurHash3]: https://github.com/aappleby/smhasher/blob/master/src/MurmurHash3.cpp
