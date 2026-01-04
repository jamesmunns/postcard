#![allow(missing_docs, dead_code)]

use serde::{ser::SerializeStruct, Serialize};
use str_intern::streq;
use ty_intern::str_subslice;

use super::{Data, DataModelType, NamedField, Variant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct InternStrRef {
    offset: usize,
    len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum InternDataModelTypeRef {
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

    /// The `()` Serde Data Model Type
    Unit,

    Ref(usize),

    /// A [`DataModelType`]/[`OwnedDataModelType`](owned::OwnedDataModelType)
    Schema,
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
        data: InternData,
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
pub enum InternData {
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
    pub data: InternData,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntermediateSchema<
    'a,
    const DMTS: usize,
    const RUNDMTS: usize,
    const NFS: usize,
    const VNTS: usize,
> {
    strs: &'a str,

    run_dmts: [InternDataModelType; RUNDMTS],
    run_dmts_len: usize,

    dmts: [InternDataModelType; DMTS],
    dmts_len: usize,

    nfs: [InternNamedField; NFS],
    nfs_len: usize,

    vnts: [InternVariant; VNTS],
    vnts_len: usize,
}

impl<'a, const DMTS: usize, const RUNDMTS: usize, const NFS: usize, const VNTS: usize> Serialize
    for IntermediateSchema<'a, DMTS, RUNDMTS, NFS, VNTS>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("IntermediateSchema", 5)?;
        s.serialize_field("strs", self.strs)?;
        s.serialize_field("run_dmts", self.run_dmts())?;
        s.serialize_field("dmts", self.dmts())?;
        s.serialize_field("nfs", self.nfs())?;
        s.serialize_field("vnts", self.vnts())?;
        s.end()
    }
}

impl<'a, const DMTS: usize, const RUNDMTS: usize, const NFS: usize, const VNTS: usize>
    core::fmt::Debug for IntermediateSchema<'a, DMTS, RUNDMTS, NFS, VNTS>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IntermediateSchema")
            .field("run_dmts", &self.run_dmts())
            .field("dmts", &self.dmts())
            .field("nfs", &self.nfs())
            .field("vnts", &self.vnts())
            .field("strs", &self.strs)
            .finish()
    }
}

