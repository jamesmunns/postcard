//! The `Schema` impls of the schema table types, written by hand as this crate
//! can't use its own derive. `tests/verify_impls.rs` checks them.

use super::{names, NameRef, SchemaTable, SchemaTableRef, TableField};
use crate::{
    schema::{Data, DataModelType as D, NamedField as F, Variant as V},
    Schema,
};

const U32: &D = &D::U32;
const MAX_LEN: &D = <Option<u64> as Schema>::SCHEMA;
const U32S: &D = &D::Seq {
    element: U32,
    max_len: None,
};

const fn unit(name: &'static str) -> V {
    V {
        name,
        data: Data::Unit,
    }
}

const fn field(name: &'static str, ty: &'static D) -> F {
    F { name, ty }
}

impl Schema for NameRef {
    const SCHEMA: &'static D = &D::Struct {
        name: "NameRef",
        data: Data::Struct(&[&field("offset", U32), &field("len", U32)]),
    };
}

impl Schema for TableField {
    const SCHEMA: &'static D = &D::Struct {
        name: "TableField",
        data: Data::Struct(&[&field("name", NameRef::SCHEMA), &field("ty", U32)]),
    };
}

const DATA: &D = &D::Enum {
    name: names::DATA,
    variants: &[
        &unit("Unit"),
        &V {
            name: "Newtype",
            data: Data::Newtype(U32),
        },
        &V {
            name: "Tuple",
            data: Data::Newtype(U32S),
        },
        &V {
            name: "Struct",
            data: Data::Newtype(&D::Seq {
                element: TableField::SCHEMA,
                max_len: None,
            }),
        },
    ],
};

const MAX_LEN_FIELD: &F = &field("max_len", MAX_LEN);
const NAME_FIELD: &F = &field("name", NameRef::SCHEMA);
const DATA_FIELD: &F = &field("data", DATA);

const VARIANT: &D = &D::Struct {
    name: names::VARIANT,
    data: Data::Struct(&[NAME_FIELD, DATA_FIELD]),
};

const NODE: &D = &D::Enum {
    name: names::NODE,
    variants: &[
        &unit("Bool"),
        &unit("I8"),
        &unit("U8"),
        &unit("I16"),
        &unit("I32"),
        &unit("I64"),
        &unit("I128"),
        &unit("U16"),
        &unit("U32"),
        &unit("U64"),
        &unit("U128"),
        &unit("F32"),
        &unit("F64"),
        &unit("Char"),
        &V {
            name: "String",
            data: Data::Struct(&[MAX_LEN_FIELD]),
        },
        &V {
            name: "ByteArray",
            data: Data::Struct(&[MAX_LEN_FIELD]),
        },
        &V {
            name: "Option",
            data: Data::Newtype(U32),
        },
        &unit("Unit"),
        &V {
            name: "Seq",
            data: Data::Struct(&[&field("element", U32), MAX_LEN_FIELD]),
        },
        &V {
            name: "Tuple",
            data: Data::Newtype(U32S),
        },
        &V {
            name: "Map",
            data: Data::Struct(&[&field("key", U32), &field("val", U32), MAX_LEN_FIELD]),
        },
        &V {
            name: "Struct",
            data: Data::Struct(&[NAME_FIELD, DATA_FIELD]),
        },
        &V {
            name: "Enum",
            data: Data::Struct(&[
                NAME_FIELD,
                &field(
                    "variants",
                    &D::Seq {
                        element: VARIANT,
                        max_len: None,
                    },
                ),
            ]),
        },
        &V {
            name: "Array",
            data: Data::Struct(&[&field("element", U32), &field("len", &D::U64)]),
        },
    ],
};

/// The schema of every form of a schema table
const TABLE: &D = &D::Struct {
    name: names::TABLE,
    data: Data::Struct(&[
        &field("version", U32),
        &field("roots", U32S),
        &field(
            "nodes",
            &D::Seq {
                element: NODE,
                max_len: None,
            },
        ),
        &field("names", &D::String { max_len: None }),
    ]),
};

impl Schema for SchemaTableRef<'_> {
    const SCHEMA: &'static D = TABLE;
}

impl<
        const ROOTS: usize,
        const NODES: usize,
        const CHILDREN: usize,
        const VARIANTS: usize,
        const FIELDS: usize,
        const NAME_BYTES: usize,
    > Schema for SchemaTable<ROOTS, NODES, CHILDREN, VARIANTS, FIELDS, NAME_BYTES>
{
    const SCHEMA: &'static D = TABLE;
}

#[cfg(any(feature = "std", feature = "alloc"))]
mod owned {
    use super::super::{
        OwnedSchemaTable, OwnedTableData, OwnedTableNode, OwnedTableVariant, RawOwnedSchemaTable,
    };
    use super::{DATA, NODE, TABLE, VARIANT};
    use crate::{schema::DataModelType as D, Schema};

    impl Schema for OwnedSchemaTable {
        const SCHEMA: &'static D = TABLE;
    }

    impl Schema for RawOwnedSchemaTable {
        const SCHEMA: &'static D = TABLE;
    }

    impl Schema for OwnedTableNode {
        const SCHEMA: &'static D = NODE;
    }

    impl Schema for OwnedTableData {
        const SCHEMA: &'static D = DATA;
    }

    impl Schema for OwnedTableVariant {
        const SCHEMA: &'static D = VARIANT;
    }
}
