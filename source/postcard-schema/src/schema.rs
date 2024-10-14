use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum DataModelVariant {
    /// The "unit variant" Serde Data Model Type
    UnitVariant,
    /// The "newtype variant" Serde Data Model Type
    NewtypeVariant(&'static NamedType),
    /// The "Tuple Variant" Serde Data Model Type
    TupleVariant(&'static [&'static NamedType]),
    /// The "Struct Variant" Serde Data Model Type
    StructVariant(&'static [&'static NamedValue]),
}

/// Serde Data Model Types (and friends)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum DataModelType {
    /// The `bool` Serde Data Model Type
    Bool,

    /// The `i8` Serde Data Model Type
    I8,

    /// The `u8` Serde Data Model Type
    U8,

    /// A variably encoded i16
    I16,

    /// A variably encoded i32
    I32,

    /// A variably encoded i64
    I64,

    /// A variably encoded i128
    I128,

    /// A variably encoded u16
    U16,

    /// A variably encoded u32
    U32,

    /// A variably encoded u64
    U64,

    /// A variably encoded u128
    U128,

    /// A variably encoded usize
    Usize,

    /// A variably encoded isize
    Isize,

    /// The `f32` Serde Data Model Type
    F32,

    /// The `f64 Serde Data Model Type
    F64,

    /// The `char` Serde Data Model Type
    Char,

    /// The `String` Serde Data Model Type
    String,

    /// The `&[u8]` Serde Data Model Type
    ByteArray,

    /// The `Option<T>` Serde Data Model Type
    Option(&'static NamedType),

    /// The `()` Serde Data Model Type
    Unit,

    /// The "unit struct" Serde Data Model Type
    UnitStruct,

    /// The "newtype struct" Serde Data Model Type
    NewtypeStruct(&'static NamedType),

    /// The "Sequence" Serde Data Model Type
    Seq(&'static NamedType),

    /// The "Tuple" Serde Data Model Type
    Tuple(&'static [&'static NamedType]),

    /// The "Tuple Struct" Serde Data Model Type
    TupleStruct(&'static [&'static NamedType]),

    /// The "Map" Serde Data Model Type
    Map {
        /// The map "Key" type
        key: &'static NamedType,
        /// The map "Value" type
        val: &'static NamedType,
    },

    /// The "Struct" Serde Data Model Type
    Struct(&'static [&'static NamedValue]),

    /// The "Enum" Serde Data Model Type (which contains any of the "Variant" types)
    Enum(&'static [&'static NamedVariant]),

    /// A NamedType/OwnedNamedType
    Schema,
}

/// A data type with a name - e.g. a field of a Struct
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct NamedValue {
    /// The name of this value
    pub name: &'static str,
    /// The type of this value
    pub ty: &'static NamedType,
}

/// A data type - e.g. a custom `struct Foo{ ... }` type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct NamedType {
    /// The name of this type
    pub name: &'static str,
    /// The type
    pub ty: &'static DataModelType,
}

/// An enum variant with a name, e.g. `T::Bar(...)`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct NamedVariant {
    /// The name of this variant
    pub name: &'static str,
    /// The type of this variant
    pub ty: &'static DataModelVariant,
}

#[cfg(any(feature = "use-std", feature = "alloc"))]
pub mod owned {
    use super::*;

    #[cfg(feature = "use-std")]
    use std::{boxed::Box, collections::HashSet, string::String, vec::Vec};

    #[cfg(all(not(feature = "use-std"), feature = "alloc"))]
    use alloc::{
        boxed::Box,
        string::{String, ToString},
        vec::Vec,
    };

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum OwnedDataModelVariant {
        /// The "unit variant" Serde Data Model Type
        UnitVariant,
        /// The "newtype variant" Serde Data Model Type
        NewtypeVariant(Box<OwnedNamedType>),
        /// The "Tuple Variant" Serde Data Model Type
        TupleVariant(Vec<OwnedNamedType>),
        /// The "Struct Variant" Serde Data Model Type
        StructVariant(Vec<OwnedNamedValue>),
    }