impl<'a, const DMTS: usize, const RUNDMTS: usize, const NFS: usize, const VNTS: usize>
    IntermediateSchema<'a, DMTS, RUNDMTS, NFS, VNTS>
{
    pub const fn blammo(s: &'a str, dmt: &DataModelType) -> Self {
        let mut me = Self::new(s);
        _ = me.get_or_insert_interned_dmt(dmt);
        me
    }

    const fn new(s: &'a str) -> Self {
        const NULLSTR: InternStrRef = InternStrRef { offset: 0, len: 0 };
        Self {
            strs: s,
            run_dmts: [InternDataModelType::Schema; RUNDMTS],
            run_dmts_len: 0,
            dmts: [InternDataModelType::Schema; DMTS],
            dmts_len: 0,
            nfs: [InternNamedField {
                name: NULLSTR,
                ty: InternDataModelTypeRef::Schema,
            }; NFS],
            nfs_len: 0,
            vnts: [InternVariant {
                name: NULLSTR,
                data: InternData::Unit,
            }; VNTS],
            vnts_len: 0,
        }
    }

    pub const fn get_interned_str(&self, frag: &str) -> InternStrRef {
        if frag.len() > self.strs.len() {
            panic!()
        }
        if frag.is_empty() {
            panic!();
        }
        let strs = self.strs.as_bytes();
        let frag = frag.as_bytes();

        let mut bi = 0;
        'outer: while bi <= (strs.len() - frag.len()) {
            let mut off = 0;
            while (off < frag.len()) && ((off + bi) < strs.len()) {
                if frag[off] != strs[off + bi] {
                    bi += 1;
                    continue 'outer;
                }
                off += 1;
            }
            return InternStrRef {
                offset: bi,
                len: frag.len(),
            };
        }
        panic!()
    }

    pub const fn get_or_insert_nf_group(
        &mut self,
        nfs: &[&NamedField],
    ) -> InternNamedFieldGroupRef {
        if let Some(nfg) = self.find_named_field_group(nfs) {
            return nfg;
        }
        let mut count = 0;
        let start = self.nfs_len;
        while count < nfs.len() {
            self.nfs[self.nfs_len] = InternNamedField {
                name: self.get_interned_str(nfs[count].name),
                ty: self.get_or_insert_interned_dmt(nfs[count].ty),
            };
            self.nfs_len += 1;
            count += 1;
        }
        InternNamedFieldGroupRef {
            offset: start,
            len: nfs.len(),
        }
    }

    pub const fn get_or_insert_variant_group(
        &mut self,
        vnts: &[&Variant],
    ) -> InternVariantGroupRef {
        if let Some(vnt) = self.find_variant_group(vnts) {
            return vnt;
        }
        let mut count = 0;
        let start = self.vnts_len;
        while count < vnts.len() {
            self.vnts[self.vnts_len] = InternVariant {
                name: self.get_interned_str(vnts[count].name),
                data: self.get_or_insert_interned_data(&vnts[count].data),
            };
            self.vnts_len += 1;
            count += 1;
        }
        InternVariantGroupRef {
            offset: start,
            len: vnts.len(),
        }
    }

    pub const fn get_or_insert_run_dmt(
        &mut self,
        dmts: &[&DataModelType],
    ) -> InternDataModelGroupRef {
        if let Some(rdmts) = self.find_run_dmt(dmts) {
            return rdmts;
        }
        let mut count = 0;
        let start = self.run_dmts_len;
        while count < dmts.len() {
            let dr = self.get_or_insert_interned_dmt(dmts[count]);
            self.run_dmts[self.run_dmts_len] = self.dmt_from_ref(&dr);
            self.run_dmts_len += 1;
            count += 1;
        }
        InternDataModelGroupRef {
            offset: start,
            len: dmts.len(),
        }
    }

    pub const fn get_or_insert_interned_data(&mut self, data: &Data) -> InternData {
        match data {
            Data::Unit => InternData::Unit,
            Data::Newtype(data_model_type) => {
                let rf = self.get_or_insert_interned_dmt(data_model_type);
                InternData::Newtype(rf)
            }
            Data::Tuple(data_model_types) => {
                InternData::Tuple(self.get_or_insert_run_dmt(data_model_types))
            }
            Data::Struct(named_fields) => {
                InternData::Struct(self.get_or_insert_nf_group(named_fields))
            }
        }
    }

    pub const fn get_or_insert_interned_dmt(
        &mut self,
        dmt: &DataModelType,
    ) -> InternDataModelTypeRef {
        match dmt {
            DataModelType::Bool => return InternDataModelTypeRef::Bool,
            DataModelType::I8 => return InternDataModelTypeRef::I8,
            DataModelType::U8 => return InternDataModelTypeRef::U8,
            DataModelType::I16 => return InternDataModelTypeRef::I16,
            DataModelType::I32 => return InternDataModelTypeRef::I32,
            DataModelType::I64 => return InternDataModelTypeRef::I64,
            DataModelType::I128 => return InternDataModelTypeRef::I128,
            DataModelType::U16 => return InternDataModelTypeRef::U16,
            DataModelType::U32 => return InternDataModelTypeRef::U32,
            DataModelType::U64 => return InternDataModelTypeRef::U64,
            DataModelType::U128 => return InternDataModelTypeRef::U128,
            DataModelType::Usize => return InternDataModelTypeRef::Usize,
            DataModelType::Isize => return InternDataModelTypeRef::Isize,
            DataModelType::F32 => return InternDataModelTypeRef::F32,
            DataModelType::F64 => return InternDataModelTypeRef::F64,
            DataModelType::Char => return InternDataModelTypeRef::Char,
            DataModelType::String => return InternDataModelTypeRef::String,
            DataModelType::ByteArray => return InternDataModelTypeRef::ByteArray,
            DataModelType::Unit => return InternDataModelTypeRef::Unit,
            _ => {}
        }

        let mut count = 0;
        while count < self.dmts().len() {
            if self.is_dmt_match(dmt, &self.dmts[count]) {
                return InternDataModelTypeRef::Ref(count);
            }
            count += 1;
        }
        let old_len = self.dmts_len;
        self.dmts[self.dmts_len] = match dmt {
            DataModelType::Bool => unreachable!(),
            DataModelType::I8 => unreachable!(),
            DataModelType::U8 => unreachable!(),
            DataModelType::I16 => unreachable!(),
            DataModelType::I32 => unreachable!(),
            DataModelType::I64 => unreachable!(),
            DataModelType::I128 => unreachable!(),
            DataModelType::U16 => unreachable!(),
            DataModelType::U32 => unreachable!(),
            DataModelType::U64 => unreachable!(),
            DataModelType::U128 => unreachable!(),
            DataModelType::Usize => unreachable!(),
            DataModelType::Isize => unreachable!(),
            DataModelType::F32 => unreachable!(),
            DataModelType::F64 => unreachable!(),
            DataModelType::Char => unreachable!(),
            DataModelType::String => unreachable!(),
            DataModelType::ByteArray => unreachable!(),
            DataModelType::Unit => unreachable!(),
            DataModelType::Option(data_model_type) => {
                let rf = self.get_or_insert_interned_dmt(data_model_type);
                InternDataModelType::Option(rf)
            }
            DataModelType::Seq(data_model_type) => {
                InternDataModelType::Seq(self.get_or_insert_interned_dmt(data_model_type))
            }
            DataModelType::Tuple(data_model_types) => {
                InternDataModelType::Tuple(self.get_or_insert_run_dmt(data_model_types))
            }
            DataModelType::Array { item, count } => InternDataModelType::Array {
                item: self.get_or_insert_interned_dmt(item),
                count: *count,
            },
            DataModelType::Map { key, val } => InternDataModelType::Map {
                key: self.get_or_insert_interned_dmt(key),
                val: self.get_or_insert_interned_dmt(val),
            },
            DataModelType::Struct { name, data } => InternDataModelType::Struct {
                name: self.get_interned_str(name),
                data: self.get_or_insert_interned_data(data),
            },
            DataModelType::Enum { name, variants } => InternDataModelType::Enum {
                name: self.get_interned_str(name),
                variants: self.get_or_insert_variant_group(variants),
            },
            DataModelType::Schema => todo!(),
        };
        self.dmts_len += 1;
        InternDataModelTypeRef::Ref(old_len)
    }

    const fn run_dmts(&self) -> &[InternDataModelType] {
        let (now, _later) = self.run_dmts.split_at(self.run_dmts_len);
        now
    }

    const fn dmts(&self) -> &[InternDataModelType] {
        let (now, _later) = self.dmts.split_at(self.dmts_len);
        now
    }

    const fn nfs(&self) -> &[InternNamedField] {
        let (now, _later) = self.nfs.split_at(self.nfs_len);
        now
    }

    const fn vnts(&self) -> &[InternVariant] {
        let (now, _later) = self.vnts.split_at(self.vnts_len);
        now
    }

    ///////

    pub const fn is_data_match(&self, data: &Data, intern: &InternData) -> bool {
        match (data, intern) {
            (Data::Unit, InternData::Unit) => true,
            (Data::Newtype(data_model_type), InternData::Newtype(intern_data_model_type_ref)) => {
                let data = self.dmt_from_ref(intern_data_model_type_ref);

                self.is_dmt_match(data_model_type, &data)
            }
            (Data::Tuple(data_model_types), InternData::Tuple(intern_data_model_group_ref)) => {
                if data_model_types.len() != intern_data_model_group_ref.len {
                    return false;
                }
                let mut count = 0;
                while count < data_model_types.len() {
                    let m = self.is_dmt_match(
                        data_model_types[count],
                        &self.dmts()[count + intern_data_model_group_ref.offset],
                    );
                    if !m {
                        return false;
                    }
                    count += 1;
                }
                true
            }
            (Data::Struct(named_fields), InternData::Struct(intern_named_field_group_ref)) => {
                if named_fields.len() != intern_named_field_group_ref.len {
                    return false;
                }
                let mut count = 0;
                while count < named_fields.len() {
                    let offct = intern_named_field_group_ref.offset + count;
                    let inf = &self.nfs()[offct];
                    if !streq(
                        named_fields[count].name,
                        str_subslice(self.strs, inf.name.offset, inf.name.len),
                    ) {
                        return false;
                    }

                    let data = self.dmt_from_ref(&inf.ty);

                    if !self.is_dmt_match(named_fields[count].ty, &data) {
                        return false;
                    }
                    count += 1;
                }
                true
            }
            _ => false,
        }
    }

    pub const fn dmt_from_ref(&self, dref: &InternDataModelTypeRef) -> InternDataModelType {
        match dref {
            InternDataModelTypeRef::Bool => InternDataModelType::Bool,
            InternDataModelTypeRef::I8 => InternDataModelType::I8,
            InternDataModelTypeRef::U8 => InternDataModelType::U8,
            InternDataModelTypeRef::I16 => InternDataModelType::I16,
            InternDataModelTypeRef::I32 => InternDataModelType::I32,
            InternDataModelTypeRef::I64 => InternDataModelType::I64,
            InternDataModelTypeRef::I128 => InternDataModelType::I128,
            InternDataModelTypeRef::U16 => InternDataModelType::U16,
            InternDataModelTypeRef::U32 => InternDataModelType::U32,
            InternDataModelTypeRef::U64 => InternDataModelType::U64,
            InternDataModelTypeRef::U128 => InternDataModelType::U128,
            InternDataModelTypeRef::Usize => InternDataModelType::Usize,
            InternDataModelTypeRef::Isize => InternDataModelType::Isize,
            InternDataModelTypeRef::F32 => InternDataModelType::F32,
            InternDataModelTypeRef::F64 => InternDataModelType::F64,
            InternDataModelTypeRef::Char => InternDataModelType::Char,
            InternDataModelTypeRef::String => InternDataModelType::String,
            InternDataModelTypeRef::ByteArray => InternDataModelType::ByteArray,
            InternDataModelTypeRef::Unit => InternDataModelType::Unit,
            InternDataModelTypeRef::Ref(idx) => self.dmts()[*idx],
            InternDataModelTypeRef::Schema => todo!(),
        }
    }

    pub const fn is_dmt_match(&self, dmt: &DataModelType, idmt: &InternDataModelType) -> bool {
        match (idmt, dmt) {
            (InternDataModelType::Bool, DataModelType::Bool) => true,
            (InternDataModelType::I8, DataModelType::I8) => true,
            (InternDataModelType::U8, DataModelType::U8) => true,
            (InternDataModelType::I16, DataModelType::I16) => true,
            (InternDataModelType::I32, DataModelType::I32) => true,
            (InternDataModelType::I64, DataModelType::I64) => true,
            (InternDataModelType::I128, DataModelType::I128) => true,
            (InternDataModelType::U16, DataModelType::U16) => true,
            (InternDataModelType::U32, DataModelType::U32) => true,
            (InternDataModelType::U64, DataModelType::U64) => true,
            (InternDataModelType::U128, DataModelType::U128) => true,
            (InternDataModelType::Usize, DataModelType::Usize) => true,
            (InternDataModelType::Isize, DataModelType::Isize) => true,
            (InternDataModelType::F32, DataModelType::F32) => true,
            (InternDataModelType::F64, DataModelType::F64) => true,
            (InternDataModelType::Char, DataModelType::Char) => true,
            (InternDataModelType::String, DataModelType::String) => true,
            (InternDataModelType::ByteArray, DataModelType::ByteArray) => true,
            (
                InternDataModelType::Option(intern_data_model_type_ref),
                DataModelType::Option(data_model_type),
            ) => self.is_dmt_match(
                data_model_type,
                &self.dmt_from_ref(intern_data_model_type_ref),
            ),
            (InternDataModelType::Unit, DataModelType::Unit) => true,
            (
                InternDataModelType::Seq(intern_data_model_type_ref),
                DataModelType::Seq(data_model_type),
            ) => self.is_dmt_match(
                data_model_type,
                &self.dmt_from_ref(intern_data_model_type_ref),
            ),
            (
                InternDataModelType::Tuple(intern_data_model_group_ref),
                DataModelType::Tuple(data_model_types),
            ) => {
                if intern_data_model_group_ref.len != data_model_types.len() {
                    return false;
                }
                let mut count = 0;
                while count < data_model_types.len() {
                    let idm = intern_data_model_group_ref.offset + count;
                    if !self.is_dmt_match(data_model_types[count], &self.dmts()[idm]) {
                        return false;
                    }
                    count += 1;
                }
                true
            }
            (
                InternDataModelType::Array { item, count },
                DataModelType::Array {
                    item: ditem,
                    count: dcount,
                },
            ) => (*count == *dcount) && self.is_dmt_match(ditem, &self.dmt_from_ref(item)),
            (
                InternDataModelType::Map { key, val },
                DataModelType::Map {
                    key: dkey,
                    val: dval,
                },
            ) => {
                self.is_dmt_match(dkey, &self.dmt_from_ref(key))
                    && self.is_dmt_match(dval, &self.dmt_from_ref(val))
            }
            (
                InternDataModelType::Struct { name, data },
                DataModelType::Struct {
                    name: dname,
                    data: ddata,
                },
            ) => {
                let name_match = streq(str_subslice(self.strs, name.offset, name.len), dname);
                let data_match = self.is_data_match(ddata, data);
                name_match && data_match
            }
            (
                InternDataModelType::Enum { name, variants },
                DataModelType::Enum {
                    name: dname,
                    variants: dvariants,
                },
            ) => {
                if !streq(str_subslice(self.strs, name.offset, name.len), dname) {
                    return false;
                }
                if variants.len != dvariants.len() {
                    return false;
                }
                let mut count = 0;
                while count < dvariants.len() {
                    let ivar = &self.vnts()[variants.offset + count];
                    if !streq(
                        dvariants[count].name,
                        str_subslice(self.strs, ivar.name.offset, ivar.name.len),
                    ) {
                        return false;
                    }
                    if !self.is_data_match(&dvariants[count].data, &ivar.data) {
                        return false;
                    }
                    count += 1;
                }
                true
            }
            (InternDataModelType::Schema, DataModelType::Schema) => todo!(),
            _ => false,
        }
    }

    pub const fn find_run_dmt(&self, dmts: &[&DataModelType]) -> Option<InternDataModelGroupRef> {
        if self.dmts.len() < dmts.len() {
            return None;
        }
        assert!(!dmts.is_empty());

        let mut count = 0;
        'outer: while count < (self.dmts.len() - dmts.len()) {
            let mut icount = 0;
            while icount < dmts.len() {
                if !self.is_dmt_match(dmts[icount], &self.dmts[count + icount]) {
                    count += 1;
                    continue 'outer;
                }
                icount += 1;
            }
            return Some(InternDataModelGroupRef {
                offset: count,
                len: dmts.len(),
            });
        }
        None
    }

    pub const fn find_named_field_group(
        &self,
        nfs: &[&NamedField],
    ) -> Option<InternNamedFieldGroupRef> {
        if self.nfs.len() < nfs.len() {
            return None;
        }
        assert!(!nfs.is_empty());

        let mut count = 0;
        'outer: while count < (self.nfs.len() - nfs.len()) {
            let mut icount = 0;
            while icount < nfs.len() {
                if !streq(
                    nfs[icount].name,
                    str_subslice(
                        self.strs,
                        self.nfs[count + icount].name.offset,
                        self.nfs[count + icount].name.len,
                    ),
                ) {
                    count += 1;
                    continue 'outer;
                }
                if !self.is_dmt_match(
                    nfs[icount].ty,
                    &self.dmt_from_ref(&self.nfs[count + icount].ty),
                ) {
                    count += 1;
                    continue 'outer;
                }
                icount += 1;
            }
            return Some(InternNamedFieldGroupRef {
                offset: count,
                len: nfs.len(),
            });
        }
        None
    }

    pub const fn find_variant_group(&self, vnts: &[&Variant]) -> Option<InternVariantGroupRef> {
        if self.vnts.len() < vnts.len() {
            return None;
        }
        assert!(!vnts.is_empty());

        let mut count = 0;
        'outer: while count < (self.vnts.len() - vnts.len()) {
            let mut icount = 0;
            while icount < vnts.len() {
                if !streq(
                    vnts[icount].name,
                    str_subslice(
                        self.strs,
                        self.vnts[count + icount].name.offset,
                        self.vnts[count + icount].name.len,
                    ),
                ) {
                    count += 1;
                    continue 'outer;
                }
                if !self.is_data_match(&vnts[icount].data, &self.vnts[count + icount].data) {
                    count += 1;
                    continue 'outer;
                }
                icount += 1;
            }
            return Some(InternVariantGroupRef {
                offset: count,
                len: vnts.len(),
            });
        }
        None
    }
}

