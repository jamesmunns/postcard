//! Check a type's [`Schema`] against its serde implementations
//!
//! A schema describes the serde calls that a type's `Serialize` and
//! `Deserialize` implementations make. A schema that disagrees with them
//! describes messages that the type never produces, so tools that use the
//! schema, such as `postcard-dyn`, encode or decode the wrong bytes, and keys
//! may be the same for types that are not wire compatible.
//!
//! [`verify`] checks a sample value of a type in three steps:
//!
//! 1. It serializes the value with a serializer that walks `T::SCHEMA`, and
//!    checks all serde calls against the schema.
//! 2. It deserializes a new value with a deserializer that walks `T::SCHEMA`,
//!    and checks all serde calles against the schema. It also ensures that all
//!    items are consumed.
//! 3. It serializes the new value, and fails if that records different
//!    values.
//!
//! Only the serde calls that the sample value makes are checked: a `None`, an
//! empty sequence, or one variant of an enum leave the rest of the schema
//! unchecked. Check several values that together reach every part of the
//! schema.
//!
//! [`verify_all`] checks several values, and fails unless every part of the
//! schema is reached. For types that borrow, add the [`Recording`]s to a
//! [`Coverage`] instead.
//!
//! ```
//! use postcard_schema::verify::{verify, verify_all};
//!
//! verify(&Some((1u8, -2i32))).unwrap();
//! verify(&None::<(u8, i32)>).unwrap();
//!
//! verify_all(&[Some((1u8, -2i32)), None]).unwrap();
//! ```

extern crate alloc;

use alloc::{
    borrow::ToOwned,
    boxed::Box,
    format,
    string::{String, ToString},
    vec::Vec,
};
use core::fmt::{self, Display};

use serde::{
    de::{self, value::U32Deserializer, DeserializeOwned, DeserializeSeed, IntoDeserializer},
    ser, Deserialize, Serialize,
};

use crate::{
    schema::{Data, DataModelType, NamedField, Variant},
    Schema,
};

/// A disagreement between a schema and a serde implementation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    path: String,
    msg: String,
}

impl Error {
    fn new(path: &str, msg: impl Display) -> Self {
        Error {
            path: path.to_owned(),
            msg: msg.to_string(),
        }
    }

    /// Where in the value the error happened, such as `.a[2].b`. Empty for the
    /// value itself.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// What went wrong
    pub fn message(&self) -> &str {
        &self.msg
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.path.is_empty() {
            f.write_str(&self.msg)
        } else {
            write!(f, "at `{}`: {}", self.path, self.msg)
        }
    }
}

impl core::error::Error for Error {}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::new("", msg)
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::new("", msg)
    }
}

/// Check `value` against `T::SCHEMA`, as described in the
/// [module documentation](self).
pub fn verify<T>(value: &T) -> Result<(), Error>
where
    T: Schema + Serialize + DeserializeOwned,
{
    verify_serialize(value)?.verify_deserialize::<T>()
}

/// Check the serde calls that serializing `value` makes against `T::SCHEMA`,
/// and record them.
///
/// For a type that borrows from its input, such as `&str`, use
/// [`Recording::verify_deserialize`] on the result to check deserializing.
pub fn verify_serialize<T>(value: &T) -> Result<Recording, Error>
where
    T: Schema + Serialize + ?Sized,
{
    let rec = value.serialize(Ser {
        ty: T::SCHEMA,
        path: String::new(),
    })?;
    Ok(Recording {
        schema: T::SCHEMA,
        rec,
    })
}

/// The values recorded while serializing a value, from [`verify_serialize`]
#[derive(Debug, Clone, PartialEq)]
pub struct Recording {
    schema: &'static DataModelType,
    rec: Rec,
}

impl Recording {
    /// Deserialize a `T` from the recorded values, checking each
    /// `deserialize_*` call against `T::SCHEMA`, then serialize it again and
    /// check that it records the same values.
    pub fn verify_deserialize<'de, T>(&'de self) -> Result<(), Error>
    where
        T: Schema + Serialize + Deserialize<'de>,
    {
        if T::SCHEMA != self.schema {
            return Err(Error::new("", "the schema differs from the recorded one"));
        }
        let value = T::deserialize(De {
            ty: self.schema,
            rec: &self.rec,
            path: String::new(),
        })?;
        let again = verify_serialize(&value)?;
        if again.rec != self.rec {
            return Err(Error::new(
                "",
                format!(
                    "serializing the deserialized value gave {:?}, not {:?}",
                    again.rec, self.rec
                ),
            ));
        }
        Ok(())
    }
}

/// Check every value in `values` as [`verify`] does, then check that every part
/// of the `T::SCHEMA` is reached as [`Coverage`] describes.
pub fn verify_all<T>(values: &[T]) -> Result<(), Error>
where
    T: Schema + Serialize + DeserializeOwned,
{
    let mut coverage = Coverage::new::<T>();
    for (i, value) in values.iter().enumerate() {
        let rec = verify_serialize(value).map_err(|e| e.in_value(i))?;
        rec.verify_deserialize::<T>().map_err(|e| e.in_value(i))?;
        coverage.add(&rec)?;
    }
    coverage.check()
}