    /// Serde Data Model Types (and friends)
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum OwnedDataModelType {
        /// The `bool` Serde Data Model Type
        Bool,

        /// The `i8` Serde Data Model Type
        I8,

        /// The `u8` Serde Data Model Type
        U8,

        /// A variably encoded i16
        I16,

        /// A variably encoded i32
        I32,

        /// A variably encoded i64
        I64,

        /// A variably encoded i128
        I128,

        /// A variably encoded u16
        U16,

        /// A variably encoded u32
        U32,

        /// A variably encoded u64
        U64,

        /// A variably encoded u128
        U128,

        /// A variably encoded usize
        Usize,

        /// A variably encoded isize
        Isize,

        /// The `f32` Serde Data Model Type
        F32,

        /// The `f64 Serde Data Model Type
        F64,

        /// The `char` Serde Data Model Type
        Char,

        /// The `String` Serde Data Model Type
        String,

        /// The `&[u8]` Serde Data Model Type
        ByteArray,

        /// The `Option<T>` Serde Data Model Type
        Option(Box<OwnedNamedType>),

        /// The `()` Serde Data Model Type
        Unit,

        /// The "unit struct" Serde Data Model Type
        UnitStruct,

        /// The "newtype struct" Serde Data Model Type
        NewtypeStruct(Box<OwnedNamedType>),

        /// The "Sequence" Serde Data Model Type
        Seq(Box<OwnedNamedType>),

        /// The "Tuple" Serde Data Model Type
        Tuple(Vec<OwnedNamedType>),

        /// The "Tuple Struct" Serde Data Model Type
        TupleStruct(Vec<OwnedNamedType>),

        /// The "Map" Serde Data Model Type
        Map {
            /// The map "Key" type
            key: Box<OwnedNamedType>,
            /// The map "Value" type
            val: Box<OwnedNamedType>,
        },

        /// The "Struct" Serde Data Model Type
        Struct(Vec<OwnedNamedValue>),

        /// The "Enum" Serde Data Model Type (which contains any of the "Variant" types)
        Enum(Vec<OwnedNamedVariant>),

        /// A NamedType/OwnedNamedType
        Schema,
    }

    impl From<&DataModelVariant> for OwnedDataModelVariant {
        fn from(value: &DataModelVariant) -> Self {
            match value {
                DataModelVariant::UnitVariant => Self::UnitVariant,
                DataModelVariant::NewtypeVariant(d) => Self::NewtypeVariant(Box::new((*d).into())),
                DataModelVariant::TupleVariant(d) => {
                    Self::TupleVariant(d.iter().map(|i| (*i).into()).collect())
                }
                DataModelVariant::StructVariant(d) => {
                    Self::StructVariant(d.iter().map(|i| (*i).into()).collect())
                }
            }
        }
    }