pub struct InternedSchema<'a, const DMTS: usize, const NFS: usize, const VNTS: usize> {
    strs: &'a str,
    dmts: [InternDataModelType; DMTS],
    nfs: [InternNamedField; NFS],
    vnts: [InternVariant; VNTS],
}

pub mod ty_intern {
    use crate::schema::Data;
    use crate::schema::DataModelType;

    //////////////////////////////////////////////////////////////////////
    // RUN DMTS
    //////////////////////////////////////////////////////////////////////

    pub const fn count_run_dmts_data(data: &Data) -> usize {
        match data {
            Data::Unit => 0,
            Data::Newtype(data_model_type) => count_run_dmts(data_model_type),
            Data::Tuple(data_model_types) => {
                let mut count = data_model_types.len();
                let mut idx = 0;
                while idx < data_model_types.len() {
                    count += count_run_dmts(data_model_types[idx]);
                    idx += 1;
                }
                count
            }
            Data::Struct(named_fields) => {
                let mut count = 0;
                let mut idx = 0;
                while idx < named_fields.len() {
                    count += count_run_dmts(named_fields[idx].ty);
                    idx += 1;
                }
                count
            }
        }
    }

    pub const fn count_run_dmts(dmt: &DataModelType) -> usize {
        match dmt {
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
            DataModelType::Option(data_model_type) => count_run_dmts(data_model_type),
            DataModelType::Unit => 0,
            DataModelType::Seq(data_model_type) => count_run_dmts(data_model_type),
            DataModelType::Tuple(data_model_types) => {
                let mut count = data_model_types.len();
                let mut idx = 0;
                while idx < data_model_types.len() {
                    count += count_run_dmts(data_model_types[idx]);
                    idx += 1;
                }
                count
            }
            DataModelType::Array { item, count: _ } => count_run_dmts(item),
            DataModelType::Map { key, val } => count_run_dmts(key) + count_run_dmts(val),
            DataModelType::Struct { name: _, data } => count_run_dmts_data(data),
            DataModelType::Enum { name: _, variants } => {
                let mut count = 0;
                let mut idx = 0;
                while idx < variants.len() {
                    count += count_run_dmts_data(&variants[idx].data);
                    idx += 1;
                }
                count
            }
            DataModelType::Schema => todo!(),
        }
    }

