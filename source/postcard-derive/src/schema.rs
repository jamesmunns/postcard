use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
    parse_quote, punctuated::Punctuated, spanned::Spanned, Attribute, Data, DeriveInput, Error,
    Fields, GenericParam, Generics, Token,
};

pub fn do_derive_schema(input: DeriveInput) -> Result<TokenStream, Error> {
    let span = input.span();
    let name = input.ident;

    // Add a bound `T: Schema` to every type parameter T.
    let generics = match find_bounds(&input.attrs, &input.generics)? {
        Some(x) => x,
        None => add_trait_bounds(input.generics),
    };
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let ty = generate_type(&input.data, span, name.to_string())?;

    let expanded = quote! {
        impl #impl_generics ::postcard::experimental::schema::Schema for #name #ty_generics #where_clause {
            const SCHEMA: &'static ::postcard::experimental::schema::NamedType = #ty;
        }
    };

    Ok(expanded)
}

fn generate_type(data: &Data, span: Span, name: String) -> Result<TokenStream, Error> {
    let ty = match data {
        Data::Struct(data) => generate_struct(&data.fields),
        Data::Enum(data) => {
            let name = data.variants.iter().map(|v| v.ident.to_string());
            let ty = data.variants.iter().map(|v| generate_variants(&v.fields));

            quote! {
                &::postcard::experimental::schema::SdmTy::Enum(&[
                    #( &::postcard::experimental::schema::NamedVariant { name: #name, ty: #ty } ),*
                ])
            }
        }
        Data::Union(_) => {
            return Err(Error::new(
                span,
                "unions are not supported by `postcard::experimental::schema`",
            ))
        }
    };

    Ok(quote! {
        &::postcard::experimental::schema::NamedType {
            name: #name,
            ty: #ty,
        }
    })
}

fn generate_struct(fields: &Fields) -> TokenStream {
    match fields {
        syn::Fields::Named(fields) => {
            let fields = fields.named.iter().map(|f| {
                let ty = &f.ty;
                let name = f.ident.as_ref().unwrap().to_string();
                quote_spanned!(f.span() => &::postcard::experimental::schema::NamedValue { name: #name, ty: <#ty as ::postcard::experimental::schema::Schema>::SCHEMA })
            });
            quote! { &::postcard::experimental::schema::SdmTy::Struct(&[
                #( #fields ),*
            ]) }
        }
        syn::Fields::Unnamed(fields) => {
            let fields = fields.unnamed.iter().map(|f| {
                let ty = &f.ty;
                quote_spanned!(f.span() => <#ty as ::postcard::experimental::schema::Schema>::SCHEMA)
            });
            quote! { &::postcard::experimental::schema::SdmTy::TupleStruct(&[
                #( #fields ),*
            ]) }
        }
        syn::Fields::Unit => {
            quote! { &::postcard::experimental::schema::SdmTy::UnitStruct }
        }
    }
}

fn generate_variants(fields: &Fields) -> TokenStream {
    match fields {
        syn::Fields::Named(fields) => {
            let fields = fields.named.iter().map(|f| {
                let ty = &f.ty;
                let name = f.ident.as_ref().unwrap().to_string();
                quote_spanned!(f.span() => &::postcard::experimental::schema::NamedValue { name: #name, ty: <#ty as ::postcard::experimental::schema::Schema>::SCHEMA })
            });
            quote! { &::postcard::experimental::schema::SdmTy::StructVariant(&[
                #( #fields ),*
            ]) }
        }
        syn::Fields::Unnamed(fields) => {
            let fields = fields.unnamed.iter().map(|f| {
                let ty = &f.ty;
                quote_spanned!(f.span() => <#ty as ::postcard::experimental::schema::Schema>::SCHEMA)
            });
            quote! { &::postcard::experimental::schema::SdmTy::TupleVariant(&[
                #( #fields ),*
            ]) }
        }
        syn::Fields::Unit => {
            quote! { &::postcard::experimental::schema::SdmTy::UnitVariant }
        }
    }
}

/// Add a bound `T: MaxSize` to every type parameter T.
fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param
                .bounds
                .push(parse_quote!(::postcard::experimental::schema::Schema));
        }
    }
    generics
}

fn find_bounds(attrs: &[Attribute], generics: &Generics) -> Result<Option<Generics>, Error> {
    let mut result = None;
    for attr in attrs {
        if !attr.path.is_ident("postcard") {
            continue;
        }
        let span = attr.span();
        let meta = match attr.parse_meta()? {
            syn::Meta::List(x) => x,
            _ => return Err(Error::new(span, "expected #[postcard(...)]")),
        };
        for meta in meta.nested {
            let meta = match meta {
                syn::NestedMeta::Meta(x) => x,
                _ => return Err(Error::new(span, "expected #[postcard($meta)]")),
            };
            let meta = match meta {
                syn::Meta::NameValue(x) => x,
                _ => return Err(Error::new(span, "expected #[postcard($path = $lit)]")),
            };
            if !meta.path.is_ident("bound") {
                return Err(Error::new(span, "expected #[postcard(bound = $lit)]"));
            }
            if result.is_some() {
                return Err(Error::new(span, "duplicate #[postcard(bound = \"...\")]"));
            }
            let bound = match meta.lit {
                syn::Lit::Str(x) => x,
                _ => return Err(Error::new(span, "expected #[postcard(bound = \"...\")]")),
            };
            let bound =
                bound.parse_with(Punctuated::<syn::WherePredicate, Token![,]>::parse_terminated)?;
            let mut generics = generics.clone();
            generics.make_where_clause().predicates.extend(bound);
            result = Some(generics);
        }
    }
    Ok(result)
}