/// The parts of a schema that a set of [`Recording`]s reached
///
/// A schema is covered when the recordings reach every variant of each enum,
/// both `None` and `Some` of each option, and a non-empty sequence and map.
/// Each type in the schema only needs to be covered once.
#[derive(Debug, Clone)]
pub struct Coverage {
    schema: &'static DataModelType,
    reached: Vec<(&'static DataModelType, Part)>,
}

/// A part of a type that a value may or may not reach
#[derive(Debug, Clone, Copy, PartialEq)]
enum Part {
    None,
    Some,
    NonEmpty,
    Variant(usize),
}

impl Coverage {
    /// No coverage yet of `T::SCHEMA`
    pub fn new<T: Schema + ?Sized>() -> Self {
        Coverage {
            schema: T::SCHEMA,
            reached: Vec::new(),
        }
    }

    /// Add the parts of the schema that `rec` reached
    pub fn add(&mut self, rec: &Recording) -> Result<(), Error> {
        if rec.schema != self.schema {
            return Err(Error::new("", "the schema differs from the recorded one"));
        }
        self.mark(self.schema, &rec.rec);
        Ok(())
    }

    /// Fail on the first part of the schema that no recording reached
    pub fn check(&self) -> Result<(), Error> {
        self.unreached(self.schema, String::new())
    }

    fn reach(&mut self, ty: &'static DataModelType, part: Part) {
        if !self.has(ty, part) {
            self.reached.push((ty, part));
        }
    }

    fn has(&self, ty: &DataModelType, part: Part) -> bool {
        self.reached.iter().any(|(t, p)| *p == part && *t == ty)
    }

    fn mark(&mut self, ty: &'static DataModelType, rec: &Rec) {
        match (ty, rec) {
            (DataModelType::Option(_), Rec::None) => self.reach(ty, Part::None),
            (DataModelType::Option(t), Rec::Some(r)) => {
                self.reach(ty, Part::Some);
                self.mark(t, r);
            }
            (DataModelType::Seq { element, .. }, Rec::List(items)) => {
                if !items.is_empty() {
                    self.reach(ty, Part::NonEmpty);
                }
                for r in items {
                    self.mark(element, r);
                }
            }
            (DataModelType::Map { key, val, .. }, Rec::Map(items)) => {
                if !items.is_empty() {
                    self.reach(ty, Part::NonEmpty);
                }
                for (k, v) in items {
                    self.mark(key, k);
                    self.mark(val, v);
                }
            }
            (DataModelType::Tuple(ts), Rec::List(items)) => {
                for (t, r) in ts.iter().zip(items) {
                    self.mark(t, r);
                }
            }
            (DataModelType::Struct { data, .. }, r) => self.mark_data(data, r),
            (DataModelType::Enum { variants, .. }, Rec::Variant(idx, r)) => {
                let idx = *idx as usize;
                if let Some(var) = variants.get(idx) {
                    self.reach(ty, Part::Variant(idx));
                    self.mark_data(&var.data, r);
                }
            }
            _ => {}
        }
    }

    fn mark_data(&mut self, data: &'static Data, rec: &Rec) {
        match (data, rec) {
            (Data::Newtype(t), r) => self.mark(t, r),
            (Data::Tuple(ts), Rec::List(items)) => {
                for (t, r) in ts.iter().zip(items) {
                    self.mark(t, r);
                }
            }
            (Data::Struct(fs), Rec::List(items)) => {
                for (f, r) in fs.iter().zip(items) {
                    self.mark(f.ty, r);
                }
            }
            _ => {}
        }
    }

