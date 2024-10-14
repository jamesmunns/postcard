use serde::{Deserialize, Serialize};

/// Serde Data Model Types (and friends)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum SdmTy {
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

    /// The "unit variant" Serde Data Model Type
    UnitVariant,

    /// The "newtype struct" Serde Data Model Type
    NewtypeStruct(&'static NamedType),

    /// The "newtype variant" Serde Data Model Type
    NewtypeVariant(&'static NamedType),

    /// The "Sequence" Serde Data Model Type
    Seq(&'static NamedType),

    /// The "Tuple" Serde Data Model Type
    Tuple(&'static [&'static NamedType]),

    /// The "Tuple Struct" Serde Data Model Type
    TupleStruct(&'static [&'static NamedType]),

    /// The "Tuple Variant" Serde Data Model Type
    TupleVariant(&'static [&'static NamedType]),

    /// The "Map" Serde Data Model Type
    Map {
        /// The map "Key" type
        key: &'static NamedType,
        /// The map "Value" type
        val: &'static NamedType,
    },

    /// The "Struct" Serde Data Model Type
    Struct(&'static [&'static NamedValue]),

    /// The "Struct Variant" Serde Data Model Type
    StructVariant(&'static [&'static NamedValue]),

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
    pub ty: &'static SdmTy,
}

/// An enum variant with a name, e.g. `T::Bar(...)`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct NamedVariant {
    /// The name of this variant
    pub name: &'static str,
    /// The type of this variant
    pub ty: &'static SdmTy,
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

    /// Serde Data Model Types (and friends)
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum OwnedSdmTy {
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

        /// The "unit variant" Serde Data Model Type
        UnitVariant,

        /// The "newtype struct" Serde Data Model Type
        NewtypeStruct(Box<OwnedNamedType>),

        /// The "newtype variant" Serde Data Model Type
        NewtypeVariant(Box<OwnedNamedType>),

        /// The "Sequence" Serde Data Model Type
        Seq(Box<OwnedNamedType>),

        /// The "Tuple" Serde Data Model Type
        Tuple(Vec<OwnedNamedType>),

        /// The "Tuple Struct" Serde Data Model Type
        TupleStruct(Vec<OwnedNamedType>),

        /// The "Tuple Variant" Serde Data Model Type
        TupleVariant(Vec<OwnedNamedType>),

        /// The "Map" Serde Data Model Type
        Map {
            /// The map "Key" type
            key: Box<OwnedNamedType>,
            /// The map "Value" type
            val: Box<OwnedNamedType>,
        },

        /// The "Struct" Serde Data Model Type
        Struct(Vec<OwnedNamedValue>),

        /// The "Struct Variant" Serde Data Model Type
        StructVariant(Vec<OwnedNamedValue>),

        /// The "Enum" Serde Data Model Type (which contains any of the "Variant" types)
        Enum(Vec<OwnedNamedVariant>),

        /// A NamedType/OwnedNamedType
        Schema,
    }

    impl From<&SdmTy> for OwnedSdmTy {
        fn from(other: &SdmTy) -> Self {
            match other {
                SdmTy::Bool => Self::Bool,
                SdmTy::I8 => Self::I8,
                SdmTy::U8 => Self::U8,
                SdmTy::I16 => Self::I16,
                SdmTy::I32 => Self::I32,
                SdmTy::I64 => Self::I64,
                SdmTy::I128 => Self::I128,
                SdmTy::U16 => Self::U16,
                SdmTy::U32 => Self::U32,
                SdmTy::U64 => Self::U64,
                SdmTy::U128 => Self::U128,
                SdmTy::Usize => Self::Usize,
                SdmTy::Isize => Self::Isize,
                SdmTy::F32 => Self::F32,
                SdmTy::F64 => Self::F64,
                SdmTy::Char => Self::Char,
                SdmTy::String => Self::String,
                SdmTy::ByteArray => Self::ByteArray,
                SdmTy::Option(o) => Self::Option(Box::new((*o).into())),
                SdmTy::Unit => Self::Unit,
                SdmTy::UnitStruct => Self::UnitStruct,
                SdmTy::UnitVariant => Self::UnitVariant,
                SdmTy::NewtypeStruct(nts) => Self::NewtypeStruct(Box::new((*nts).into())),
                SdmTy::NewtypeVariant(ntv) => Self::NewtypeVariant(Box::new((*ntv).into())),
                SdmTy::Seq(s) => Self::Seq(Box::new((*s).into())),
                SdmTy::Tuple(t) => Self::Tuple(t.iter().map(|i| (*i).into()).collect()),
                SdmTy::TupleStruct(ts) => {
                    Self::TupleStruct(ts.iter().map(|i| (*i).into()).collect())
                }
                SdmTy::TupleVariant(tv) => {
                    Self::TupleVariant(tv.iter().map(|i| (*i).into()).collect())
                }
                SdmTy::Map { key, val } => Self::Map {
                    key: Box::new((*key).into()),
                    val: Box::new((*val).into()),
                },
                SdmTy::Struct(s) => Self::Struct(s.iter().map(|i| (*i).into()).collect()),
                SdmTy::StructVariant(sv) => {
                    Self::StructVariant(sv.iter().map(|i| (*i).into()).collect())
                }
                SdmTy::Enum(e) => Self::Enum(e.iter().map(|i| (*i).into()).collect()),
                SdmTy::Schema => Self::Schema,
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
        pub ty: OwnedSdmTy,
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
        pub ty: OwnedSdmTy,
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
            ty: &SdmTy::Schema,
        };
    }
}