    //////////////////////////////////////////////////////////////////////
    // INTERN INTERN INTERN
    //////////////////////////////////////////////////////////////////////

    pub const fn str_subslice(s: &str, offset: usize, len: usize) -> &str {
        let s = s.as_bytes();
        let (_before, now) = s.split_at(offset);
        let (now, _after) = now.split_at(len);
        match core::str::from_utf8(now) {
            Ok(s) => s,
            Err(_) => panic!(),
        }
    }

    //////////////////////////////////////////////////////////////////////
    // Named Fields
    //////////////////////////////////////////////////////////////////////

    pub const fn count_named_fields_data(data: &Data) -> usize {
        match data {
            Data::Unit => 0,
            Data::Newtype(data_model_type) => count_named_fields(data_model_type),
            Data::Tuple(data_model_types) => {
                let mut count = 0;
                let mut idx = 0;
                while idx < data_model_types.len() {
                    count += count_named_fields(data_model_types[idx]);
                    idx += 1;
                }
                count
            }
            Data::Struct(named_fields) => {
                let mut count = named_fields.len();
                let mut idx = 0;
                while idx < named_fields.len() {
                    count += count_named_fields(named_fields[idx].ty);
                    idx += 1;
                }
                count
            }
        }
    }

    pub const fn count_named_fields(dmt: &DataModelType) -> usize {
        match dmt {
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
            DataModelType::Option(data_model_type) => count_named_fields(data_model_type),
            DataModelType::Unit => 0,
            DataModelType::Seq(data_model_type) => count_named_fields(data_model_type),
            DataModelType::Tuple(data_model_types) => {
                let mut count = 0;
                let mut idx = 0;
                while idx < data_model_types.len() {
                    count += count_named_fields(data_model_types[idx]);
                    idx += 1;
                }
                count
            }
            DataModelType::Array { item, count: _ } => count_named_fields(item),
            DataModelType::Map { key, val } => count_named_fields(key) + count_named_fields(val),
            DataModelType::Struct { name: _, data } => count_named_fields_data(data),
            DataModelType::Enum { name: _, variants } => {
                let mut count = 0;
                let mut idx = 0;
                while idx < variants.len() {
                    count += count_named_fields_data(&variants[idx].data);
                    idx += 1;
                }
                count
            }
            DataModelType::Schema => todo!(),
        }
    }

