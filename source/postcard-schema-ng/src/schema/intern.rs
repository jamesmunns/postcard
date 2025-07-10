#![allow(missing_docs, dead_code)]

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct InternStrRef {
    offset: usize,
    len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct InternDataModelTypeRef {
    idx: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct InternDataModelGroupRef {
    offset: usize,
    len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct InternNamedFieldGroupRef {
    offset: usize,
    len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct InternVariantGroupRef {
    offset: usize,
    len: usize,
}

/// This enum lists which of the Data Model Types apply to a given type. This describes how the
/// type is encoded on the wire.
///
/// This enum contains all Serde Data Model types as well as a "Schema" Type,
/// which corresponds to [`DataModelType`] itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum InternDataModelType {
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

    /// The `f64` Serde Data Model Type
    F64,

    /// The `char` Serde Data Model Type
    Char,

    /// The `String` Serde Data Model Type
    String,

    /// The `&[u8]` Serde Data Model Type
    ByteArray,

    /// The `Option<T>` Serde Data Model Type
    Option(InternDataModelTypeRef),

    /// The `()` Serde Data Model Type
    Unit,

    /// The "Sequence" Serde Data Model Type
    Seq(InternDataModelTypeRef),

    /// The "Tuple" Serde Data Model Type
    Tuple(InternDataModelGroupRef),

    /// The Array type: Not part of the Serde Data Model
    ///
    /// This is for fixed length arrays like [T; N], in earlier
    /// versions of the schema, we mapped this to a Tuple, which
    /// worked but makes schemas unfortunately long.
    Array {
        /// The array element's type
        item: InternDataModelTypeRef,
        /// The number of items in the fixed size array
        count: usize,
    },

    /// The "Map" Serde Data Model Type
    Map {
        /// The map "Key" type
        key: InternDataModelTypeRef,
        /// The map "Value" type
        val: InternDataModelTypeRef,
    },

    /// One of the struct Serde Data Model types
    Struct {
        /// The name of this struct
        name: InternStrRef,
        /// The data contained in this struct
        data: Data,
    },

    /// The "Enum" Serde Data Model Type (which contains any of the "Variant" types)
    Enum {
        /// The name of this struct
        name: InternStrRef,
        /// The variants contained in this enum
        variants: InternVariantGroupRef,
    },

    /// A [`DataModelType`]/[`OwnedDataModelType`](owned::OwnedDataModelType)
    Schema,
}

/// The contents of a struct or enum variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum Data {
    /// The "Unit Struct" or "Unit Variant" Serde Data Model Type
    Unit,

    /// The "Newtype Struct" or "Newtype Variant" Serde Data Model Type
    Newtype(InternDataModelTypeRef),

    /// The "Tuple Struct" or "Tuple Variant" Serde Data Model Type
    Tuple(InternDataModelGroupRef),

    /// The "Struct" or "Struct Variant" Serde Data Model Type
    Struct(InternNamedFieldGroupRef),
}

// TODO:
//
// * First populate the tuples, they need to have types in groups
//   * We might want to keep the "tuple list" and "other items" in separate lists
//   * we can merge

/// This represents a named struct field.
///
/// For example, in `struct Ex { a: u32 }` the field `a` would be reflected as
/// `NamedField { name: "a", ty: DataModelType::U32 }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct InternNamedField {
    /// The name of this field
    pub name: InternStrRef,
    /// The type of this field
    pub ty: InternDataModelTypeRef,
}

/// An enum variant e.g. `T::Bar(...)`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct InternVariant {
    /// The name of this variant
    pub name: InternStrRef,
    /// The data contained in this variant
    pub data: Data,
}

pub struct IntermedSchema<
    'a,
    const DMTS: usize,
    const RUNDMTS: usize,
    const NFS: usize,
    const VNTS: usize,
> {
    strs: &'a str,
    run_dmts: [InternDataModelType; RUNDMTS],
    dmts: [InternDataModelType; DMTS],
    nfs: [InternNamedField; NFS],
    vnts: [InternVariant; VNTS],
}

