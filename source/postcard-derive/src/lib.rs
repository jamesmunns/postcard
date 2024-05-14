use syn::{parse_macro_input, DeriveInput};

mod max_size;
mod schema;

/// Derive the `postcard::MaxSize` trait for a struct or enum.
#[proc_macro_derive(MaxSize)]
pub fn derive_max_size(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    max_size::do_derive_max_size(item)
}

/// Derive the `postcard::Schema` trait for a struct or enum.
#[proc_macro_derive(Schema, attributes(postcard))]
pub fn derive_schema(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    schema::do_derive_schema(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
