use syn::{DeriveInput, parse_macro_input};

mod schema;

/// Derive the `postcard_schema::Schema` trait for a struct or enum.
#[proc_macro_derive(Schema, attributes(postcard, serde))]
pub fn derive_schema(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    schema::do_derive_schema(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