    /// The first part of `ty` that was not reached, at `path`. The parts inside
    /// an unreached part are not reported.
    fn unreached(&self, ty: &DataModelType, path: String) -> Result<(), Error> {
        let need = |part, what: &str| {
            if self.has(ty, part) {
                Ok(())
            } else {
                Err(Error::new(&path, format!("no value reached {what}")))
            }
        };
        match ty {
            DataModelType::Option(t) => {
                need(Part::None, "`None`")?;
                need(Part::Some, "`Some`")?;
                self.unreached(t, format!("{path}.some"))
            }
            DataModelType::Seq { element, .. } => {
                need(Part::NonEmpty, "a non-empty sequence")?;
                self.unreached(element, format!("{path}[]"))
            }
            DataModelType::Map { key, val, .. } => {
                need(Part::NonEmpty, "a non-empty map")?;
                self.unreached(key, format!("{path}[].key"))?;
                self.unreached(val, format!("{path}[].value"))
            }
            DataModelType::Tuple(ts) => {
                for (i, t) in ts.iter().enumerate() {
                    self.unreached(t, format!("{path}.{i}"))?;
                }
                Ok(())
            }
            DataModelType::Struct { data, .. } => self.unreached_data(data, path),
            DataModelType::Enum { variants, .. } => {
                for (idx, var) in variants.iter().enumerate() {
                    need(Part::Variant(idx), &format!("variant `{}`", var.name))?;
                    self.unreached_data(&var.data, format!("{path}::{}", var.name))?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn unreached_data(&self, data: &Data, path: String) -> Result<(), Error> {
        match data {
            Data::Unit => Ok(()),
            Data::Newtype(t) => self.unreached(t, format!("{path}.0")),
            Data::Tuple(ts) => {
                for (i, t) in ts.iter().enumerate() {
                    self.unreached(t, format!("{path}.{i}"))?;
                }
                Ok(())
            }
            Data::Struct(fs) => {
                for f in fs.iter() {
                    self.unreached(f.ty, format!("{path}.{}", f.name))?;
                }
                Ok(())
            }
        }
    }
}

impl Error {
    /// Says which of [`verify_all`]'s values this error is about
    fn in_value(mut self, i: usize) -> Self {
        self.msg = format!("value {i}: {}", self.msg);
        self
    }
}

/// A recorded value. Which variant of the schema it belongs to is known from
/// the schema, so this only keeps the data.
#[derive(Debug, Clone, PartialEq)]
enum Rec {
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    /// Floats are compared by their bits
    F32(u32),
    F64(u64),
    Char(char),
    Str(String),
    Bytes(Vec<u8>),
    None,
    Some(Box<Rec>),
    /// `unit`, a unit struct
    Unit,
    /// A sequence, tuple, tuple struct, or struct
    List(Vec<Rec>),
    Map(Vec<(Rec, Rec)>),
    /// A variant's index and data. The data of a unit variant is [`Rec::Unit`],
    /// and of a newtype variant its field.
    Variant(u32, Box<Rec>),
}

/// A short description of a schema, for errors
fn describe(ty: &DataModelType) -> String {
    match ty {
        DataModelType::Struct { name, data } => {
            format!("struct `{name}` ({})", describe_data(data))
        }
        DataModelType::Enum { name, .. } => format!("enum `{name}`"),
        DataModelType::Option(_) => "Option".to_owned(),
        DataModelType::Seq { .. } => "Seq".to_owned(),
        DataModelType::Tuple(ts) => format!("Tuple of {}", ts.len()),
        DataModelType::Map { .. } => "Map".to_owned(),
        DataModelType::String { .. } => "String".to_owned(),
        DataModelType::ByteArray { .. } => "ByteArray".to_owned(),
        other => format!("{other:?}"),
    }
}

fn describe_data(data: &Data) -> String {
    match data {
        Data::Unit => "unit".to_owned(),
        Data::Newtype(_) => "newtype".to_owned(),
        Data::Tuple(ts) => format!("tuple of {}", ts.len()),
        Data::Struct(fs) => format!("{} fields", fs.len()),
    }
}

fn check_len(path: &str, len: usize, max_len: &Option<usize>) -> Result<(), Error> {
    match max_len {
        Some(max) if len > *max => Err(Error::new(
            path,
            format!("length {len} is more than the schema's max_len of {max}"),
        )),
        _ => Ok(()),
    }
}

fn check_name(path: &str, what: &str, schema: &str, serde: &str) -> Result<(), Error> {
    if schema == serde {
        Ok(())
    } else {
        Err(Error::new(
            path,
            format!("serde's {what} name is `{serde}`, but the schema's is `{schema}`"),
        ))
    }
}

fn check_count(path: &str, what: &str, schema: usize, serde: usize) -> Result<(), Error> {
    if schema == serde {
        Ok(())
    } else {
        Err(Error::new(
            path,
            format!("serde has {serde} {what}, but the schema has {schema}"),
        ))
    }
}

/// The variant with index `idx` of an enum schema, checked against the names
/// serde gave
fn variant(
    path: &str,
    ty: &'static DataModelType,
    call: &str,
    name: &str,
    idx: u32,
    var: &str,
) -> Result<&'static Variant, Error> {
    let DataModelType::Enum {
        name: sname,
        variants,
    } = ty
    else {
        return Err(mismatch(path, call, ty));
    };
    check_name(path, "enum", sname, name)?;
    let Some(v) = variants.get(idx as usize) else {
        return Err(Error::new(
            path,
            format!(
                "variant index {idx} (`{var}`) is out of range for the schema's {} variants",
                variants.len()
            ),
        ));
    };
    check_name(path, "variant", v.name, var)?;
    Ok(v)
}

fn mismatch(path: &str, call: &str, ty: &DataModelType) -> Error {
    Error::new(
        path,
        format!("`{call}` was called, but the schema is {}", describe(ty)),
    )
}

fn field_names(fields: &[&NamedField]) -> Vec<&'static str> {
    fields.iter().map(|f| f.name).collect()
}

//
// Serializing
//

struct Ser {
    ty: &'static DataModelType,
    path: String,
}

impl Ser {
    fn child(&self, ty: &'static DataModelType, seg: impl Display) -> Ser {
        Ser {
            ty,
            path: format!("{}{seg}", self.path),
        }
    }

    fn mismatch(&self, call: &str) -> Error {
        mismatch(&self.path, call, self.ty)
    }

    fn variant(
        &self,
        call: &str,
        name: &str,
        idx: u32,
        var: &str,
    ) -> Result<&'static Variant, Error> {
        variant(&self.path, self.ty, call, name, idx, var)
    }
}

macro_rules! ser_prim {
    ($($method:ident($t:ty) => $kind:ident => $rec:expr;)*) => {
        $(
            fn $method(self, v: $t) -> Result<Rec, Error> {
                match self.ty {
                    DataModelType::$kind => Ok($rec(v)),
                    _ => Err(self.mismatch(stringify!($method))),
                }
            }
        )*
    };
}

impl ser::Serializer for Ser {
    type Ok = Rec;
    type Error = Error;
    type SerializeSeq = SeqSer;
    type SerializeTuple = ElemSer;
    type SerializeTupleStruct = ElemSer;
    type SerializeTupleVariant = ElemSer;
    type SerializeMap = MapSer;
    type SerializeStruct = ElemSer;
    type SerializeStructVariant = ElemSer;

    ser_prim! {
        serialize_bool(bool) => Bool => Rec::Bool;
        serialize_i8(i8) => I8 => Rec::I8;
        serialize_i16(i16) => I16 => Rec::I16;
        serialize_i32(i32) => I32 => Rec::I32;
        serialize_i64(i64) => I64 => Rec::I64;
        serialize_i128(i128) => I128 => Rec::I128;
        serialize_u8(u8) => U8 => Rec::U8;
        serialize_u16(u16) => U16 => Rec::U16;
        serialize_u32(u32) => U32 => Rec::U32;
        serialize_u64(u64) => U64 => Rec::U64;
        serialize_u128(u128) => U128 => Rec::U128;
        serialize_f32(f32) => F32 => |v: f32| Rec::F32(v.to_bits());
        serialize_f64(f64) => F64 => |v: f64| Rec::F64(v.to_bits());
        serialize_char(char) => Char => Rec::Char;
    }

