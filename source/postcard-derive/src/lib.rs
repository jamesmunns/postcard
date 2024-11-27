mod max_size;
mod schema;

/// Derive the `postcard::MaxSize` trait for a struct or enum.
#[proc_macro_derive(MaxSize)]
pub fn derive_max_size(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    max_size::do_derive_max_size(item)
}

/// Derive the `postcard_schema::Schema` trait for a struct or enum.
#[proc_macro_derive(Schema, attributes(postcard))]
pub fn derive_schema(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    schema::do_derive_schema(item)
}
