//! Const shims for operating on schema types

//////////////////////////////////////////////////////////////////////////////
// STAGE 0 - HELPERS
//////////////////////////////////////////////////////////////////////////////

use crate::schema::{DataModelType, DataModelVariant, NamedType, NamedValue, NamedVariant};

/// `is_prim` returns whether the type is a *primitive*, or a built-in type that
/// does not need to be sent over the wire.
pub const fn is_prim(dmt: &DataModelType) -> bool {
    match dmt {
        // These are all primitives
        DataModelType::Bool => true,
        DataModelType::I8 => true,
        DataModelType::U8 => true,
        DataModelType::I16 => true,
        DataModelType::I32 => true,
        DataModelType::I64 => true,
        DataModelType::I128 => true,
        DataModelType::U16 => true,
        DataModelType::U32 => true,
        DataModelType::U64 => true,
        DataModelType::U128 => true,
        DataModelType::Usize => true,
        DataModelType::Isize => true,
        DataModelType::F32 => true,
        DataModelType::F64 => true,
        DataModelType::Char => true,
        DataModelType::String => true,
        DataModelType::ByteArray => true,
        DataModelType::Unit => true,
        DataModelType::Schema => true,

        // A unit-struct is always named, so it is not primitive, as the
        // name has meaning even without a value
        DataModelType::UnitStruct => false,
        // Items with subtypes are composite, and therefore not primitives, as
        // we need to convey this information.
        DataModelType::Option(_) | DataModelType::NewtypeStruct(_) | DataModelType::Seq(_) => false,
        DataModelType::Tuple(_) | DataModelType::TupleStruct(_) => false,
        DataModelType::Map { .. } => false,
        DataModelType::Struct(_) => false,
        DataModelType::Enum(_) => false,
    }
}

/// A const version of `<str as PartialEq>::eq`
pub const fn str_eq(a: &str, b: &str) -> bool {
    let mut i = 0;
    if a.len() != b.len() {
        return false;
    }
    let a_by = a.as_bytes();
    let b_by = b.as_bytes();
    while i < a.len() {
        if a_by[i] != b_by[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// A const version of `<NamedType as PartialEq>::eq`
pub const fn nty_eq(a: &NamedType, b: &NamedType) -> bool {
    str_eq(a.name, b.name) && dmt_eq(a.ty, b.ty)
}

/// A const version of `<[&NamedType] as PartialEq>::eq`
pub const fn ntys_eq(a: &[&NamedType], b: &[&NamedType]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if !nty_eq(a[i], b[i]) {
            return false;
        }
        i += 1;
    }
    true
}

/// A const version of `<DataModelType as PartialEq>::eq`
pub const fn dmt_eq(a: &DataModelType, b: &DataModelType) -> bool {
    match (a, b) {
        // Data model types are ONLY matching if they are both the same variant
        //
        // For primitives (and unit structs), we only check the discriminant matches.
        (DataModelType::Bool, DataModelType::Bool) => true,
        (DataModelType::I8, DataModelType::I8) => true,
        (DataModelType::U8, DataModelType::U8) => true,
        (DataModelType::I16, DataModelType::I16) => true,
        (DataModelType::I32, DataModelType::I32) => true,
        (DataModelType::I64, DataModelType::I64) => true,
        (DataModelType::I128, DataModelType::I128) => true,
        (DataModelType::U16, DataModelType::U16) => true,
        (DataModelType::U32, DataModelType::U32) => true,
        (DataModelType::U64, DataModelType::U64) => true,
        (DataModelType::U128, DataModelType::U128) => true,
        (DataModelType::Usize, DataModelType::Usize) => true,
        (DataModelType::Isize, DataModelType::Isize) => true,
        (DataModelType::F32, DataModelType::F32) => true,
        (DataModelType::F64, DataModelType::F64) => true,
        (DataModelType::Char, DataModelType::Char) => true,
        (DataModelType::String, DataModelType::String) => true,
        (DataModelType::ByteArray, DataModelType::ByteArray) => true,
        (DataModelType::Unit, DataModelType::Unit) => true,
        (DataModelType::UnitStruct, DataModelType::UnitStruct) => true,
        (DataModelType::Schema, DataModelType::Schema) => true,

        // For non-primitive types, we check whether all children are equivalent as well.
        (DataModelType::Option(nta), DataModelType::Option(ntb)) => nty_eq(nta, ntb),
        (DataModelType::NewtypeStruct(nta), DataModelType::NewtypeStruct(ntb)) => nty_eq(nta, ntb),
        (DataModelType::Seq(nta), DataModelType::Seq(ntb)) => nty_eq(nta, ntb),

        (DataModelType::Tuple(ntsa), DataModelType::Tuple(ntsb)) => ntys_eq(ntsa, ntsb),
        (DataModelType::TupleStruct(ntsa), DataModelType::TupleStruct(ntsb)) => ntys_eq(ntsa, ntsb),
        (
            DataModelType::Map {
                key: keya,
                val: vala,
            },
            DataModelType::Map {
                key: keyb,
                val: valb,
            },
        ) => nty_eq(keya, keyb) && nty_eq(vala, valb),
        (DataModelType::Struct(nvalsa), DataModelType::Struct(nvalsb)) => vals_eq(nvalsa, nvalsb),
        (DataModelType::Enum(nvarsa), DataModelType::Enum(nvarsb)) => vars_eq(nvarsa, nvarsb),

        // Any mismatches are not equal
        _ => false,
    }
}

/// A const version of `<NamedVariant as PartialEq>::eq`
pub const fn var_eq(a: &NamedVariant, b: &NamedVariant) -> bool {
    str_eq(a.name, b.name) && dmv_eq(a.ty, b.ty)
}

/// A const version of `<&[&NamedVariant] as PartialEq>::eq`
pub const fn vars_eq(a: &[&NamedVariant], b: &[&NamedVariant]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if !var_eq(a[i], b[i]) {
            return false;
        }
        i += 1;
    }
    true
}

/// A const version of `<&[&NamedValue] as PartialEq>::eq`
pub const fn vals_eq(a: &[&NamedValue], b: &[&NamedValue]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if !str_eq(a[i].name, b[i].name) {
            return false;
        }
        if !nty_eq(a[i].ty, b[i].ty) {
            return false;
        }

        i += 1;
    }
    true
}

/// A const version of `<DataModelVariant as PartialEq>::eq`
pub const fn dmv_eq(a: &DataModelVariant, b: &DataModelVariant) -> bool {
    match (a, b) {
        (DataModelVariant::UnitVariant, DataModelVariant::UnitVariant) => true,
        (DataModelVariant::NewtypeVariant(nta), DataModelVariant::NewtypeVariant(ntb)) => {
            nty_eq(nta, ntb)
        }
        (DataModelVariant::TupleVariant(ntsa), DataModelVariant::TupleVariant(ntsb)) => {
            ntys_eq(ntsa, ntsb)
        }
        (DataModelVariant::StructVariant(nvarsa), DataModelVariant::StructVariant(nvarsb)) => {
            vals_eq(nvarsa, nvarsb)
        }
        _ => false,
    }
}