#[cfg(any(feature = "use-std", feature = "alloc"))]
pub(crate) mod fmt {
    use super::owned::{OwnedNamedType, OwnedSdmTy};

    #[cfg(feature = "use-std")]
    use std::{string::String, vec::Vec};

    #[cfg(all(not(feature = "use-std"), feature = "alloc"))]
    extern crate alloc;

    #[cfg(all(not(feature = "use-std"), feature = "alloc"))]
    use alloc::{format, string::String, vec::Vec};

    /// Is this [`OwnedSdmTy`] a primitive?
    pub fn is_prim(osdmty: &OwnedSdmTy) -> bool {
        match osdmty {
            OwnedSdmTy::Bool => true,
            OwnedSdmTy::I8 => true,
            OwnedSdmTy::U8 => true,
            OwnedSdmTy::I16 => true,
            OwnedSdmTy::I32 => true,
            OwnedSdmTy::I64 => true,
            OwnedSdmTy::I128 => true,
            OwnedSdmTy::U16 => true,
            OwnedSdmTy::U32 => true,
            OwnedSdmTy::U64 => true,
            OwnedSdmTy::U128 => true,
            OwnedSdmTy::Usize => true,
            OwnedSdmTy::Isize => true,
            OwnedSdmTy::F32 => true,
            OwnedSdmTy::F64 => true,
            OwnedSdmTy::Char => true,
            OwnedSdmTy::String => true,
            OwnedSdmTy::ByteArray => true,
            OwnedSdmTy::Option(owned_named_type) => is_prim(&owned_named_type.ty),
            OwnedSdmTy::Unit => true,
            OwnedSdmTy::UnitStruct => true,
            OwnedSdmTy::UnitVariant => true,
            OwnedSdmTy::NewtypeStruct(owned_named_type) => is_prim(&owned_named_type.ty),
            OwnedSdmTy::NewtypeVariant(owned_named_type) => is_prim(&owned_named_type.ty),
            OwnedSdmTy::Seq(_) => false,
            OwnedSdmTy::Tuple(_) => false,
            OwnedSdmTy::TupleStruct(vec) => vec.iter().all(|e| is_prim(&e.ty)),
            OwnedSdmTy::TupleVariant(vec) => vec.iter().all(|e| is_prim(&e.ty)),
            OwnedSdmTy::Map { key, val } => is_prim(&key.ty) && is_prim(&val.ty),
            OwnedSdmTy::Struct(_) => false,
            OwnedSdmTy::StructVariant(_) => false,
            OwnedSdmTy::Enum(_) => false,
            OwnedSdmTy::Schema => true,
        }
    }