    impl From<&DataModelType> for OwnedDataModelType {
        fn from(other: &DataModelType) -> Self {
            match other {
                DataModelType::Bool => Self::Bool,
                DataModelType::I8 => Self::I8,
                DataModelType::U8 => Self::U8,
                DataModelType::I16 => Self::I16,
                DataModelType::I32 => Self::I32,
                DataModelType::I64 => Self::I64,
                DataModelType::I128 => Self::I128,
                DataModelType::U16 => Self::U16,
                DataModelType::U32 => Self::U32,
                DataModelType::U64 => Self::U64,
                DataModelType::U128 => Self::U128,
                DataModelType::Usize => Self::Usize,
                DataModelType::Isize => Self::Isize,
                DataModelType::F32 => Self::F32,
                DataModelType::F64 => Self::F64,
                DataModelType::Char => Self::Char,
                DataModelType::String => Self::String,
                DataModelType::ByteArray => Self::ByteArray,
                DataModelType::Option(o) => Self::Option(Box::new((*o).into())),
                DataModelType::Unit => Self::Unit,
                DataModelType::UnitStruct => Self::UnitStruct,
                DataModelType::NewtypeStruct(nts) => Self::NewtypeStruct(Box::new((*nts).into())),
                DataModelType::Seq(s) => Self::Seq(Box::new((*s).into())),
                DataModelType::Tuple(t) => Self::Tuple(t.iter().map(|i| (*i).into()).collect()),
                DataModelType::TupleStruct(ts) => {
                    Self::TupleStruct(ts.iter().map(|i| (*i).into()).collect())
                }
                DataModelType::Map { key, val } => Self::Map {
                    key: Box::new((*key).into()),
                    val: Box::new((*val).into()),
                },
                DataModelType::Struct(s) => Self::Struct(s.iter().map(|i| (*i).into()).collect()),
                DataModelType::Enum(e) => Self::Enum(e.iter().map(|i| (*i).into()).collect()),
                DataModelType::Schema => Self::Schema,
            }
        }
    }

    /// A data type with a name - e.g. a field of a Struct
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct OwnedNamedValue {
        /// The name of this value
        pub name: String,
        /// The type of this value
        pub ty: OwnedNamedType,
    }

    impl From<&NamedValue> for OwnedNamedValue {
        fn from(value: &NamedValue) -> Self {
            Self {
                name: value.name.to_string(),
                ty: value.ty.into(),
            }
        }
    }

    /// A data type - e.g. a custom `struct Foo{ ... }` type
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct OwnedNamedType {
        /// The name of this type
        pub name: String,
        /// The type
        pub ty: OwnedDataModelType,
    }

    impl core::fmt::Display for OwnedNamedType {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            let pc = self.to_pseudocode();
            f.write_str(&pc)
        }
    }

    impl OwnedNamedType {
        /// Convert an [OwnedNamedType] to a pseudo-Rust type format
        pub fn to_pseudocode(&self) -> String {
            let mut buf = String::new();
            fmt::fmt_owned_nt_to_buf(self, &mut buf, true);
            buf
        }

        /// Collect all types used recursively by this type
        #[cfg(feature = "use-std")]
        pub fn all_used_types(&self) -> HashSet<OwnedNamedType> {
            let mut buf = HashSet::new();
            fmt::discover_tys(self, &mut buf);
            buf
        }
    }

    impl From<&NamedType> for OwnedNamedType {
        fn from(value: &NamedType) -> Self {
            Self {
                name: value.name.to_string(),
                ty: value.ty.into(),
            }
        }
    }

    /// An enum variant with a name, e.g. `T::Bar(...)`
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct OwnedNamedVariant {
        /// The name of this variant
        pub name: String,
        /// The type of this variant
        pub ty: OwnedDataModelVariant,
    }

    impl From<&NamedVariant> for OwnedNamedVariant {
        fn from(value: &NamedVariant) -> Self {
            Self {
                name: value.name.to_string(),
                ty: value.ty.into(),
            }
        }
    }

    impl crate::Schema for OwnedNamedType {
        const SCHEMA: &'static NamedType = &NamedType {
            name: "OwnedNamedType",
            ty: &DataModelType::Schema,
        };
    }
}

#[cfg(any(feature = "use-std", feature = "alloc"))]
pub(crate) mod fmt {
    use super::owned::{OwnedNamedType, OwnedDataModelType, OwnedDataModelVariant};

    #[cfg(feature = "use-std")]
    use std::{string::String, vec::Vec};

    #[cfg(all(not(feature = "use-std"), feature = "alloc"))]
    extern crate alloc;

    #[cfg(all(not(feature = "use-std"), feature = "alloc"))]
    use alloc::{format, string::String, vec::Vec};

