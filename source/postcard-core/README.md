# Postcard Core

`postcard-core` contains the minimal logic necessary for encoding data to and decoding data from
the postcard wire format. It is not coupled to any specific serialization or deserialization
framework, and is intended to be usable when writing a custom postcard serializer or deserializer.

For an example of usage of this crate, see the [`postcard2`] crate, which uses `postcard-core`
to implement a [`serde`] compatible serializer/deserializer.

[`postcard2`]: https://github.com/jamesmunns/postcard/tree/main/source/postcard2
[`serde`]: https://serde.rs
