//! .

use crate::{schema::{DataModelType, DataModelVariant, NamedType}, Schema};

pub const fn max_size<T: Schema>() -> Option<usize> {
    max_size_nt(T::SCHEMA)
}

pub const fn max_size_nt(nt: &NamedType) -> Option<usize> {
    max_size_dmt(nt.ty)
}

pub const fn max_size_dmt(dmt: &DataModelType) -> Option<usize> {
    match dmt {
        DataModelType::Bool => Some(1),
        DataModelType::I8 => Some(1),
        DataModelType::U8 => Some(1),
        DataModelType::I16 => Some(3),
        DataModelType::I32 => Some(5),
        DataModelType::I64 => Some(10),
        DataModelType::I128 => Some(19),
        DataModelType::U16 => Some(3),
        DataModelType::U32 => Some(5),
        DataModelType::U64 => Some(10),
        DataModelType::U128 => Some(19),
        DataModelType::Usize => None, // TODO: these don't impl schema and are platform dependent
        DataModelType::Isize => None, // TODO: these don't impl schema and are platform dependent
        DataModelType::F32 => Some(4),
        DataModelType::F64 => Some(8),
        DataModelType::Char => Some(5), // I think? 1 len + up to 4 bytes
        DataModelType::String => None,
        DataModelType::ByteArray => None,
        DataModelType::Option(nt) => max_size_nt(nt),
        DataModelType::Unit => Some(0),
        DataModelType::UnitStruct => Some(0),
        DataModelType::NewtypeStruct(nt) => max_size_nt(nt),
        DataModelType::Seq(_) => None,
        DataModelType::Tuple(nts) | DataModelType::TupleStruct(nts) => {
            let mut i = 0;
            let mut ct = 0;
            while i < nts.len() {
                let Some(sz) = max_size_nt(nts[i]) else {
                    return None;
                };
                ct += sz;
                i += 1;
            }
            Some(ct)
        },
        DataModelType::Map { key, val } => {
            let Some(sz1) = max_size_nt(key) else {
                return None;
            };
            let Some(sz2) = max_size_nt(val) else {
                return None;
            };
            Some(sz1 + sz2)
        },
        DataModelType::Struct(nvals) => {
            let mut i = 0;
            let mut ct = 0;
            while i < nvals.len() {
                let Some(sz) = max_size_dmt(nvals[i].ty.ty) else {
                    return None;
                };
                ct += sz;
                i += 1;
            }
            Some(ct)
        },
        DataModelType::Enum(nvars) => {
            let mut i = 0;
            let mut max = 0;
            while i < nvars.len() {
                let sz = match nvars[i].ty {
                    DataModelVariant::UnitVariant => 0,
                    DataModelVariant::NewtypeVariant(nt) => {
                        let Some(sz) = max_size_nt(nt) else {
                            return None;
                        };
                        sz
                    },
                    DataModelVariant::TupleVariant(nts) => {
                        let mut j = 0;
                        let mut ct = 0;
                        while j < nts.len() {
                            let Some(sz) = max_size_nt(nts[j]) else {
                                return None;
                            };
                            ct += sz;
                            j += 1;
                        }
                        ct
                    },
                    DataModelVariant::StructVariant(nvars) => {
                        let mut j = 0;
                        let mut ct = 0;
                        while j < nvars.len() {
                            let Some(sz) = max_size_dmt(nvars[j].ty.ty) else {
                                return None;
                            };
                            ct += sz;
                            j += 1;
                        }
                        ct
                    },
                };

                if sz > max {
                    max = sz;
                }

                i += 1;
            }
            // discriminants are `varint(u32)`
            Some(max + 5)
        },
        DataModelType::Schema => None,
    }
}