    /// Is this [`OwnedDataModelType`] a primitive?
    pub fn is_prim(osdmty: &OwnedDataModelType) -> bool {
        match osdmty {
            OwnedDataModelType::Bool => true,
            OwnedDataModelType::I8 => true,
            OwnedDataModelType::U8 => true,
            OwnedDataModelType::I16 => true,
            OwnedDataModelType::I32 => true,
            OwnedDataModelType::I64 => true,
            OwnedDataModelType::I128 => true,
            OwnedDataModelType::U16 => true,
            OwnedDataModelType::U32 => true,
            OwnedDataModelType::U64 => true,
            OwnedDataModelType::U128 => true,
            OwnedDataModelType::Usize => true,
            OwnedDataModelType::Isize => true,
            OwnedDataModelType::F32 => true,
            OwnedDataModelType::F64 => true,
            OwnedDataModelType::Char => true,
            OwnedDataModelType::String => true,
            OwnedDataModelType::ByteArray => true,
            OwnedDataModelType::Option(owned_named_type) => is_prim(&owned_named_type.ty),
            OwnedDataModelType::Unit => true,
            OwnedDataModelType::UnitStruct => true,
            OwnedDataModelType::NewtypeStruct(owned_named_type) => is_prim(&owned_named_type.ty),
            OwnedDataModelType::Seq(_) => false,
            OwnedDataModelType::Tuple(_) => false,
            OwnedDataModelType::TupleStruct(vec) => vec.iter().all(|e| is_prim(&e.ty)),
            OwnedDataModelType::Map { key, val } => is_prim(&key.ty) && is_prim(&val.ty),
            OwnedDataModelType::Struct(_) => false,
            OwnedDataModelType::Enum(_) => false,
            OwnedDataModelType::Schema => true,
        }
    }

