# Changelog

Changes will be described here.

## 1.0.8 -> Unreleased

* None yet!

## 1.0.7 -> 1.0.8

* Add IO traits support by @xgroleau in https://github.com/jamesmunns/postcard/pull/91

## 1.0.6 -> 1.0.7

* Fix off-by-one in varint size calculation by @Palladinium in https://github.com/jamesmunns/postcard/pull/111
* Add specific error for Crc errors by @CBJamo in https://github.com/jamesmunns/postcard/pull/112

## 1.0.5 -> 1.0.6

* Add cfg information to docs by @dtolnay in https://github.com/jamesmunns/postcard/pull/108

## 1.0.4 -> 1.0.5

* improved documentation of the cobs decoding by @gutzchi in https://github.com/jamesmunns/postcard/pull/97
* Fix strict provenance use of pointers by @jamesmunns in https://github.com/jamesmunns/postcard/pull/100
* Introduces CRCs by @huntc in https://github.com/jamesmunns/postcard/pull/98
* Format with rustfmt to eliminate trailing whitespaces by @dtolnay in https://github.com/jamesmunns/postcard/pull/104
* Fix documentation of re-exports that have dedicated doc comment by @dtolnay in https://github.com/jamesmunns/postcard/pull/107

## New Contributors

* @gutzchi made their first contribution in https://github.com/jamesmunns/postcard/pull/97

**Full Changelog**: https://github.com/jamesmunns/postcard/compare/v1.0.4...v1.0.5

## 1.0.3 -> 1.0.4

* fix cobs accumulator out-of-bounds index when data is 1 byte too long ([PR#90])
* Move cobs accumulator tests into a `cfg(test)` module

[PR#90]: https://github.com/jamesmunns/postcard/pull/90

## 1.0.2 -> 1.0.3

* PhantomData's T doesn't need MaxSize impl ([PR#87])
* Add function for computing the postcard serialized size of a value. ([PR#86])
* Fix typos & encoding example in wire doc ([PR#83])
* Optimize varint decode ([PR#62])
* Bump postcard-derive version ([PR#74])
* add std::string::String and alloc::string::String ([PR#76])
* Make fixints usable through serde field attributes instead of wrappers ([PR#69])
* Add support for 16-bit and 8-bit architectures ([PR#64])
* Add feed_ref to cobs_accumulator ([PR#70])
* Add a link to doc.rs documentation in README ([PR#72])

[PR#87]: https://github.com/jamesmunns/postcard/pull/87
[PR#86]: https://github.com/jamesmunns/postcard/pull/86
[PR#83]: https://github.com/jamesmunns/postcard/pull/83
[PR#62]: https://github.com/jamesmunns/postcard/pull/62
[PR#74]: https://github.com/jamesmunns/postcard/pull/74
[PR#76]: https://github.com/jamesmunns/postcard/pull/76
[PR#69]: https://github.com/jamesmunns/postcard/pull/69
[PR#64]: https://github.com/jamesmunns/postcard/pull/64
[PR#70]: https://github.com/jamesmunns/postcard/pull/70
[PR#72]: https://github.com/jamesmunns/postcard/pull/72

## 1.0.1 -> 1.0.2

* Correct exporting of experimental Schema proc macro ([PR#73])

[PR#73]: https://github.com/jamesmunns/postcard/pull/73

## 1.0.0 -> 1.0.1

* [Fixed deserialization] of `i128`, which was using the "new style" varint serialization, but the incorrect, "old style" fixed deserialization.
    * This is considered a defect, and not a breaking change, as it brings the code back in line with the specification behavior.
    * Version 1.0.0 will be yanked due to this defect.

[Fixed deserialization]: https://github.com/jamesmunns/postcard/commit/70ea33a1ac7f82632697f4578002267eaf9095f5

## 1.0.0-alpha.4 -> 1.0.0

* Added experimental derive features
* Made Flavor fields private again
* Optimized varint encoding
* Use crate `Result` for `Flavor`s

## 1.0.0-alpha.3 -> 1.0.0-alpha.4

* Updated the signature of deserialization `Flavor` trait
* Added documentation and tests
* Removed the `Encoder` wrapper type to better match serialization and deserialization types
* Renamed `ser_flavor::Flavor::release` to `finalize` for consistency
* Re-organized some public items and modules
* Made `Error` non-exhaustive
* Added a `fixint` type to avoid varints

## 1.0.0-alpha.2 -> 1.0.0-alpha.3

* Moved back to `cobs` from `postcard-cobs`
    * This fixed a number of upstream issues, including removal of panicking branches
* Improved documentation and code examples
* Corrected the behavior of `take_from_cobs`
* Added support for serializing `Debug`/`Display` representation strings via serde's `collect_str` method (and removed the panic)

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