    /// Format an [`OwnedNamedType`] to the given string.
    ///
    /// Use `top_level = true` when this is a standalone type, and `top_level = false`
    /// when this type is contained within another type
    pub fn fmt_owned_nt_to_buf(ont: &OwnedNamedType, buf: &mut String, top_level: bool) {
        match &ont.ty {
            OwnedSdmTy::Bool => *buf += "bool",
            OwnedSdmTy::I8 => *buf += "i8",
            OwnedSdmTy::U8 => *buf += "u8",
            OwnedSdmTy::I16 => *buf += "i16",
            OwnedSdmTy::I32 => *buf += "i32",
            OwnedSdmTy::I64 => *buf += "i64",
            OwnedSdmTy::I128 => *buf += "i128",
            OwnedSdmTy::U16 => *buf += "u16",
            OwnedSdmTy::U32 => *buf += "u32",
            OwnedSdmTy::U64 => *buf += "u64",
            OwnedSdmTy::U128 => *buf += "u128",
            OwnedSdmTy::Usize => *buf += "usize",
            OwnedSdmTy::Isize => *buf += "isize",
            OwnedSdmTy::F32 => *buf += "f32",
            OwnedSdmTy::F64 => *buf += "f64",
            OwnedSdmTy::Char => *buf += "char",
            OwnedSdmTy::String => *buf += "String",
            OwnedSdmTy::ByteArray => *buf += "[u8]",
            OwnedSdmTy::Option(owned_named_type) => {
                *buf += "Option<";
                fmt_owned_nt_to_buf(owned_named_type, buf, false);
                *buf += ">";
            }
            OwnedSdmTy::Unit => *buf += "()",
            OwnedSdmTy::UnitStruct => {
                if top_level {
                    *buf += "struct ";
                }
                *buf += &ont.name;
            }
            OwnedSdmTy::NewtypeStruct(owned_named_type) => {
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
            OwnedSdmTy::Seq(owned_named_type) => {
                *buf += "[";
                *buf += &owned_named_type.name;
                *buf += "]";
            }
            OwnedSdmTy::Tuple(vec) => {
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
            OwnedSdmTy::TupleStruct(vec) => {
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
            OwnedSdmTy::Map { key, val } => {
                *buf += "Map<";
                *buf += &key.name;
                *buf += ", ";
                *buf += &val.name;
                *buf += ">";
            }
            OwnedSdmTy::Struct(vec) => {
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
            OwnedSdmTy::Enum(vec) => {
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
                                OwnedSdmTy::UnitVariant => {}
                                OwnedSdmTy::NewtypeVariant(owned_named_type) => {
                                    buf += "(";
                                    fmt_owned_nt_to_buf(owned_named_type, &mut buf, false);
                                    buf += ")";
                                }
                                OwnedSdmTy::TupleVariant(vec) => {
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
                                OwnedSdmTy::StructVariant(vec) => {
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
                                _ => unreachable!(),
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
            OwnedSdmTy::Schema => *buf += "Schema",

            // We only handle variants as part of an enum
            OwnedSdmTy::UnitVariant => unreachable!(),
            OwnedSdmTy::NewtypeVariant(_) => unreachable!(),
            OwnedSdmTy::TupleVariant(_) => unreachable!(),
            OwnedSdmTy::StructVariant(_) => unreachable!(),
        }
    }

    /// Collect unique types mentioned by this [`OwnedNamedType`]
    #[cfg(feature = "use-std")]
    pub fn discover_tys(ont: &OwnedNamedType, set: &mut std::collections::HashSet<OwnedNamedType>) {
        set.insert(ont.clone());
        discover_tys_sdm(&ont.ty, set);
    }

    /// Collect unique types mentioned by this [`OwnedSdmTy`]
    #[cfg(feature = "use-std")]
    pub fn discover_tys_sdm(sdm: &OwnedSdmTy, set: &mut std::collections::HashSet<OwnedNamedType>) {
        use crate::Schema;
        use core::ops::Deref;
        match sdm {
            OwnedSdmTy::Bool => set.insert(bool::SCHEMA.into()),
            OwnedSdmTy::I8 => set.insert(i8::SCHEMA.into()),
            OwnedSdmTy::U8 => set.insert(u8::SCHEMA.into()),
            OwnedSdmTy::I16 => set.insert(i16::SCHEMA.into()),
            OwnedSdmTy::I32 => set.insert(i32::SCHEMA.into()),
            OwnedSdmTy::I64 => set.insert(i64::SCHEMA.into()),
            OwnedSdmTy::I128 => set.insert(i128::SCHEMA.into()),
            OwnedSdmTy::U16 => set.insert(u16::SCHEMA.into()),
            OwnedSdmTy::U32 => set.insert(u32::SCHEMA.into()),
            OwnedSdmTy::U64 => set.insert(u64::SCHEMA.into()),
            OwnedSdmTy::U128 => set.insert(u128::SCHEMA.into()),

            // TODO: usize and isize don't impl Schema, which, fair.
            OwnedSdmTy::Usize => unreachable!(),
            OwnedSdmTy::Isize => unreachable!(),
            //

            OwnedSdmTy::F32 => set.insert(f32::SCHEMA.into()),
            OwnedSdmTy::F64 => set.insert(f64::SCHEMA.into()),
            OwnedSdmTy::Char => set.insert(char::SCHEMA.into()),
            OwnedSdmTy::String => set.insert(String::SCHEMA.into()),
            OwnedSdmTy::ByteArray => set.insert(<[u8]>::SCHEMA.into()),
            OwnedSdmTy::Option(owned_named_type) => set.insert(owned_named_type.deref().clone()),
            OwnedSdmTy::Unit => set.insert(<()>::SCHEMA.into()),
            OwnedSdmTy::UnitStruct => false,
            OwnedSdmTy::UnitVariant => false,
            OwnedSdmTy::NewtypeStruct(owned_named_type) => {
                set.insert(owned_named_type.deref().clone())
            }
            OwnedSdmTy::NewtypeVariant(owned_named_type) => {
                set.insert(owned_named_type.deref().clone())
            }
            OwnedSdmTy::Seq(owned_named_type) => set.insert(owned_named_type.deref().clone()),
            OwnedSdmTy::Tuple(vec) | OwnedSdmTy::TupleStruct(vec) => {
                for v in vec.iter() {
                    discover_tys_sdm(&v.ty, set);
                }
                false
            }
            OwnedSdmTy::TupleVariant(vec) => {
                for v in vec.iter() {
                    discover_tys(v, set);
                }
                false
            }
            OwnedSdmTy::Map { key, val } => {
                set.insert(key.deref().clone());
                set.insert(val.deref().clone());
                false
            }
            OwnedSdmTy::Struct(vec) | OwnedSdmTy::StructVariant(vec) => {
                for v in vec.iter() {
                    discover_tys(&v.ty, set);
                }
                false
            }
            OwnedSdmTy::Enum(vec) => {
                for v in vec.iter() {
                    discover_tys_sdm(&v.ty, set);
                }
                false
            }
            OwnedSdmTy::Schema => todo!(),
        };
    }
}