    /// Format an [`OwnedNamedType`] to the given string.
    ///
    /// Use `top_level = true` when this is a standalone type, and `top_level = false`
    /// when this type is contained within another type
    pub fn fmt_owned_nt_to_buf(ont: &OwnedNamedType, buf: &mut String, top_level: bool) {
        match &ont.ty {
            OwnedDataModelType::Bool => *buf += "bool",
            OwnedDataModelType::I8 => *buf += "i8",
            OwnedDataModelType::U8 => *buf += "u8",
            OwnedDataModelType::I16 => *buf += "i16",
            OwnedDataModelType::I32 => *buf += "i32",
            OwnedDataModelType::I64 => *buf += "i64",
            OwnedDataModelType::I128 => *buf += "i128",
            OwnedDataModelType::U16 => *buf += "u16",
            OwnedDataModelType::U32 => *buf += "u32",
            OwnedDataModelType::U64 => *buf += "u64",
            OwnedDataModelType::U128 => *buf += "u128",
            OwnedDataModelType::Usize => *buf += "usize",
            OwnedDataModelType::Isize => *buf += "isize",
            OwnedDataModelType::F32 => *buf += "f32",
            OwnedDataModelType::F64 => *buf += "f64",
            OwnedDataModelType::Char => *buf += "char",
            OwnedDataModelType::String => *buf += "String",
            OwnedDataModelType::ByteArray => *buf += "[u8]",
            OwnedDataModelType::Option(owned_named_type) => {
                *buf += "Option<";
                fmt_owned_nt_to_buf(owned_named_type, buf, false);
                *buf += ">";
            }
            OwnedDataModelType::Unit => *buf += "()",
            OwnedDataModelType::UnitStruct => {
                if top_level {
                    *buf += "struct ";
                }
                *buf += &ont.name;
            }
            OwnedDataModelType::NewtypeStruct(owned_named_type) => {
                if top_level {
                    *buf += "struct ";
                }
                *buf += &ont.name;
                if top_level {
                    *buf += "(";
                    fmt_owned_nt_to_buf(owned_named_type, buf, false);
                    *buf += ")";
                }
            }
            OwnedDataModelType::Seq(owned_named_type) => {
                *buf += "[";
                *buf += &owned_named_type.name;
                *buf += "]";
            }
            OwnedDataModelType::Tuple(vec) => {
                if !vec.is_empty() {
                    let first = &vec[0];
                    if vec.iter().all(|v| first == v) {
                        // This is a fixed size array
                        *buf += "[";
                        *buf += &first.name;
                        *buf += "; ";
                        *buf += &format!("{}", vec.len());
                        *buf += "]";
                    } else {
                        *buf += "(";
                        let fields = vec
                            .iter()
                            .map(|v| {
                                let mut buf = String::new();
                                fmt_owned_nt_to_buf(v, &mut buf, false);
                                buf
                            })
                            .collect::<Vec<_>>()
                            .join(", ");
                        *buf += &fields;
                        *buf += ")";
                    }
                } else {
                    *buf += "()";
                }
            }
            OwnedDataModelType::TupleStruct(vec) => {
                if top_level {
                    *buf += "struct ";
                    *buf += &ont.name;
                    *buf += "(";
                    let fields = vec
                        .iter()
                        .map(|v| {
                            let mut buf = String::new();
                            fmt_owned_nt_to_buf(v, &mut buf, false);
                            buf
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    *buf += &fields;
                    *buf += ")";
                } else {
                    *buf += &ont.name;
                }
            }
            OwnedDataModelType::Map { key, val } => {
                *buf += "Map<";
                *buf += &key.name;
                *buf += ", ";
                *buf += &val.name;
                *buf += ">";
            }
            OwnedDataModelType::Struct(vec) => {
                if top_level {
                    *buf += "struct ";
                    *buf += &ont.name;
                    *buf += " { ";
                    let fields = vec
                        .iter()
                        .map(|v| {
                            let mut buf = String::new();
                            buf += &v.name;
                            buf += ": ";
                            fmt_owned_nt_to_buf(&v.ty, &mut buf, false);
                            buf
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    *buf += &fields;
                    *buf += " }";
                } else {
                    *buf += &ont.name;
                }
            }
            OwnedDataModelType::Enum(vec) => {
                if top_level {
                    *buf += "enum ";
                    *buf += &ont.name;
                    *buf += " { ";

                    let fields = vec
                        .iter()
                        .map(|v| {
                            let mut buf = String::new();
                            buf += &v.name;
                            match &v.ty {
                                OwnedDataModelVariant::UnitVariant => {}
                                OwnedDataModelVariant::NewtypeVariant(owned_named_type) => {
                                    buf += "(";
                                    fmt_owned_nt_to_buf(owned_named_type, &mut buf, false);
                                    buf += ")";
                                }
                                OwnedDataModelVariant::TupleVariant(vec) => {
                                    buf += "(";
                                    let fields = vec
                                        .iter()
                                        .map(|ont| {
                                            let mut buf = String::new();
                                            fmt_owned_nt_to_buf(ont, &mut buf, false);
                                            buf
                                        })
                                        .collect::<Vec<_>>()
                                        .join(", ");
                                    buf += &fields;
                                    buf += ")";
                                }
                                OwnedDataModelVariant::StructVariant(vec) => {
                                    buf += "{ ";
                                    let fields = vec
                                        .iter()
                                        .map(|nv| {
                                            let mut buf = String::new();
                                            buf += &nv.name;
                                            buf += ": ";
                                            fmt_owned_nt_to_buf(&nv.ty, &mut buf, false);
                                            buf
                                        })
                                        .collect::<Vec<_>>()
                                        .join(", ");
                                    buf += &fields;
                                    buf += "}";
                                }
                            }
                            buf
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    *buf += &fields;
                    *buf += " }";
                } else {
                    *buf += &ont.name;
                }
            }
            OwnedDataModelType::Schema => *buf += "Schema",
        }
    }

    /// Collect unique types mentioned by this [`OwnedNamedType`]
    #[cfg(feature = "use-std")]
    pub fn discover_tys(ont: &OwnedNamedType, set: &mut std::collections::HashSet<OwnedNamedType>) {
        set.insert(ont.clone());
        discover_tys_sdm(&ont.ty, set);
    }

    /// Collect unique types mentioned by this [`OwnedDataModelType`]
    #[cfg(feature = "use-std")]
    pub fn discover_tys_sdm(sdm: &OwnedDataModelType, set: &mut std::collections::HashSet<OwnedNamedType>) {
        use crate::Schema;
        match sdm {
            OwnedDataModelType::Bool => set.insert(bool::SCHEMA.into()),
            OwnedDataModelType::I8 => set.insert(i8::SCHEMA.into()),
            OwnedDataModelType::U8 => set.insert(u8::SCHEMA.into()),
            OwnedDataModelType::I16 => set.insert(i16::SCHEMA.into()),
            OwnedDataModelType::I32 => set.insert(i32::SCHEMA.into()),
            OwnedDataModelType::I64 => set.insert(i64::SCHEMA.into()),
            OwnedDataModelType::I128 => set.insert(i128::SCHEMA.into()),
            OwnedDataModelType::U16 => set.insert(u16::SCHEMA.into()),
            OwnedDataModelType::U32 => set.insert(u32::SCHEMA.into()),
            OwnedDataModelType::U64 => set.insert(u64::SCHEMA.into()),
            OwnedDataModelType::U128 => set.insert(u128::SCHEMA.into()),

            // TODO: usize and isize don't impl Schema, which, fair.
            OwnedDataModelType::Usize => unreachable!(),
            OwnedDataModelType::Isize => unreachable!(),
            //
            OwnedDataModelType::F32 => set.insert(f32::SCHEMA.into()),
            OwnedDataModelType::F64 => set.insert(f64::SCHEMA.into()),
            OwnedDataModelType::Char => set.insert(char::SCHEMA.into()),
            OwnedDataModelType::String => set.insert(String::SCHEMA.into()),
            OwnedDataModelType::ByteArray => set.insert(<[u8]>::SCHEMA.into()),
            OwnedDataModelType::Option(owned_named_type) => {
                discover_tys(owned_named_type, set);
                false
            }
            OwnedDataModelType::Unit => set.insert(<()>::SCHEMA.into()),
            OwnedDataModelType::UnitStruct => false,
            OwnedDataModelType::NewtypeStruct(owned_named_type) => {
                discover_tys(owned_named_type, set);
                false
            }
            OwnedDataModelType::Seq(owned_named_type) => {
                discover_tys(owned_named_type, set);
                false
            }
            OwnedDataModelType::Tuple(vec) | OwnedDataModelType::TupleStruct(vec) => {
                for v in vec.iter() {
                    discover_tys_sdm(&v.ty, set);
                }
                false
            }
            OwnedDataModelType::Map { key, val } => {
                discover_tys(key, set);
                discover_tys(val, set);
                false
            }
            OwnedDataModelType::Struct(vec) => {
                for v in vec.iter() {
                    discover_tys(&v.ty, set);
                }
                false
            }
            OwnedDataModelType::Enum(vec) => {
                for v in vec.iter() {
                    match &v.ty {
                        OwnedDataModelVariant::UnitVariant => {}
                        OwnedDataModelVariant::NewtypeVariant(owned_named_type) => {
                            discover_tys(owned_named_type, set);
                        }
                        OwnedDataModelVariant::TupleVariant(vec) => {
                            for v in vec.iter() {
                                discover_tys(v, set);
                            }
                        }
                        OwnedDataModelVariant::StructVariant(vec) => {
                            for v in vec.iter() {
                                discover_tys(&v.ty, set);
                            }
                        }
                    }
                }
                false
            }
            OwnedDataModelType::Schema => todo!(),
        };
    }
}
