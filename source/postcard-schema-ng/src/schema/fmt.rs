//! Formatting helper functionality
//!
//! This module provides ways of turning Data Model information into a human
//! readable output

#[cfg(all(not(feature = "use-std"), feature = "alloc"))]
extern crate alloc;

use super::owned::{OwnedData, OwnedDataModelType};

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
        OwnedDataModelType::Option(ty) => is_prim(ty),
        OwnedDataModelType::Unit => true,
        OwnedDataModelType::Seq(_) => false,
        OwnedDataModelType::Tuple(_) => false,
        OwnedDataModelType::Array { item, count: _ } => is_prim(item),
        OwnedDataModelType::Map { key, val } => is_prim(key) && is_prim(val),
        OwnedDataModelType::Struct { .. } => false,
        OwnedDataModelType::Enum { .. } => false,
        OwnedDataModelType::Schema => true,
    }
}

/// Format an [`OwnedDataModelType`] to the given string.
///
/// Use `top_level = true` when this is a standalone type, and `top_level = false`
/// when this type is contained within another type
pub fn fmt_owned_dmt_to_buf(dmt: &OwnedDataModelType, buf: &mut String, top_level: bool) {
    let fmt_data = |data: &OwnedData, buf: &mut String| match data {
        OwnedData::Unit => {}
        OwnedData::Newtype(inner) => {
            *buf += "(";
            fmt_owned_dmt_to_buf(inner, buf, false);
            *buf += ")";
        }
        OwnedData::Tuple(fields) => {
            *buf += "(";
            let mut fields = fields.iter();
            if let Some(first) = fields.next() {
                fmt_owned_dmt_to_buf(first, buf, false);
            }
            for field in fields {
                *buf += ", ";
                fmt_owned_dmt_to_buf(field, buf, false);
            }
            *buf += ")";
        }
        OwnedData::Struct(fields) => {
            *buf += " { ";
            let mut fields = fields.iter();
            if let Some(first) = fields.next() {
                *buf += &first.name;
                *buf += ": ";
                fmt_owned_dmt_to_buf(&first.ty, buf, false);
            }
            for field in fields {
                *buf += ", ";
                *buf += &field.name;
                *buf += ": ";
                fmt_owned_dmt_to_buf(&field.ty, buf, false);
            }
            *buf += " }";
        }
    };

    match dmt {
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
        OwnedDataModelType::Option(ty) => {
            *buf += "Option<";
            fmt_owned_dmt_to_buf(ty, buf, false);
            *buf += ">";
        }
        OwnedDataModelType::Unit => *buf += "()",
        OwnedDataModelType::Seq(ty) => {
            *buf += "[";
            fmt_owned_dmt_to_buf(ty, buf, false);
            *buf += "]";
        }
        OwnedDataModelType::Tuple(vec) => {
            if !vec.is_empty() {
                let first = &vec[0];
                if vec.iter().all(|v| first == v) {
                    // This is a fixed size array
                    *buf += "[";
                    fmt_owned_dmt_to_buf(first, buf, false);
                    *buf += "; ";
                    *buf += &format!("{}", vec.len());
                    *buf += "]";
                } else {
                    *buf += "(";
                    let fields = vec
                        .iter()
                        .map(|v| {
                            let mut buf = String::new();
                            fmt_owned_dmt_to_buf(v, &mut buf, false);
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
        OwnedDataModelType::Array { item, count } => {
            *buf += "[";
            fmt_owned_dmt_to_buf(item, buf, false);
            *buf += "; ";
            *buf += &format!("{count}");
            *buf += "]";
        }
        OwnedDataModelType::Map { key, val } => {
            *buf += "Map<";
            fmt_owned_dmt_to_buf(key, buf, false);
            *buf += ", ";
            fmt_owned_dmt_to_buf(val, buf, false);
            *buf += ">";
        }
        OwnedDataModelType::Struct { name, data } => {
            if top_level {
                *buf += "struct ";
                *buf += name;
                fmt_data(data, buf);
            } else {
                *buf += name;
            }
        }
        OwnedDataModelType::Enum { name, variants } => {
            if top_level {
                *buf += "enum ";
                *buf += name;
                *buf += " { ";

                let fields = variants
                    .iter()
                    .map(|v| {
                        let mut buf = String::new();
                        buf += &v.name;
                        fmt_data(&v.data, &mut buf);
                        buf
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                *buf += &fields;
                *buf += " }";
            } else {
                *buf += name;
            }
        }
        OwnedDataModelType::Schema => *buf += "Schema",
    }
}

/// Collect unique types mentioned by this [`OwnedDataModelType`]
#[cfg(feature = "use-std")]
pub fn discover_tys(
    ty: &OwnedDataModelType,
    set: &mut std::collections::HashSet<OwnedDataModelType>,
) {
    let discover_tys_data = |data: &OwnedData, set: &mut _| match data {
        OwnedData::Unit => {}
        OwnedData::Newtype(inner) => discover_tys(inner, set),
        OwnedData::Tuple(elements) => {
            for element in elements {
                discover_tys(element, set)
            }
        }
        OwnedData::Struct(fields) => {
            for field in fields {
                discover_tys(&field.ty, set)
            }
        }
    };

    set.insert(ty.clone());
    match ty {
        OwnedDataModelType::Bool => {}
        OwnedDataModelType::I8 => {}
        OwnedDataModelType::U8 => {}
        OwnedDataModelType::I16 => {}
        OwnedDataModelType::I32 => {}
        OwnedDataModelType::I64 => {}
        OwnedDataModelType::I128 => {}
        OwnedDataModelType::U16 => {}
        OwnedDataModelType::U32 => {}
        OwnedDataModelType::U64 => {}
        OwnedDataModelType::U128 => {}

        // TODO: usize and isize don't impl Schema, which, fair.
        OwnedDataModelType::Usize => unreachable!(),
        OwnedDataModelType::Isize => unreachable!(),
        //
        OwnedDataModelType::F32 => {}
        OwnedDataModelType::F64 => {}
        OwnedDataModelType::Char => {}
        OwnedDataModelType::String => {}
        OwnedDataModelType::ByteArray => {}
        OwnedDataModelType::Option(inner) => {
            discover_tys(inner, set);
        }
        OwnedDataModelType::Unit => {}
        OwnedDataModelType::Seq(elements) => {
            discover_tys(elements, set);
        }
        OwnedDataModelType::Tuple(vec) => {
            for v in vec.iter() {
                discover_tys(v, set);
            }
        }
        OwnedDataModelType::Array { item, count: _ } => {
            discover_tys(item, set);
        }
        OwnedDataModelType::Map { key, val } => {
            discover_tys(key, set);
            discover_tys(val, set);
        }
        OwnedDataModelType::Struct { name: _, data } => {
            discover_tys_data(data, set);
        }
        OwnedDataModelType::Enum { name: _, variants } => {
            for variant in variants {
                discover_tys_data(&variant.data, set);
            }
        }
        OwnedDataModelType::Schema => todo!(),
    };
}