    fn serialize_str(self, v: &str) -> Result<Rec, Error> {
        match self.ty {
            DataModelType::String { max_len } => {
                check_len(&self.path, v.len(), max_len)?;
                Ok(Rec::Str(v.to_owned()))
            }
            _ => Err(self.mismatch("serialize_str")),
        }
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Rec, Error> {
        match self.ty {
            DataModelType::ByteArray { max_len } => {
                check_len(&self.path, v.len(), max_len)?;
                Ok(Rec::Bytes(v.to_owned()))
            }
            _ => Err(self.mismatch("serialize_bytes")),
        }
    }

    fn serialize_none(self) -> Result<Rec, Error> {
        match self.ty {
            DataModelType::Option(_) => Ok(Rec::None),
            _ => Err(self.mismatch("serialize_none")),
        }
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Rec, Error> {
        match self.ty {
            DataModelType::Option(t) => {
                let inner = value.serialize(self.child(t, ".some"))?;
                Ok(Rec::Some(Box::new(inner)))
            }
            _ => Err(self.mismatch("serialize_some")),
        }
    }

    fn serialize_unit(self) -> Result<Rec, Error> {
        match self.ty {
            DataModelType::Unit => Ok(Rec::Unit),
            _ => Err(self.mismatch("serialize_unit")),
        }
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Rec, Error> {
        match self.ty {
            DataModelType::Struct {
                name: sname,
                data: Data::Unit,
            } => {
                check_name(&self.path, "struct", sname, name)?;
                Ok(Rec::Unit)
            }
            _ => Err(self.mismatch("serialize_unit_struct")),
        }
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        idx: u32,
        var: &'static str,
    ) -> Result<Rec, Error> {
        let call = "serialize_unit_variant";
        match self.variant(call, name, idx, var)?.data {
            Data::Unit => Ok(Rec::Variant(idx, Box::new(Rec::Unit))),
            ref data => Err(Error::new(
                &self.path,
                format!(
                    "`{call}` was called for `{var}`, but its data is {}",
                    describe_data(data)
                ),
            )),
        }
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Rec, Error> {
        match self.ty {
            DataModelType::Struct {
                name: sname,
                data: Data::Newtype(t),
            } => {
                check_name(&self.path, "struct", sname, name)?;
                value.serialize(self.child(t, ".0"))
            }
            _ => Err(self.mismatch("serialize_newtype_struct")),
        }
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        idx: u32,
        var: &'static str,
        value: &T,
    ) -> Result<Rec, Error> {
        let call = "serialize_newtype_variant";
        match self.variant(call, name, idx, var)?.data {
            Data::Newtype(t) => {
                let inner = value.serialize(self.child(t, format!("::{var}.0")))?;
                Ok(Rec::Variant(idx, Box::new(inner)))
            }
            ref data => Err(Error::new(
                &self.path,
                format!(
                    "`{call}` was called for `{var}`, but its data is {}",
                    describe_data(data)
                ),
            )),
        }
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<SeqSer, Error> {
        match self.ty {
            DataModelType::Seq { element, max_len } => {
                let Some(len) = len else {
                    return Err(Error::new(
                        &self.path,
                        "`serialize_seq` was called without a length, which postcard requires",
                    ));
                };
                check_len(&self.path, len, max_len)?;
                Ok(SeqSer {
                    ser: self.child(element, ""),
                    len,
                    items: Vec::new(),
                })
            }
            _ => Err(self.mismatch("serialize_seq")),
        }
    }

    fn serialize_tuple(self, len: usize) -> Result<ElemSer, Error> {
        match self.ty {
            DataModelType::Tuple(ts) => {
                check_count(&self.path, "tuple elements", ts.len(), len)?;
                Ok(ElemSer::new(self, Elems::Unnamed(ts), None))
            }
            _ => Err(self.mismatch("serialize_tuple")),
        }
    }

    fn serialize_tuple_struct(self, name: &'static str, len: usize) -> Result<ElemSer, Error> {
        match self.ty {
            DataModelType::Struct {
                name: sname,
                data: Data::Tuple(ts),
            } => {
                check_name(&self.path, "struct", sname, name)?;
                check_count(&self.path, "tuple struct fields", ts.len(), len)?;
                Ok(ElemSer::new(self, Elems::Unnamed(ts), None))
            }
            _ => Err(self.mismatch("serialize_tuple_struct")),
        }
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        idx: u32,
        var: &'static str,
        len: usize,
    ) -> Result<ElemSer, Error> {
        let call = "serialize_tuple_variant";
        match self.variant(call, name, idx, var)?.data {
            Data::Tuple(ts) => {
                check_count(&self.path, "tuple variant fields", ts.len(), len)?;
                let ser = self.child(self.ty, format!("::{var}"));
                Ok(ElemSer::new(ser, Elems::Unnamed(ts), Some(idx)))
            }
            ref data => Err(Error::new(
                &self.path,
                format!(
                    "`{call}` was called for `{var}`, but its data is {}",
                    describe_data(data)
                ),
            )),
        }
    }

    fn serialize_map(self, len: Option<usize>) -> Result<MapSer, Error> {
        match self.ty {
            DataModelType::Map { key, val, max_len } => {
                let Some(len) = len else {
                    return Err(Error::new(
                        &self.path,
                        "`serialize_map` was called without a length, which postcard requires",
                    ));
                };
                check_len(&self.path, len, max_len)?;
                Ok(MapSer {
                    key: self.child(key, ".key"),
                    val: self.child(val, ".value"),
                    len,
                    items: Vec::new(),
                    pending: None,
                })
            }
            _ => Err(self.mismatch("serialize_map")),
        }
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<ElemSer, Error> {
        match self.ty {
            DataModelType::Struct {
                name: sname,
                data: Data::Struct(fs),
            } => {
                check_name(&self.path, "struct", sname, name)?;
                check_count(&self.path, "struct fields", fs.len(), len)?;
                Ok(ElemSer::new(self, Elems::Named(fs), None))
            }
            _ => Err(self.mismatch("serialize_struct")),
        }
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        idx: u32,
        var: &'static str,
        len: usize,
    ) -> Result<ElemSer, Error> {
        let call = "serialize_struct_variant";
        match self.variant(call, name, idx, var)?.data {
            Data::Struct(fs) => {
                check_count(&self.path, "struct variant fields", fs.len(), len)?;
                let ser = self.child(self.ty, format!("::{var}"));
                Ok(ElemSer::new(ser, Elems::Named(fs), Some(idx)))
            }
            ref data => Err(Error::new(
                &self.path,
                format!(
                    "`{call}` was called for `{var}`, but its data is {}",
                    describe_data(data)
                ),
            )),
        }
    }

    fn collect_str<T: ?Sized + Display>(self, value: &T) -> Result<Rec, Error> {
        self.serialize_str(&value.to_string())
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

struct SeqSer {
    ser: Ser,
    len: usize,
    items: Vec<Rec>,
}

impl ser::SerializeSeq for SeqSer {
    type Ok = Rec;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        let ser = self
            .ser
            .child(self.ser.ty, format!("[{}]", self.items.len()));
        self.items.push(value.serialize(ser)?);
        Ok(())
    }

    fn end(self) -> Result<Rec, Error> {
        if self.items.len() != self.len {
            return Err(Error::new(
                &self.ser.path,
                format!(
                    "`serialize_seq` was given a length of {}, but {} elements were serialized",
                    self.len,
                    self.items.len()
                ),
            ));
        }
        Ok(Rec::List(self.items))
    }
}

/// The elements of a tuple, tuple struct, or struct
#[derive(Clone, Copy)]
enum Elems {
    Unnamed(&'static [&'static DataModelType]),
    Named(&'static [&'static NamedField]),
}

impl Elems {
    fn len(&self) -> usize {
        match self {
            Elems::Unnamed(ts) => ts.len(),
            Elems::Named(fs) => fs.len(),
        }
    }
}

/// Serializes the elements of a tuple, tuple struct, or struct, and of the
/// tuple and struct variants
struct ElemSer {
    ser: Ser,
    elems: Elems,
    variant: Option<u32>,
    items: Vec<Rec>,
}

impl ElemSer {
    fn new(ser: Ser, elems: Elems, variant: Option<u32>) -> Self {
        ElemSer {
            ser,
            elems,
            variant,
            items: Vec::new(),
        }
    }

    fn element<T: ?Sized + Serialize>(
        &mut self,
        key: Option<&str>,
        value: &T,
    ) -> Result<(), Error> {
        let idx = self.items.len();
        let path = &self.ser.path;
        let ser = match (self.elems, key) {
            (Elems::Unnamed(ts), None) => match ts.get(idx) {
                Some(t) => self.ser.child(t, format!(".{idx}")),
                None => {
                    return Err(Error::new(
                        path,
                        format!(
                            "element {idx} was serialized, but the schema has {}",
                            ts.len()
                        ),
                    ))
                }
            },
            (Elems::Named(fs), Some(key)) => match fs.get(idx) {
                Some(f) => {
                    check_name(path, "field", f.name, key)?;
                    self.ser.child(f.ty, format!(".{key}"))
                }
                None => {
                    return Err(Error::new(
                        path,
                        format!(
                            "field `{key}` was serialized, but the schema has {} fields",
                            fs.len()
                        ),
                    ))
                }
            },
            _ => unreachable!(
                "serde's struct traits always give a key, and its tuple traits never do"
            ),
        };
        self.items.push(value.serialize(ser)?);
        Ok(())
    }

    fn end(self) -> Result<Rec, Error> {
        check_count(
            &self.ser.path,
            "elements",
            self.elems.len(),
            self.items.len(),
        )?;
        let list = Rec::List(self.items);
        Ok(match self.variant {
            Some(idx) => Rec::Variant(idx, Box::new(list)),
            None => list,
        })
    }
}

impl ser::SerializeTuple for ElemSer {
    type Ok = Rec;
    type Error = Error;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        self.element(None, value)
    }
    fn end(self) -> Result<Rec, Error> {
        ElemSer::end(self)
    }
}

impl ser::SerializeTupleStruct for ElemSer {
    type Ok = Rec;
    type Error = Error;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        self.element(None, value)
    }
    fn end(self) -> Result<Rec, Error> {
        ElemSer::end(self)
    }
}

impl ser::SerializeTupleVariant for ElemSer {
    type Ok = Rec;
    type Error = Error;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        self.element(None, value)
    }
    fn end(self) -> Result<Rec, Error> {
        ElemSer::end(self)
    }
}

impl ser::SerializeStruct for ElemSer {
    type Ok = Rec;
    type Error = Error;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        self.element(Some(key), value)
    }
    fn skip_field(&mut self, key: &'static str) -> Result<(), Error> {
        Err(Error::new(
            &self.ser.path,
            format!("field `{key}` was skipped, which postcard does not support"),
        ))
    }
    fn end(self) -> Result<Rec, Error> {
        ElemSer::end(self)
    }
}

impl ser::SerializeStructVariant for ElemSer {
    type Ok = Rec;
    type Error = Error;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        self.element(Some(key), value)
    }
    fn skip_field(&mut self, key: &'static str) -> Result<(), Error> {
        Err(Error::new(
            &self.ser.path,
            format!("field `{key}` was skipped, which postcard does not support"),
        ))
    }
    fn end(self) -> Result<Rec, Error> {
        ElemSer::end(self)
    }
}

struct MapSer {
    key: Ser,
    val: Ser,
    len: usize,
    items: Vec<(Rec, Rec)>,
    pending: Option<Rec>,
}

impl ser::SerializeMap for MapSer {
    type Ok = Rec;
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Error> {
        let n = self.items.len();
        let ser = self.key.child(self.key.ty, format!("[{n}]"));
        self.pending = Some(key.serialize(ser)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        let n = self.items.len();
        let ser = self.val.child(self.val.ty, format!("[{n}]"));
        let val = value.serialize(ser)?;
        let key = self.pending.take().ok_or_else(|| {
            Error::new(
                &self.val.path,
                "`serialize_value` was called before `serialize_key`",
            )
        })?;
        self.items.push((key, val));
        Ok(())
    }

    fn end(self) -> Result<Rec, Error> {
        if self.items.len() != self.len {
            return Err(Error::new(
                &self.key.path,
                format!(
                    "`serialize_map` was given a length of {}, but {} entries were serialized",
                    self.len,
                    self.items.len()
                ),
            ));
        }
        Ok(Rec::Map(self.items))
    }
}

//
// Deserializing
//

struct De<'de> {
    ty: &'static DataModelType,
    rec: &'de Rec,
    path: String,
}

impl<'de> De<'de> {
    fn mismatch(&self, call: &str) -> Error {
        mismatch(&self.path, call, self.ty)
    }

