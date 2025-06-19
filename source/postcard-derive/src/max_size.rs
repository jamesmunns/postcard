use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Data, DeriveInput, Fields, GenericParam,
    Generics,
};

pub fn do_derive_max_size(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let span = input.span();
    let name = input.ident;

    // Add a bound `T: MaxSize` to every type parameter T.
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let sum = max_size_sum(&input.data, span).unwrap_or_else(syn::Error::into_compile_error);

    let expanded = quote! {
        impl #impl_generics ::postcard::experimental::max_size::MaxSize for #name #ty_generics #where_clause {
            const POSTCARD_MAX_SIZE: usize = #sum;
        }
    };

    expanded.into()
}

/// Add a bound `T: MaxSize` to every type parameter T.
fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param
                .bounds
                .push(parse_quote!(::postcard::experimental::max_size::MaxSize));
        }
    }
    generics
}

/// Generate a constant expression that sums up the maximum size of the type.
fn max_size_sum(data: &Data, span: Span) -> Result<TokenStream, syn::Error> {
    match data {
        Data::Struct(data) => Ok(sum_fields(&data.fields)),
        Data::Enum(data) => {
            let variant_count = data.variants.len();

            let recurse = data.variants.iter().map(|v| sum_fields(&v.fields));

            let discriminant_size = varint_size_discriminant(variant_count as u32) as usize;

            // Generate a tree of max expressions.
            let max = recurse.fold(quote!(0), |acc, x| {
                quote! {
                    {
                        let lhs = #acc;
                        let rhs = #x;
                        if lhs > rhs {
                            lhs
                        } else {
                            rhs
                        }
                    }
                }
            });

            Ok(quote! {
                #discriminant_size + #max
            })
        }
        Data::Union(_) => Err(syn::Error::new(
            span,
            "unions are not supported by `postcard::MaxSize`",
        )),
    }
}

fn sum_fields(fields: &Fields) -> TokenStream {
    match fields {
        syn::Fields::Named(fields) => {
            // Expands to an expression like
            //
            //    0 + <Field1Type>::POSTCARD_MAX_SIZE + <Field2Type>::POSTCARD_MAX_SIZE + ...
            //
            // but using fully qualified syntax.

            let recurse = fields.named.iter().map(|f| {
                let ty = &f.ty;
                quote_spanned! { f.span() => <#ty as ::postcard::experimental::max_size::MaxSize>::POSTCARD_MAX_SIZE }
            });

            quote! {
                0 #(+ #recurse)*
            }
        }
        syn::Fields::Unnamed(fields) => {
            let recurse = fields.unnamed.iter().map(|f| {
                let ty = &f.ty;
                quote_spanned! { f.span() => <#ty as ::postcard::experimental::max_size::MaxSize>::POSTCARD_MAX_SIZE }
            });

            quote! {
                0 #(+ #recurse)*
            }
        }
        syn::Fields::Unit => quote!(0),
    }
}

fn varint_size_discriminant(max_n: u32) -> u32 {
    const BITS_PER_BYTE: u32 = 8;
    const BITS_PER_VARINT_BYTE: u32 = 7;

    // How many data bits do we need for `max_n`.
    let bits = core::mem::size_of::<u32>() as u32 * BITS_PER_BYTE - max_n.leading_zeros();

    let roundup_bits = bits + (BITS_PER_VARINT_BYTE - 1);

    // Apply division, using normal "round down" integer division
    roundup_bits / BITS_PER_VARINT_BYTE
}
