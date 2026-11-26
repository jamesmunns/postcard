//! Encoding schema tables as postcard in const context
//!
//! This writes the same bytes as serializing a table with postcard, but in a
//! `const fn`, so a table's encoding can be a `static` byte array. A device can
//! then send its schemas without serializing them at runtime.

use super::{NameRef, SchemaTable, SchemaTableRef, TableData, TableNode};

/// Writes bytes into `buf`. Keeps counting past the end of `buf`.
struct Out<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl Out<'_> {
    const fn byte(&mut self, b: u8) {
        if self.pos < self.buf.len() {
            self.buf[self.pos] = b;
        }
        self.pos += 1;
    }

    /// A `varint(u64)`, which is also how postcard writes `u32`s, lengths and
    /// enum discriminants.
    const fn varint(&mut self, n: u64) {
        let mut n = n;
        loop {
            let b = (n & 0x7F) as u8;
            n >>= 7;
            if n == 0 {
                self.byte(b);
                return;
            }
            self.byte(b | 0x80);
        }
    }

    const fn u32s(&mut self, ns: &[u32]) {
        self.varint(ns.len() as u64);
        let mut i = 0;
        while i < ns.len() {
            self.varint(ns[i] as u64);
            i += 1;
        }
    }

    const fn name(&mut self, n: NameRef) {
        self.varint(n.offset as u64);
        self.varint(n.len as u64);
    }

    const fn max_len(&mut self, m: Option<u64>) {
        match m {
            None => self.byte(0),
            Some(n) => {
                self.byte(1);
                self.varint(n);
            }
        }
    }

    const fn data(&mut self, t: &SchemaTableRef<'_>, d: TableData) {
        match d {
            TableData::Unit => self.varint(0),
            TableData::Newtype(n) => {
                self.varint(1);
                self.varint(n as u64);
            }
            TableData::Tuple(s) => {
                self.varint(2);
                self.u32s(t.children_of(s));
            }
            TableData::Struct(s) => {
                self.varint(3);
                let fields = t.fields_of(s);
                self.varint(fields.len() as u64);
                let mut i = 0;
                while i < fields.len() {
                    self.name(fields[i].name);
                    self.varint(fields[i].ty as u64);
                    i += 1;
                }
            }
        }
    }

    // schema[impl table.encoding]
    const fn node(&mut self, t: &SchemaTableRef<'_>, n: TableNode) {
        use TableNode as T;
        match n {
            T::Bool => self.varint(0x00),
            T::I8 => self.varint(0x01),
            T::U8 => self.varint(0x02),
            T::I16 => self.varint(0x03),
            T::I32 => self.varint(0x04),
            T::I64 => self.varint(0x05),
            T::I128 => self.varint(0x06),
            T::U16 => self.varint(0x07),
            T::U32 => self.varint(0x08),
            T::U64 => self.varint(0x09),
            T::U128 => self.varint(0x0A),
            T::F32 => self.varint(0x0B),
            T::F64 => self.varint(0x0C),
            T::Char => self.varint(0x0D),
            T::String { max_len } => {
                self.varint(0x0E);
                self.max_len(max_len);
            }
            T::ByteArray { max_len } => {
                self.varint(0x0F);
                self.max_len(max_len);
            }
            T::Option(inner) => {
                self.varint(0x10);
                self.varint(inner as u64);
            }
            T::Unit => self.varint(0x11),
            T::Seq { element, max_len } => {
                self.varint(0x12);
                self.varint(element as u64);
                self.max_len(max_len);
            }
            T::Tuple(s) => {
                self.varint(0x13);
                self.u32s(t.children_of(s));
            }
            T::Map { key, val, max_len } => {
                self.varint(0x14);
                self.varint(key as u64);
                self.varint(val as u64);
                self.max_len(max_len);
            }
            T::Struct { name, data } => {
                self.varint(0x15);
                self.name(name);
                self.data(t, data);
            }
            T::Enum { name, variants } => {
                self.varint(0x16);
                self.name(name);
                let variants = t.variants_of(variants);
                self.varint(variants.len() as u64);
                let mut i = 0;
                while i < variants.len() {
                    self.name(variants[i].name);
                    self.data(t, variants[i].data);
                    i += 1;
                }
            }
            T::Array { element, len } => {
                self.varint(0x17);
                self.varint(element as u64);
                self.varint(len);
            }
        }
    }

    const fn table(&mut self, t: &SchemaTableRef<'_>) {
        self.varint(t.version as u64);
        self.u32s(t.roots);
        // nodes, with their lists inline
        self.varint(t.nodes.len() as u64);
        let mut i = 0;
        while i < t.nodes.len() {
            self.node(t, t.nodes[i]);
            i += 1;
        }
        // names, as a `str`
        let names = t.names.as_bytes();
        self.varint(names.len() as u64);
        let mut i = 0;
        while i < names.len() {
            self.byte(names[i]);
            i += 1;
        }
    }
}

impl SchemaTableRef<'_> {
    /// The length of this table's postcard encoding
    pub const fn encoded_len(&self) -> usize {
        let mut out = Out {
            buf: &mut [],
            pos: 0,
        };
        out.table(self);
        out.pos
    }

    /// Writes this table's postcard encoding to the start of `buf`, returning
    /// its length, or `None` if `buf` is too short.
    pub const fn encode_into(&self, buf: &mut [u8]) -> Option<usize> {
        let len = self.encoded_len();
        if len > buf.len() {
            return None;
        }
        let mut out = Out { buf, pos: 0 };
        out.table(self);
        Some(len)
    }
}

impl<
        const ROOTS: usize,
        const NODES: usize,
        const CHILDREN: usize,
        const VARIANTS: usize,
        const FIELDS: usize,
        const NAME_BYTES: usize,
    > SchemaTable<ROOTS, NODES, CHILDREN, VARIANTS, FIELDS, NAME_BYTES>
{
    /// The length of this table's postcard encoding
    pub const fn encoded_len(&self) -> usize {
        self.as_table_ref().encoded_len()
    }

    /// This table's postcard encoding, as an array
    ///
    /// Fails (at compile time, in const context) if `L` isn't
    /// [`SchemaTable::encoded_len()`]. Use [`schema_table_bytes!`] rather than
    /// calling this directly.
    ///
    /// [`schema_table_bytes!`]: crate::schema_table_bytes
    pub const fn to_bytes<const L: usize>(&self) -> [u8; L] {
        let mut buf = [0; L];
        match self.as_table_ref().encode_into(&mut buf) {
            Some(len) if len == L => buf,
            _ => panic!("wrong encoded length"),
        }
    }
}