    /// The recorded value does not fit the schema. The serializer only records
    /// values that fit, so this is a bug in this module.
    fn bad_rec(&self) -> Error {
        Error::new(
            &self.path,
            format!(
                "internal error: recorded {:?} for {}",
                self.rec,
                describe(self.ty)
            ),
        )
    }

    fn list(&self) -> Result<&'de [Rec], Error> {
        match self.rec {
            Rec::List(items) => Ok(items),
            _ => Err(self.bad_rec()),
        }
    }

    /// serde's list of fields also has their aliases, so only check that it has
    /// every field of the schema.
    fn check_fields(&self, fields: &[&'static str], schema: &[&NamedField]) -> Result<(), Error> {
        let schema = field_names(schema);
        match schema.iter().find(|f| !fields.contains(f)) {
            None => Ok(()),
            Some(missing) => Err(Error::new(
                &self.path,
                format!("serde's fields are {fields:?}, which is missing the schema's `{missing}`"),
            )),
        }
    }
}

macro_rules! de_prim {
    ($($method:ident => $kind:ident, $rec:ident, $visit:ident;)*) => {
        $(
            fn $method<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
                match (self.ty, self.rec) {
                    (DataModelType::$kind, Rec::$rec(v)) => visitor.$visit(*v),
                    (DataModelType::$kind, _) => Err(self.bad_rec()),
                    _ => Err(self.mismatch(stringify!($method))),
                }
            }
        )*
    };
}

impl<'de> de::Deserializer<'de> for De<'de> {
    type Error = Error;