    //////////////////////////////////////////////////////////////////////
    // Variants
    //////////////////////////////////////////////////////////////////////

    pub const fn count_variants_data(data: &Data) -> usize {
        match data {
            Data::Unit => 0,
            Data::Newtype(data_model_type) => count_variants(data_model_type),
            Data::Tuple(data_model_types) => {
                let mut count = 0;
                let mut idx = 0;
                while idx < data_model_types.len() {
                    count += count_variants(data_model_types[idx]);
                    idx += 1;
                }
                count
            }
            Data::Struct(named_fields) => {
                let mut count = 0;
                let mut idx = 0;
                while idx < named_fields.len() {
                    count += count_variants(named_fields[idx].ty);
                    idx += 1;
                }
                count
            }
        }
    }

    pub const fn count_variants(dmt: &DataModelType) -> usize {
        match dmt {
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
            DataModelType::Option(data_model_type) => count_variants(data_model_type),
            DataModelType::Unit => 0,
            DataModelType::Seq(data_model_type) => count_variants(data_model_type),
            DataModelType::Tuple(data_model_types) => {
                let mut count = 0;
                let mut idx = 0;
                while idx < data_model_types.len() {
                    count += count_variants(data_model_types[idx]);
                    idx += 1;
                }
                count
            }
            DataModelType::Array { item, count: _ } => count_variants(item),
            DataModelType::Map { key, val } => count_variants(key) + count_variants(val),
            DataModelType::Struct { name: _, data } => count_variants_data(data),
            DataModelType::Enum { name: _, variants } => {
                let mut count = variants.len();
                let mut idx = 0;
                while idx < variants.len() {
                    count += count_variants_data(&variants[idx].data);
                    idx += 1;
                }
                count
            }
            DataModelType::Schema => todo!(),
        }
    }

