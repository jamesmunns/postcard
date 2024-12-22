//! Formatting helper functionality
//!
//! This module provides ways of turning Data Model information into a human
//! readable output

#[cfg(all(not(feature = "use-std"), feature = "alloc"))]
extern crate alloc;

use super::owned::{OwnedDataModelType, OwnedDataModelVariant, OwnedNamedType};

#[cfg(feature = "use-std")]
use std::{string::String, vec::Vec};

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
pub fn discover_tys_sdm(
    sdm: &OwnedDataModelType,
    set: &mut std::collections::HashSet<OwnedNamedType>,
) {
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