    fn deserialize_any<V: de::Visitor<'de>>(self, _: V) -> Result<V::Value, Error> {
        Err(Error::new(
            &self.path,
            "`deserialize_any` was called, which postcard does not support",
        ))
    }

    de_prim! {
        deserialize_bool => Bool, Bool, visit_bool;
        deserialize_i8 => I8, I8, visit_i8;
        deserialize_i16 => I16, I16, visit_i16;
        deserialize_i32 => I32, I32, visit_i32;
        deserialize_i64 => I64, I64, visit_i64;
        deserialize_i128 => I128, I128, visit_i128;
        deserialize_u8 => U8, U8, visit_u8;
        deserialize_u16 => U16, U16, visit_u16;
        deserialize_u32 => U32, U32, visit_u32;
        deserialize_u64 => U64, U64, visit_u64;
        deserialize_u128 => U128, U128, visit_u128;
        deserialize_char => Char, Char, visit_char;
    }

    fn deserialize_f32<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match (self.ty, self.rec) {
            (DataModelType::F32, Rec::F32(v)) => visitor.visit_f32(f32::from_bits(*v)),
            (DataModelType::F32, _) => Err(self.bad_rec()),
            _ => Err(self.mismatch("deserialize_f32")),
        }
    }

    fn deserialize_f64<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match (self.ty, self.rec) {
            (DataModelType::F64, Rec::F64(v)) => visitor.visit_f64(f64::from_bits(*v)),
            (DataModelType::F64, _) => Err(self.bad_rec()),
            _ => Err(self.mismatch("deserialize_f64")),
        }
    }

    fn deserialize_str<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match (self.ty, self.rec) {
            (DataModelType::String { .. }, Rec::Str(s)) => visitor.visit_borrowed_str(s),
            (DataModelType::String { .. }, _) => Err(self.bad_rec()),
            _ => Err(self.mismatch("deserialize_str")),
        }
    }

    fn deserialize_string<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match (self.ty, self.rec) {
            (DataModelType::String { .. }, Rec::Str(s)) => visitor.visit_borrowed_str(s),
            (DataModelType::String { .. }, _) => Err(self.bad_rec()),
            _ => Err(self.mismatch("deserialize_string")),
        }
    }

    fn deserialize_bytes<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match (self.ty, self.rec) {
            (DataModelType::ByteArray { .. }, Rec::Bytes(b)) => visitor.visit_borrowed_bytes(b),
            (DataModelType::ByteArray { .. }, _) => Err(self.bad_rec()),
            _ => Err(self.mismatch("deserialize_bytes")),
        }
    }

    fn deserialize_byte_buf<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match (self.ty, self.rec) {
            (DataModelType::ByteArray { .. }, Rec::Bytes(b)) => visitor.visit_borrowed_bytes(b),
            (DataModelType::ByteArray { .. }, _) => Err(self.bad_rec()),
            _ => Err(self.mismatch("deserialize_byte_buf")),
        }
    }

    fn deserialize_option<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match (self.ty, self.rec) {
            (DataModelType::Option(_), Rec::None) => visitor.visit_none(),
            (DataModelType::Option(t), Rec::Some(inner)) => visitor.visit_some(De {
                ty: t,
                rec: inner,
                path: format!("{}.some", self.path),
            }),
            (DataModelType::Option(_), _) => Err(self.bad_rec()),
            _ => Err(self.mismatch("deserialize_option")),
        }
    }

    fn deserialize_unit<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.ty {
            DataModelType::Unit => visitor.visit_unit(),
            _ => Err(self.mismatch("deserialize_unit")),
        }
    }

    fn deserialize_unit_struct<V: de::Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        match self.ty {
            DataModelType::Struct {
                name: sname,
                data: Data::Unit,
            } => {
                check_name(&self.path, "struct", sname, name)?;
                visitor.visit_unit()
            }
            _ => Err(self.mismatch("deserialize_unit_struct")),
        }
    }

    fn deserialize_newtype_struct<V: de::Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        match self.ty {
            DataModelType::Struct {
                name: sname,
                data: Data::Newtype(t),
            } => {
                check_name(&self.path, "struct", sname, name)?;
                visitor.visit_newtype_struct(De {
                    ty: t,
                    rec: self.rec,
                    path: format!("{}.0", self.path),
                })
            }
            _ => Err(self.mismatch("deserialize_newtype_struct")),
        }
    }

    fn deserialize_seq<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.ty {
            DataModelType::Seq { element, .. } => {
                let items = self.list()?;
                visit_seq(&self.path, SeqTys::Same(element), items, visitor)
            }
            _ => Err(self.mismatch("deserialize_seq")),
        }
    }

    fn deserialize_tuple<V: de::Visitor<'de>>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Error> {
        match self.ty {
            DataModelType::Tuple(ts) => {
                check_count(&self.path, "tuple elements", ts.len(), len)?;
                visit_seq(
                    &self.path,
                    SeqTys::Each(Elems::Unnamed(ts)),
                    self.list()?,
                    visitor,
                )
            }
            _ => Err(self.mismatch("deserialize_tuple")),
        }
    }

    fn deserialize_tuple_struct<V: de::Visitor<'de>>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Error> {
        match self.ty {
            DataModelType::Struct {
                name: sname,
                data: Data::Tuple(ts),
            } => {
                check_name(&self.path, "struct", sname, name)?;
                check_count(&self.path, "tuple struct fields", ts.len(), len)?;
                visit_seq(
                    &self.path,
                    SeqTys::Each(Elems::Unnamed(ts)),
                    self.list()?,
                    visitor,
                )
            }
            _ => Err(self.mismatch("deserialize_tuple_struct")),
        }
    }

    fn deserialize_map<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match (self.ty, self.rec) {
            (DataModelType::Map { key, val, .. }, Rec::Map(items)) => {
                let mut access = MapAcc {
                    key,
                    val,
                    items,
                    idx: 0,
                    path: &self.path,
                };
                let value = visitor.visit_map(&mut access)?;
                check_taken(&self.path, "map entries", access.idx, items.len())?;
                Ok(value)
            }
            (DataModelType::Map { .. }, _) => Err(self.bad_rec()),
            _ => Err(self.mismatch("deserialize_map")),
        }
    }

    fn deserialize_struct<V: de::Visitor<'de>>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        match self.ty {
            DataModelType::Struct {
                name: sname,
                data: Data::Struct(fs),
            } => {
                check_name(&self.path, "struct", sname, name)?;
                self.check_fields(fields, fs)?;
                visit_seq(
                    &self.path,
                    SeqTys::Each(Elems::Named(fs)),
                    self.list()?,
                    visitor,
                )
            }
            _ => Err(self.mismatch("deserialize_struct")),
        }
    }

    fn deserialize_enum<V: de::Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        let DataModelType::Enum {
            name: sname,
            variants: svariants,
        } = self.ty
        else {
            return Err(self.mismatch("deserialize_enum"));
        };
        check_name(&self.path, "enum", sname, name)?;
        // serde leaves skipped variants out of its list, so only check that the
        // schema has every variant in it.
        let snames: Vec<&str> = svariants.iter().map(|v| v.name).collect();
        if let Some(extra) = variants.iter().find(|v| !snames.contains(v)) {
            return Err(Error::new(
                &self.path,
                format!("serde has a variant `{extra}`, but the schema's variants are {snames:?}"),
            ));
        }
        let Rec::Variant(idx, data) = self.rec else {
            return Err(self.bad_rec());
        };
        let Some(var) = svariants.get(*idx as usize) else {
            return Err(self.bad_rec());
        };
        visitor.visit_enum(EnumAcc {
            idx: *idx,
            var,
            de: De {
                ty: self.ty,
                rec: data,
                path: format!("{}::{}", self.path, var.name),
            },
        })
    }

    fn deserialize_identifier<V: de::Visitor<'de>>(self, _: V) -> Result<V::Value, Error> {
        Err(self.mismatch("deserialize_identifier"))
    }

    fn deserialize_ignored_any<V: de::Visitor<'de>>(self, _: V) -> Result<V::Value, Error> {
        Err(Error::new(
            &self.path,
            "`deserialize_ignored_any` was called, which postcard does not support",
        ))
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

