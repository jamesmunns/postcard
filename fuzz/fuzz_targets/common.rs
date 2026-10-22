//! Shared recursive value type for the postcard fuzz targets.
//!
//! A self-describing recursive value that exercises every serde type path
//! postcard must serialize/deserialize (unit, bool, int, uint, float, str,
//! bytes, seq, map). Because it is recursive and there is no recursion limit
//! in postcard, a deeply nested input can also reach the stack overflow that
//! OSS-Fuzz is built to surface for Rust.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FuzzValue {
    Null,
    Bool(bool),
    Int(i64),
    UInt(u64),
    Float(f64),
    Str(String),
    Bytes(Vec<u8>),
    Seq(Vec<FuzzValue>),
    Map(BTreeMap<String, FuzzValue>),
}
