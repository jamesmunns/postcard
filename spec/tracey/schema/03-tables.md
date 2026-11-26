# Schema Tables

A [schema table] is a compact format for sending or storing schemas. It holds
one or more schemas in three parts:

- `nodes`: a list of nodes, each describing one type
- `roots`: a list of node indices, naming the "top level" types
- `names`: one string holding every name, which nodes refer to by byte range

Nodes refer to each other by index. Variants, fields, and the elements of tuples
aren't nodes: each is stored inside the node it belongs to.

A schema table is an ordinary postcard type, not a recursive one, so a message
can hold one as a field, and its schema is a finite schema like any other.

[Keys] are computed from schemas, rather than the bytes of a table. Tables of
the same schemas give the same keys, whatever their node order or how much they
are deduplicated.

## Encoding

schema[table.encoding]
A schema table MUST be the postcard encoding of a `SchemaTable`, as given by the
[Serde binding] of the [Postcard Wire Specification]:

```rust
struct SchemaTable {
    /// Always 1; see below.
    version: u32,
    /// Roots are the "entry point" types that were used to build this table.
    /// Each `u32` is an index into the `nodes` list.
    roots: Vec<u32>,
    /// Nodes are each a type, as one of the variants in the `Node` enum type.
    nodes: Vec<Node>,
    /// Every name in the table, as one UTF-8 string, which nodes refer to by
    /// `NameRef`.
    names: String,
}

/// A name: the bytes `offset..offset+len` of `names`. Used for the names of
/// types, variants and fields.
struct NameRef {
    offset: u32,
    len: u32,
}

enum Node {                                                 // Discriminant
    Bool,                                                   // 0x00
    I8,                                                     // 0x01
    U8,                                                     // 0x02
    I16,                                                    // 0x03
    I32,                                                    // 0x04
    I64,                                                    // 0x05
    I128,                                                   // 0x06
    U16,                                                    // 0x07
    U32,                                                    // 0x08
    U64,                                                    // 0x09
    U128,                                                   // 0x0A
    F32,                                                    // 0x0B
    F64,                                                    // 0x0C
    Char,                                                   // 0x0D
    String { max_len: Option<u64> },                        // 0x0E
    ByteArray { max_len: Option<u64> },                     // 0x0F
    Option(u32),                                            // 0x10
    Unit,                                                   // 0x11
    Seq { element: u32, max_len: Option<u64> },             // 0x12
    Tuple(Vec<u32>),                                        // 0x13
    Map { key: u32, val: u32, max_len: Option<u64> },       // 0x14
    Struct { name: NameRef, data: Data },                   // 0x15
    Enum { name: NameRef, variants: Vec<Variant> },         // 0x16
    Array { element: u32, len: u64 },                       // 0x17
}

/// The `Data` of a struct or enum
enum Data {                                                 // Discriminant
    /// A Unit Struct or Unit Variant
    Unit,                                                   // 0x00
    /// A Newtype Struct or Newtype Variant. The `u32` is the index of the inner
    /// type in the `nodes` list.
    Newtype(u32),                                           // 0x01
    /// A Tuple Struct or Tuple Variant. Each `u32` is the index of an
    /// element's type in the `nodes` list.
    Tuple(Vec<u32>),                                        // 0x02
    /// A (regular) Struct or Struct Variant. Its fields, in order.
    Struct(Vec<Field>),                                     // 0x03
}

/// A variant of an enum
struct Variant {
    name: NameRef,
    data: Data,
}

/// A field of a struct or struct variant. `ty` is the index of its type
/// in the `nodes` list.
struct Field {
    name: NameRef,
    ty: u32,
}
```

schema[table.version]
The version MUST be 1. A decoder MUST reject a table with any other
version.

The version is the first field, so a decoder could check it before decoding the
rest of the table. The `postcard-schema` crate decodes the whole table first, so
a later version with a different layout may fail to decode rather than being
rejected by its version.

## Meaning

schema[table.meaning]
Each root MUST be decoded as the schema of the node it refers to, and the
roots MUST be decoded in order. The schema of a node is given by the tables
below, where `T` is the schema of the node that a node index refers to, and
the `T`s of a list of node indices are the schemas of its nodes, in order.

