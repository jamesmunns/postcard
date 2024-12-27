use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
    parse_quote, punctuated::Punctuated, spanned::Spanned, Data, DeriveInput, Fields, GenericParam,
    Generics, Path, Token,
};

pub fn do_derive_schema(input: DeriveInput) -> syn::Result<TokenStream> {
    let span = input.span();
    let name = &input.ident;

    let mut generator = Generator::new(&input)?;

    // Add a bound `T: Schema` to every type parameter T unless overridden by `#[postcard(bound = "...")]`
    let generics = match generator.bound.take() {
        Some(bounds) => {
            let mut generics = input.generics;
            generics.make_where_clause().predicates.extend(bounds);
            generics
        }
        None => generator.add_trait_bounds(input.generics),
    };
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let ty = generator.generate_type(&input.data, span, name.to_string())?;

    let postcard_schema = &generator.postcard_schema;
    let expanded = quote! {
        impl #impl_generics #postcard_schema::Schema for #name #ty_generics #where_clause {
            const SCHEMA: &'static #postcard_schema::schema::DataModelType = #ty;
        }
    };

    Ok(expanded)
}

struct Generator {
    postcard_schema: Path,
    bound: Option<Punctuated<syn::WherePredicate, Token![,]>>,
}

impl Generator {
    fn new(input: &DeriveInput) -> syn::Result<Self> {
        let mut generator = Self {
            postcard_schema: parse_quote!(::postcard_schema),
            bound: None,
        };
        for attr in &input.attrs {
            if attr.path().is_ident("postcard") {
                attr.parse_nested_meta(|meta| {
                    // #[postcard(crate = path::to::postcard)]
                    if meta.path.is_ident("crate") {
                        generator.postcard_schema = meta.value()?.parse()?;
                        return Ok(());
                    }

                    // #[postcard(bound = "T: Schema")]
                    if meta.path.is_ident("bound") {
                        let bound = meta.value()?.parse::<syn::LitStr>()?;
                        let bound = bound.parse_with(
                            Punctuated::<syn::WherePredicate, Token![,]>::parse_terminated,
                        )?;
                        if generator.bound.is_some() {
                            return Err(meta.error("duplicate #[postcard(bound = \"...\")]"));
                        }
                        generator.bound = Some(bound);
                        return Ok(());
                    }

                    Err(meta.error("unsupported #[postcard] attribute"))
                })?;
            }
        }
        Ok(generator)
    }

    fn generate_type(
        &self,
        data: &Data,
        span: Span,
        name: String,
    ) -> Result<TokenStream, syn::Error> {
        let postcard_schema = &self.postcard_schema;
        match data {
            Data::Struct(data) => {
                let data = self.generate_struct(&data.fields);
                Ok(quote! {
                    &#postcard_schema::schema::DataModelType::Struct{
                        name: #name,
                        data: #data,
                    }
                })
            }
            Data::Enum(data) => {
                let variants = data.variants.iter().map(|v| {
                    let (name, data) = (v.ident.to_string(), self.generate_variants(&v.fields));
                    quote! { #postcard_schema::schema::Variant { name: #name, data: #data } }
                });

                Ok(quote! {
                    &#postcard_schema::schema::DataModelType::Enum {
                        name: #name,
                        variants: &[#(&#variants),*],
                    }
                })
            }
            Data::Union(_) => Err(syn::Error::new(
                span,
                "unions are not supported by `postcard::experimental::schema`",
            )),
        }
    }

    fn generate_struct(&self, fields: &Fields) -> TokenStream {
        let postcard_schema = &self.postcard_schema;
        match fields {
            syn::Fields::Named(fields) => {
                let fields = fields.named.iter().map(|f| {
                    let ty = &f.ty;
                    let name = f.ident.as_ref().unwrap().to_string();
                    quote_spanned!(f.span() => &#postcard_schema::schema::NamedField { name: #name, ty: <#ty as #postcard_schema::Schema>::SCHEMA })
                });
                quote! { #postcard_schema::schema::Data::Struct(&[
                    #( #fields ),*
                ]) }
            }
            syn::Fields::Unnamed(fields) => {
                if fields.unnamed.len() == 1 {
                    let f = fields.unnamed[0].clone();
                    let ty = &f.ty;
                    let qs = quote_spanned!(f.span() => <#ty as #postcard_schema::Schema>::SCHEMA);

                    quote! { #postcard_schema::schema::Data::Newtype(#qs) }
                } else {
                    let fields = fields.unnamed.iter().map(|f| {
                        let ty = &f.ty;
                        quote_spanned!(f.span() => <#ty as #postcard_schema::Schema>::SCHEMA)
                    });
                    quote! { #postcard_schema::schema::Data::Tuple(&[
                        #( #fields ),*
                    ]) }
                }
            }
            syn::Fields::Unit => {
                quote! { #postcard_schema::schema::Data::Unit }
            }
        }
    }

    fn generate_variants(&self, fields: &Fields) -> TokenStream {
        let postcard_schema = &self.postcard_schema;
        match fields {
            syn::Fields::Named(fields) => {
                let fields = fields.named.iter().map(|f| {
                    let ty = &f.ty;
                    let name = f.ident.as_ref().unwrap().to_string();
                    quote_spanned!(f.span() => &#postcard_schema::schema::NamedField { name: #name, ty: <#ty as #postcard_schema::Schema>::SCHEMA })
                });
                quote! { #postcard_schema::schema::Data::Struct(&[
                    #( #fields ),*
                ]) }
            }
            syn::Fields::Unnamed(fields) => {
                if fields.unnamed.len() == 1 {
                    let f = fields.unnamed[0].clone();
                    let ty = &f.ty;
                    let qs = quote_spanned!(f.span() => <#ty as #postcard_schema::Schema>::SCHEMA);

                    quote! { #postcard_schema::schema::Data::Newtype(#qs) }
                } else {
                    let fields = fields.unnamed.iter().map(|f| {
                        let ty = &f.ty;
                        quote_spanned!(f.span() => <#ty as #postcard_schema::Schema>::SCHEMA)
                    });
                    quote! { #postcard_schema::schema::Data::Tuple(&[
                        #( #fields ),*
                    ]) }
                }
            }
            syn::Fields::Unit => {
                quote! { #postcard_schema::schema::Data::Unit }
            }
        }
    }

    /// Add a bound `T: Schema` to every type parameter T.
    fn add_trait_bounds(&self, mut generics: Generics) -> Generics {
        let postcard_schema = &self.postcard_schema;
        for param in &mut generics.params {
            if let GenericParam::Type(ref mut type_param) = *param {
                type_param
                    .bounds
                    .push(parse_quote!(#postcard_schema::Schema));
            }
        }
        generics
    }
}
