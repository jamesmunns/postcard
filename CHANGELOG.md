# Changelog

Changes will be described here.

## Unreleased

...

## 1.0.0-alpha.1 -> 1.0.0-alpha.2

* Re-exposed fields of the Flavor constructors, made various flavors impl `Default`
* No breaking changes vs `1.0.0-alpha.1`.

## 0.7.3 -> 1.0.0-alpha.1

* WARNING: This includes a BREAKING wire change from postcard v0.x.y! Please ensure
    all devices using postcard are recompiled with the newest version!
* added `#[inline]` to many functions, increasing performance
* All unsigned integers u16-u128 are varint encoded
* All signed integers i16-i128 are zigzag + varint encoded
* Serialization flavors have been tweaked slightly, with the `Slice` flavor now faster
* Introduction of Deserialization flavors
* Please report any bugs upstream as we prepare for the v1.0.0 release!

## 0.7.2 -> 0.7.3

* Added optional [`defmt`](https://crates.io/crates/defmt) support with the `use-defmt` feature.
* Improved docs

## 0.7.1 -> 0.7.2

* Changed the `CobsAccumulator::new()` into a const fn.

## 0.7.0 -> 0.7.1

* Added the `CobsAccumulator` type for accumulating COBS encoded data for deserialization.

## 0.6.x -> 0.7.0

* Updated `heapless` dependency to `v0.7.0`, which added support for const-generic sized buffers.

## Prior to 0.7.0

* No changelog information added yet.