| Node                        | Schema                                                |
|---                          |---                                                    |
| `Bool` to `Char`, `Unit`    | the kind of the same name                             |
| `String { max_len }`        | `String { max_len }`                                  |
| `ByteArray { max_len }`     | `ByteArray { max_len }`                               |
| `Option(n)`                 | `Option(T)`                                           |
| `Seq { element, max_len }`  | `Seq { element: T, max_len }`                         |
| `Tuple(indices)`            | `Tuple(T, ...)`                                       |
| `Map { key, val, max_len }` | `Map { key: T, val: T, max_len }`                     |
| `Struct { name, data }`     | `Struct { name, data }`, with `data` as below         |
| `Enum { name, variants }`   | `Enum { name, variants }`, with each variant as below |
| `Array { element, len }`    | `Tuple(T, ...)`, with `len` copies of `T`             |

| Data              | Schema data                                      |
|---                |---                                               |
| `Unit`            | `Unit`                                           |
| `Newtype(n)`      | `Newtype(T)`                                     |
| `Tuple(indices)`  | `Tuple(T, ...)`                                  |
| `Struct(fields)`  | `Struct(field, ...)`                             |
| a `Variant`       | a variant `name` with `data` as above            |
| a `Field`         | a field `name`, whose type is the schema of `ty` |

A `max_len` of `None` means there is no maximum length. A name is the string
that its `NameRef` refers to.

schema[table.array]
An `Array` MUST be decoded as a `Tuple` of `len` copies of the schema of its
`element`. An encoder SHOULD write a `Tuple` node of two or more node indices
that are all the same as an `Array`. The `Tuple` data of a struct or variant has
no `Array` form.

An `Array` only makes the table smaller: `[u8; 32]` and a tuple of 32 `u8`s are
the same schema, as serde writes both as a tuple.

## Validity

schema[table.root]
Every root MUST be the index of a node.

schema[table.child.order]
Every node index in a node, including those in its tuple elements, its variants
and its fields, MUST be less than the index of that node.

Each node therefore comes after all of its children, and following children
always moves towards the start of `nodes`. A table can't contain a
cycle, and decoding one always ends.

Variants and fields are stored inside their node, so every node index refers to
a type.

schema[table.name]
Every `NameRef` MUST be within `names`, and MUST start and end on the boundary
of a UTF-8 character.

schema[table.reject]
A decoder MUST reject a table that does not meet every requirement of this
section, including in nodes that no root refers to.

Checking every node makes validity a property of the table alone. Each index and
each name is checked once, so checking a table takes time linear in its size.

schema[table.max-len]
A decoder MAY reject a table with a maximum length, or an `Array` length, that
it can't represent.

schema[table.budget]
A decoder MUST limit the total size of the schemas it decodes a table into, and
MUST reject a table that would exceed that limit.

Nodes can be shared, so a few nodes can describe a very large schema. A table of
a `U8` and 64 `Tuple`s, each of two of the node before it, is 65 nodes, but its
last node describes 2<sup>64</sup> `U8`s.

An encoder would write each of those `Tuple`s as an `Array` of two, but a
decoder must accept either.

Each copy in an `Array` counts towards the limit, so an `Array` with a very
large `len` is two nodes that describe a very large schema. A decoder should
check the limit before making the copies.

Names count towards the size as well, since one long name can be used by many
nodes. The `postcard-schema` crate has three limits: the number of types,
variants and fields it produces, counting an `Array` once and each of its
copies; the bytes of names it produces, counting a name each time it is used;
and the depth of nested types, so that code that walks the decoded schemas
can't run out of stack.

schema[table.accept]
A decoder MUST accept every table that meets the requirements of this section
and is within its limits, whatever its node order, including tables with
repeated nodes or names, and nodes that no root refers to.

schema[table.dedup]
An encoder SHOULD write each node, each root, and each name once, and SHOULD NOT
write nodes that no root refers to.

A table only holds types. How an application uses them, such as the request and
response of a method, belongs in the application's own data, which can refer to
the types by node index.

A `NameRef` MAY refer to part of a longer name. The `postcard-schema` crate
reuses the first match, so the name `y` can be found inside the name `xy`.

## Example

The schema table of `struct Point { xy: u8, y: u8 }`, as written by the
`postcard-schema` crate:

```text
01                                  version: 1
01 01                               roots: [1]
02                                  nodes: 2 nodes
   02                                 0: U8
   15 00 05                           1: Struct { name: "Point" (0, 5),
      03 02                                data: Struct(2 fields:
         05 02 00                            "xy" (5, 2) of type 0,
         06 01 00                            "y" (6, 1) of type 0) }
07 50 6F 69 6E 74 78 79             names: "Pointxy"
```

[schema table]: #glossary--schema-table
[Keys]: #keys
[Serde binding]: https://postcard.rs/wire/1.1/spec.html#the-serde-binding
[Postcard Wire Specification]: https://postcard.rs/wire/1.1/spec.html