fn check_taken(path: &str, what: &str, taken: usize, len: usize) -> Result<(), Error> {
    if taken == len {
        Ok(())
    } else {
        Err(Error::new(
            path,
            format!("the visitor took {taken} of {len} {what}, so postcard would misread the rest of the message"),
        ))
    }
}

/// The schemas of the elements of a sequence
#[derive(Clone, Copy)]
enum SeqTys {
    /// A sequence, all of one type
    Same(&'static DataModelType),
    /// A tuple, tuple struct, or struct
    Each(Elems),
}

fn visit_seq<'de, V: de::Visitor<'de>>(
    path: &str,
    tys: SeqTys,
    items: &'de [Rec],
    visitor: V,
) -> Result<V::Value, Error> {
    let mut access = SeqAcc {
        tys,
        items,
        idx: 0,
        path,
    };
    let value = visitor.visit_seq(&mut access)?;
    check_taken(path, "elements", access.idx, items.len())?;
    Ok(value)
}

struct SeqAcc<'de, 'p> {
    tys: SeqTys,
    items: &'de [Rec],
    idx: usize,
    path: &'p str,
}

impl<'de> de::SeqAccess<'de> for SeqAcc<'de, '_> {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Error> {
        let Some(rec) = self.items.get(self.idx) else {
            return Ok(None);
        };
        let (ty, path) = match self.tys {
            SeqTys::Same(t) => (t, format!("{}[{}]", self.path, self.idx)),
            SeqTys::Each(Elems::Unnamed(ts)) => {
                (ts[self.idx], format!("{}.{}", self.path, self.idx))
            }
            SeqTys::Each(Elems::Named(fs)) => {
                let f = fs[self.idx];
                (f.ty, format!("{}.{}", self.path, f.name))
            }
        };
        self.idx += 1;
        seed.deserialize(De { ty, rec, path }).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.items.len() - self.idx)
    }
}

