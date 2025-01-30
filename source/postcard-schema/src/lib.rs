#![cfg_attr(not(any(test, feature = "use-std")), no_std)]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
//! # Postcard Schema

pub mod impls;
pub mod key;
pub mod schema;

/// Derive [`Schema`] for a struct or enum
///
/// # Examples
///
/// ```
/// use postcard_schema::Schema;
///
/// #[derive(Schema)]
/// struct Point {
///     x: i32,
///     y: i32,
/// }
/// ```
///
/// # Attributes
///
/// ## `#[postcard(crate = ...)]`
///
/// The `#[postcard(crate = ...)]` attribute can be used to specify a path to the `postcard_schema`
/// crate instance to use when referring to [`Schema`] and [schema types](schema) from generated
/// code. This is normally only applicable when invoking re-exported derives from a different crate.
///
/// ```
/// # use postcard_schema::Schema;
/// use postcard_schema as reexported_postcard_schema;
///
/// #[derive(Schema)]
/// #[postcard(crate = reexported_postcard_schema)]
/// struct Point {
///     x: i32,
///     y: i32,
/// }
/// ```
///
/// ## `#[postcard(bound = ...)]`
///
/// The `#[postcard(bound = ...)]` attribute can be used to overwrite the default bounds when
/// deriving [`Schema`]. The default bounds are `T: Schema` for each type parameter `T`.
///
/// ```
/// # use postcard_schema::Schema;
/// #[derive(Schema)]
/// #[postcard(bound = "")]
/// struct Foo<F: Bar, T: Schema>(F::Wrap<T>);
///
/// trait Bar {
///     type Wrap<T: Schema>: Schema;
/// }
///
/// struct NoSchema;
/// impl Bar for NoSchema {
///     type Wrap<T: Schema> = Option<T>;
/// }
///
/// Foo::<NoSchema, u8>::SCHEMA;
/// ```
#[cfg(feature = "derive")]
pub use postcard_derive::Schema;

/// A trait that represents a compile time calculated schema
pub trait Schema {
    /// A recursive data structure that describes the schema of the given
    /// type.
    const SCHEMA: &'static schema::NamedType;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_path() {
        #[allow(unused)]
        #[derive(Schema)]
        #[postcard(crate = crate)]
        struct Point {
            x: i32,
            y: i32,
        }

        assert_eq!(
            Point::SCHEMA,
            &schema::NamedType {
                name: "Point",
                ty: &schema::DataModelType::Struct(&[
                    &schema::NamedValue {
                        name: "x",
                        ty: i32::SCHEMA
                    },
                    &schema::NamedValue {
                        name: "y",
                        ty: i32::SCHEMA
                    },
                ])
            }
        );
    }
}