pub struct InternedSchema<'a, const DMTS: usize, const NFS: usize, const VNTS: usize> {
    strs: &'a str,
    dmts: [InternDataModelType; DMTS],
    nfs: [InternNamedField; NFS],
    vnts: [InternVariant; VNTS],
}

pub mod ty_intern {

}

pub mod str_intern {
    use crate::schema::{Data, DataModelType};

    pub const fn collect_all_strings_data<const N: usize>(
        data: &Data,
    ) -> ([&'static str; N], usize) {
        let mut out = const { [""; N] };
        match data {
            Data::Unit => (out, 0),
            Data::Newtype(data_model_type) => collect_all_strings(data_model_type),
            Data::Tuple(data_model_types) => {
                let mut count = 0;
                let mut idx = 0;
                while idx < data_model_types.len() {
                    let (sub, subct) = collect_all_strings::<N>(data_model_types[idx]);
                    let mut icount = 0;
                    while icount < subct {
                        out[count] = sub[icount];
                        count += 1;
                        icount += 1;
                    }
                    idx += 1;
                }
                (out, count)
            }
            Data::Struct(named_fields) => {
                let mut count = 0;
                let mut idx = 0;
                while idx < named_fields.len() {
                    out[count] = named_fields[idx].name;
                    count += 1;

                    let (sub, subct) = collect_all_strings::<N>(named_fields[idx].ty);
                    let mut icount = 0;
                    while icount < subct {
                        out[count] = sub[icount];
                        count += 1;
                        icount += 1;
                    }
                    idx += 1;
                }
                (out, count)
            }
        }
    }

    pub const fn collect_all_strings<const N: usize>(
        data: &DataModelType,
    ) -> ([&'static str; N], usize) {
        let mut out = const { [""; N] };
        match data {
            DataModelType::Bool => (out, 0),
            DataModelType::I8 => (out, 0),
            DataModelType::U8 => (out, 0),
            DataModelType::I16 => (out, 0),
            DataModelType::I32 => (out, 0),
            DataModelType::I64 => (out, 0),
            DataModelType::I128 => (out, 0),
            DataModelType::U16 => (out, 0),
            DataModelType::U32 => (out, 0),
            DataModelType::U64 => (out, 0),
            DataModelType::U128 => (out, 0),
            DataModelType::Usize => (out, 0),
            DataModelType::Isize => (out, 0),
            DataModelType::F32 => (out, 0),
            DataModelType::F64 => (out, 0),
            DataModelType::Char => (out, 0),
            DataModelType::String => (out, 0),
            DataModelType::ByteArray => (out, 0),
            DataModelType::Option(data_model_type) => collect_all_strings(data_model_type),
            DataModelType::Unit => (out, 0),
            DataModelType::Seq(data_model_type) => collect_all_strings(data_model_type),
            DataModelType::Tuple(data_model_types) => {
                let mut count = 0;
                let mut idx = 0;
                while idx < data_model_types.len() {
                    let (sub, subct) = collect_all_strings::<N>(data_model_types[idx]);
                    let mut icount = 0;
                    while icount < subct {
                        out[count] = sub[icount];
                        count += 1;
                        icount += 1;
                    }
                    idx += 1;
                }
                (out, count)
            }
            DataModelType::Array { item, count: _ } => collect_all_strings(item),
            DataModelType::Map { key, val } => {
                // collect_all_strings(key) + collect_all_strings(val)
                let mut count = 0;

                let (sub, subct) = collect_all_strings::<N>(key);
                let mut icount = 0;
                while icount < subct {
                    out[count] = sub[icount];
                    count += 1;
                    icount += 1;
                }

                let (sub, subct) = collect_all_strings::<N>(val);
                let mut icount = 0;
                while icount < subct {
                    out[count] = sub[icount];
                    count += 1;
                    icount += 1;
                }

                (out, count)
            }
            DataModelType::Struct { name, data } => {
                let mut count = 0;

                out[count] = name;
                count += 1;

                let (sub, subct) = collect_all_strings_data::<N>(data);
                let mut icount = 0;
                while icount < subct {
                    out[count] = sub[icount];
                    count += 1;
                    icount += 1;
                }
                (out, count)
            }
            DataModelType::Enum { name, variants } => {
                let mut count = 0;
                let mut idx = 0;

                out[count] = name;
                count += 1;

                while idx < variants.len() {
                    out[count] = variants[idx].name;
                    count += 1;

                    let (sub, subct) = collect_all_strings_data::<N>(&variants[idx].data);
                    let mut icount = 0;
                    while icount < subct {
                        out[count] = sub[icount];
                        count += 1;
                        icount += 1;
                    }
                    idx += 1;
                }
                (out, count)
            }
            DataModelType::Schema => todo!(),
        }
    }

    pub const fn count_strings_data(data: &Data) -> usize {
        match data {
            Data::Unit => 0,
            Data::Newtype(data_model_type) => count_strings(data_model_type),
            Data::Tuple(data_model_types) => {
                let mut count = 0;
                let mut idx = 0;
                while idx < data_model_types.len() {
                    count += count_strings(data_model_types[idx]);
                    idx += 1;
                }
                count
            }
            Data::Struct(named_fields) => {
                let mut count = 0;
                let mut idx = 0;
                while idx < named_fields.len() {
                    count += 1; // name
                    count += count_strings(named_fields[idx].ty);
                    idx += 1;
                }
                count
            }
        }
    }

    pub const fn count_strings(data: &DataModelType) -> usize {
        match data {
            DataModelType::Bool => 0,
            DataModelType::I8 => 0,
            DataModelType::U8 => 0,
            DataModelType::I16 => 0,
            DataModelType::I32 => 0,
            DataModelType::I64 => 0,
            DataModelType::I128 => 0,
            DataModelType::U16 => 0,
            DataModelType::U32 => 0,
            DataModelType::U64 => 0,
            DataModelType::U128 => 0,
            DataModelType::Usize => 0,
            DataModelType::Isize => 0,
            DataModelType::F32 => 0,
            DataModelType::F64 => 0,
            DataModelType::Char => 0,
            DataModelType::String => 0,
            DataModelType::ByteArray => 0,
            DataModelType::Option(data_model_type) => count_strings(data_model_type),
            DataModelType::Unit => 0,
            DataModelType::Seq(data_model_type) => count_strings(data_model_type),
            DataModelType::Tuple(data_model_types) => {
                let mut count = 0;
                let mut idx = 0;
                while idx < data_model_types.len() {
                    count += count_strings(data_model_types[idx]);
                    idx += 1;
                }
                count
            }
            DataModelType::Array { item, count: _ } => count_strings(item),
            DataModelType::Map { key, val } => count_strings(key) + count_strings(val),
            DataModelType::Struct { name: _, data } => {
                // 1 for name
                1 + count_strings_data(data)
            }
            DataModelType::Enum { name: _, variants } => {
                // 1 for enum name
                let mut count = 1;
                let mut idx = 0;
                while idx < variants.len() {
                    // 1 for variant name
                    count += 1;
                    count += count_strings_data(&variants[idx].data);
                    idx += 1;
                }
                count
            }
            DataModelType::Schema => todo!(),
        }
    }

    pub const fn sort_arr<const N: usize>(mut arr: [&str; N]) -> [&str; N] {
        if N < 2 {
            return arr;
        }
        loop {
            let mut swapped = false;
            let mut i = 1;
            while i < arr.len() {
                // First, sort by len, longest first
                let swap = if arr[i - 1].len() < arr[i].len() {
                    true
                } else if arr[i - 1].len() == arr[i].len() {
                    // Then, if equal length, sort alphabetically
                    let mut swap = false;
                    let mut j = 0;
                    'inner: while j < arr[i].len() {
                        if arr[i - 1].as_bytes()[j] > arr[i].as_bytes()[j] {
                            swap = true;
                            break 'inner;
                        } else if arr[i - 1].as_bytes()[j] < arr[i].as_bytes()[j] {
                            break 'inner;
                        }
                        j += 1;
                    }
                    swap
                } else {
                    false
                };

                if swap {
                    let left = arr[i - 1];
                    let right = arr[i];
                    arr[i - 1] = right;
                    arr[i] = left;
                    swapped = true;
                }

                i += 1;
            }

            if !swapped {
                break;
            }
        }
        arr
    }

    pub const fn streq(a: &str, b: &str) -> bool {
        if a.len() != b.len() {
            return false;
        }
        let a = a.as_bytes();
        let b = b.as_bytes();

        let mut i = 0;
        while i < a.len() {
            if a[i] != b[i] {
                return false;
            }
            i += 1;
        }
        true
    }

    pub const fn str_contains(base: &str, frag: &str) -> bool {
        if frag.len() > base.len() {
            return false;
        }
        if streq(base, frag) {
            return true;
        }
        if frag.is_empty() {
            return true;
        }
        let base = base.as_bytes();
        let frag = frag.as_bytes();

        let mut bi = 0;
        'outer: while bi < (base.len() - frag.len()) {
            if base[bi] == frag[0] {
                let mut off = 1;
                while (off < frag.len()) && ((off + bi) < base.len()) {
                    if frag[off] != base[off + bi] {
                        bi += 1;
                        continue 'outer;
                    }
                    off += 1;
                }
                return true;
            }
            bi += 1;
        }
        false
    }