struct MapAcc<'de, 'p> {
    key: &'static DataModelType,
    val: &'static DataModelType,
    items: &'de [(Rec, Rec)],
    idx: usize,
    path: &'p str,
}

impl<'de> de::MapAccess<'de> for MapAcc<'de, '_> {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Error> {
        let Some((rec, _)) = self.items.get(self.idx) else {
            return Ok(None);
        };
        let path = format!("{}.key[{}]", self.path, self.idx);
        seed.deserialize(De {
            ty: self.key,
            rec,
            path,
        })
        .map(Some)
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, Error> {
        let Some((_, rec)) = self.items.get(self.idx) else {
            return Err(Error::new(
                self.path,
                "`next_value` was called after the last entry",
            ));
        };
        let path = format!("{}.value[{}]", self.path, self.idx);
        self.idx += 1;
        seed.deserialize(De {
            ty: self.val,
            rec,
            path,
        })
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.items.len() - self.idx)
    }
}

struct EnumAcc<'de> {
    idx: u32,
    var: &'static Variant,
    de: De<'de>,
}

impl<'de> de::EnumAccess<'de> for EnumAcc<'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self), Error> {
        let idx: U32Deserializer<Error> = self.idx.into_deserializer();
        Ok((seed.deserialize(idx)?, self))
    }
}

impl<'de> de::VariantAccess<'de> for EnumAcc<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        match self.var.data {
            Data::Unit => Ok(()),
            _ => Err(self.mismatch("unit_variant")),
        }
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value, Error> {
        match self.var.data {
            Data::Newtype(t) => seed.deserialize(De {
                ty: t,
                rec: self.de.rec,
                path: format!("{}.0", self.de.path),
            }),
            _ => Err(self.mismatch("newtype_variant")),
        }
    }

    fn tuple_variant<V: de::Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value, Error> {
        match self.var.data {
            Data::Tuple(ts) => {
                check_count(&self.de.path, "tuple variant fields", ts.len(), len)?;
                visit_seq(
                    &self.de.path,
                    SeqTys::Each(Elems::Unnamed(ts)),
                    self.de.list()?,
                    visitor,
                )
            }
            _ => Err(self.mismatch("tuple_variant")),
        }
    }

    fn struct_variant<V: de::Visitor<'de>>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        match self.var.data {
            Data::Struct(fs) => {
                self.de.check_fields(fields, fs)?;
                visit_seq(
                    &self.de.path,
                    SeqTys::Each(Elems::Named(fs)),
                    self.de.list()?,
                    visitor,
                )
            }
            _ => Err(self.mismatch("struct_variant")),
        }
    }
}

impl EnumAcc<'_> {
    fn mismatch(&self, call: &str) -> Error {
        Error::new(
            &self.de.path,
            format!(
                "`{call}` was called for `{}`, but its data is {}",
                self.var.name,
                describe_data(&self.var.data)
            ),
        )
    }
}
