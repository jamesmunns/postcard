//! .

use crate::{
    const_helpers::nty_eq, schema::{DataModelType, DataModelVariant, NamedType}, Schema
};

/// Calculate the max size of a type that impls Schema
///
/// This number must NOT be relied on for safety purposes
/// (such as unchecked access), as manual schema impls can
/// be wrong.
pub const fn max_size<T: Schema>() -> Option<usize> {
    if let Some(sz) = T::MANUAL_MAX_SIZE {
        Some(sz)
    } else {
        max_size_nt(T::SCHEMA)
    }
}

/// Calculate the max size of a NamedType
pub const fn max_size_nt(nt: &NamedType) -> Option<usize> {
    max_size_dmt(nt.ty)
}

/// .
pub const fn bounded_seq_max<Outer: Schema, Inner: Schema, const N: usize>() -> Option<usize> {
    let DataModelType::Seq(nt) = Outer::SCHEMA.ty else {
        panic!("Not a bounded seq!");
    };
    assert!(nty_eq(nt, Inner::SCHEMA), "Mismatched Outer/Inner types!");
    let size_one = max_size::<Inner>();
    if let Some(sz) = size_one {
        let data_sz = sz * N;
        let varint_sz = size_as_varint_usize(N);
        Some(data_sz + varint_sz)
    } else {
        None
    }
}

/// .
pub const fn bounded_string_max<const N: usize>() -> Option<usize> {
    // Measured in bytes
    let data_sz = N;
    let varint_sz = size_as_varint_usize(N);
    Some(data_sz + varint_sz)
}

/// Calculate the size (in bytes) it would take to store this
/// usize as a varint.
pub const fn size_as_varint_usize(n: usize) -> usize {
    if n == 0 {
        return 1;
    }
    let ttl_bits = usize::BITS as usize;
    let ldg_bits = n.leading_zeros() as usize;
    let used_bits = ttl_bits - ldg_bits;
    (used_bits + 6) / 7
}

/// Calculate the max size of a DataModelType
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
        }
        DataModelType::Map { .. } => None,
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
        }
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
                    }
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
                    }
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
                    }
                };

                if sz > max {
                    max = sz;
                }

                i += 1;
            }
            let disc_size = if nvars.is_empty() {
                1
            } else {
                // We need the size of the largest variant ID. This is the (len - 1),
                // because if we have one variant, its discriminant will be zero.
                // We already checked above that len != 0.
                size_as_varint_usize(nvars.len() - 1)
            };
            // discriminants are `varint(u32)`
            Some(max + disc_size)
        }
        DataModelType::Schema => None,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn savu() {
        assert_eq!(1, size_as_varint_usize(0x00));
        assert_eq!(1, size_as_varint_usize(0x7F));
        assert_eq!(2, size_as_varint_usize(0x80));
        assert_eq!(2, size_as_varint_usize(0x3FFF));
        assert_eq!(3, size_as_varint_usize(0x4000));
        assert_eq!(3, size_as_varint_usize(0xFFFF));

        #[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
        let _: () = {
            assert_eq!(3, size_as_varint_usize(0x1FFFFF));
            assert_eq!(4, size_as_varint_usize(0x200000));
            assert_eq!(4, size_as_varint_usize(0xFFFFFFF));
            assert_eq!(5, size_as_varint_usize(0x10000000));
            assert_eq!(5, size_as_varint_usize(0xFFFFFFFF));
        };

        #[cfg(target_pointer_width = "64")]
        let _: () = {
            assert_eq!(5, size_as_varint_usize(0x7FFFFFFFF));
            assert_eq!(6, size_as_varint_usize(0x800000000));
            assert_eq!(10, size_as_varint_usize(usize::MAX));
        };
    }
}