    pub const fn dedupe_strings<const N: usize>(strs: [&str; N]) -> ([&str; N], usize) {
        let strs = sort_arr(strs);
        let mut out = [""; N];
        let mut count = 0;
        let mut idx = 0;
        'outer: while idx < strs.len() {
            let mut icount = 0;
            while icount < count {
                if streq(out[icount], strs[idx]) {
                    // exact match
                    idx += 1;
                    continue 'outer;
                } else if str_contains(out[icount], strs[idx]) {
                    // fragment match
                    idx += 1;
                    continue 'outer;
                }
                icount += 1;
            }
            // if we reach the end, no match was found.
            out[count] = strs[idx];
            count += 1;
            idx += 1;
        }

        (out, count)
    }

    pub const fn pack_down_str<const N: usize>(strs: &[&'static str]) -> [&'static str; N] {
        let mut out = [""; N];
        let mut idx = 0;
        while idx < N {
            out[idx] = strs[idx];
            idx += 1;
        }
        out
    }

    pub const fn ttl_str_len(strs: &[&str]) -> usize {
        let mut total = 0;
        let mut idx = 0;
        while idx < strs.len() {
            total += strs[idx].len();
            idx += 1;
        }
        total
    }

    pub const fn merge_strs<const N: usize>(strs: &[&str]) -> [u8; N] {
        let mut out = [0u8; N];
        let mut count = 0;
        let mut idx = 0;
        while idx < strs.len() {
            let s = strs[idx].as_bytes();
            let mut icount = 0;
            while icount < s.len() {
                out[count] = s[icount];
                icount += 1;
                count += 1;
            }
            idx += 1;
        }
        assert!(count == N);
        out
    }

    #[macro_export]
    macro_rules! sintern {
        ($schema_ty:ty) => {
            const {
                // How many un-de-duplicated strings exist in the schema?
                const CT: usize = $crate::schema::intern::str_intern::count_strings(
                    <$schema_ty as postcard_schema_ng::Schema>::SCHEMA,
                );
                // A tuple of strings, and the number actually used (should be the same?)
                const STRS: ([&str; CT], usize) =
                    $crate::schema::intern::str_intern::collect_all_strings::<CT>(
                        <$schema_ty as postcard_schema_ng::Schema>::SCHEMA,
                    );
                // A tuple of de-duplicated and sorted strings
                const DSTRS: ([&str; CT], usize) =
                    $crate::schema::intern::str_intern::dedupe_strings(STRS.0);
                // The post-de-duplication len
                const LEN: usize = DSTRS.1;
                // An array of strs
                const PSTRS: &[&str] =
                    &$crate::schema::intern::str_intern::pack_down_str::<LEN>(&DSTRS.0);
                // The total len (in bytes) of the strs
                const LLEN: usize = $crate::schema::intern::str_intern::ttl_str_len(PSTRS);
                // The strs, merged into a single array
                const MERGED: &[u8] =
                    &$crate::schema::intern::str_intern::merge_strs::<LLEN>(PSTRS);
                // And finally, turn it back into a str
                match core::str::from_utf8(MERGED) {
                    Ok(s) => s,
                    Err(_e) => panic!(),
                }
            }
        };
    }
}