    //////////////////////////////////////////////////////////////////////
    // DMTS
    //////////////////////////////////////////////////////////////////////

    pub const fn count_dmt_data(data: &Data) -> usize {
        match data {
            Data::Unit => 1,
            Data::Newtype(data_model_type) => 1 + count_dmt(data_model_type),
            Data::Tuple(data_model_types) => {
                let mut count = 1 + data_model_types.len();
                let mut idx = 0;
                while idx < data_model_types.len() {
                    count += count_dmt(data_model_types[idx]);
                    idx += 1;
                }
                count
            }
            Data::Struct(named_fields) => {
                let mut count = 1;
                let mut idx = 0;
                while idx < named_fields.len() {
                    count += count_dmt(named_fields[idx].ty);
                    idx += 1;
                }
                count
            }
        }
    }

    pub const fn count_dmt(dmt: &DataModelType) -> usize {
        match dmt {
            DataModelType::Bool => 1,
            DataModelType::I8 => 1,
            DataModelType::U8 => 1,
            DataModelType::I16 => 1,
            DataModelType::I32 => 1,
            DataModelType::I64 => 1,
            DataModelType::I128 => 1,
            DataModelType::U16 => 1,
            DataModelType::U32 => 1,
            DataModelType::U64 => 1,
            DataModelType::U128 => 1,
            DataModelType::Usize => 1,
            DataModelType::Isize => 1,
            DataModelType::F32 => 1,
            DataModelType::F64 => 1,
            DataModelType::Char => 1,
            DataModelType::String => 1,
            DataModelType::ByteArray => 1,
            DataModelType::Option(data_model_type) => 1 + count_dmt(data_model_type),
            DataModelType::Unit => 1,
            DataModelType::Seq(data_model_type) => 1 + count_dmt(data_model_type),
            DataModelType::Tuple(data_model_types) => {
                let mut count = 1 + data_model_types.len();
                let mut idx = 0;
                while idx < data_model_types.len() {
                    count += count_dmt(data_model_types[idx]);
                    idx += 1;
                }
                count
            }
            DataModelType::Array { item, count: _ } => 1 + count_dmt(item),
            DataModelType::Map { key, val } => 1 + count_dmt(key) + count_dmt(val),
            DataModelType::Struct { name: _, data } => 1 + count_dmt_data(data),
            DataModelType::Enum { name: _, variants } => {
                let mut count = 1;
                let mut idx = 0;
                while idx < variants.len() {
                    count += count_dmt_data(&variants[idx].data);
                    idx += 1;
                }
                count
            }
            DataModelType::Schema => todo!(),
        }
    }
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
