use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Data, DeriveInput, Fields, GenericParam,
    Generics, Path,
};

pub fn do_derive_schema(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let span = input.span();
    let name = &input.ident;

    let generator = match Generator::new(&input) {
        Ok(generator) => generator,
        Err(err) => return err.into_compile_error().into(),
    };

    // Add a bound `T: Schema` to every type parameter T.
    let generics = generator.add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let ty = generator
        .generate_type(&input.data, span, name.to_string())
        .unwrap_or_else(syn::Error::into_compile_error);

    let postcard_schema = &generator.postcard_schema;
    let expanded = quote! {
        impl #impl_generics #postcard_schema::Schema for #name #ty_generics #where_clause {
            const SCHEMA: &'static #postcard_schema::schema::NamedType = #ty;
        }
    };

    expanded.into()
}

struct Generator {
    postcard_schema: Path,
}

impl Generator {
    fn new(input: &DeriveInput) -> syn::Result<Self> {
        let mut generator = Self {
            postcard_schema: parse_quote!(::postcard_schema),
        };
        for attr in &input.attrs {
            if attr.path().is_ident("postcard") {
                attr.parse_nested_meta(|meta| {
                    // #[postcard(crate = path::to::postcard)]
                    if meta.path.is_ident("crate") {
                        generator.postcard_schema = meta.value()?.parse()?;
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
        let ty = match data {
            Data::Struct(data) => self.generate_struct(&data.fields),
            Data::Enum(data) => {
                let name = data.variants.iter().map(|v| v.ident.to_string());
                let ty = data
                    .variants
                    .iter()
                    .map(|v| self.generate_variants(&v.fields));

                quote! {
                    &#postcard_schema::schema::DataModelType::Enum(&[
                        #( &#postcard_schema::schema::NamedVariant { name: #name, ty: #ty } ),*
                    ])
                }
            }
            Data::Union(_) => {
                return Err(syn::Error::new(
                    span,
                    "unions are not supported by `postcard::experimental::schema`",
                ))
            }
        };

        Ok(quote! {
            &#postcard_schema::schema::NamedType {
                name: #name,
                ty: #ty,
            }
        })
    }

    fn generate_struct(&self, fields: &Fields) -> TokenStream {
        let postcard_schema = &self.postcard_schema;
        match fields {
            syn::Fields::Named(fields) => {
                let fields = fields.named.iter().map(|f| {
                let ty = &f.ty;
                let name = f.ident.as_ref().unwrap().to_string();
                quote_spanned!(f.span() => &#postcard_schema::schema::NamedValue { name: #name, ty: <#ty as #postcard_schema::Schema>::SCHEMA })
            });
                quote! { &#postcard_schema::schema::DataModelType::Struct(&[
                    #( #fields ),*
                ]) }
            }
            syn::Fields::Unnamed(fields) => {
                if fields.unnamed.len() == 1 {
                    let f = fields.unnamed[0].clone();
                    let ty = &f.ty;
                    let qs = quote_spanned!(f.span() => <#ty as #postcard_schema::Schema>::SCHEMA);

                    quote! { &#postcard_schema::schema::DataModelType::NewtypeStruct(#qs) }
                } else {
                    let fields = fields.unnamed.iter().map(|f| {
                        let ty = &f.ty;
                        quote_spanned!(f.span() => <#ty as #postcard_schema::Schema>::SCHEMA)
                    });
                    quote! { &#postcard_schema::schema::DataModelType::TupleStruct(&[
                        #( #fields ),*
                    ]) }
                }
            }
            syn::Fields::Unit => {
                quote! { &#postcard_schema::schema::DataModelType::UnitStruct }
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
                quote_spanned!(f.span() => &#postcard_schema::schema::NamedValue { name: #name, ty: <#ty as #postcard_schema::Schema>::SCHEMA })
            });
                quote! { &#postcard_schema::schema::DataModelVariant::StructVariant(&[
                    #( #fields ),*
                ]) }
            }
            syn::Fields::Unnamed(fields) => {
                if fields.unnamed.len() == 1 {
                    let f = fields.unnamed[0].clone();
                    let ty = &f.ty;
                    let qs = quote_spanned!(f.span() => <#ty as #postcard_schema::Schema>::SCHEMA);

                    quote! { &#postcard_schema::schema::DataModelVariant::NewtypeVariant(#qs) }
                } else {
                    let fields = fields.unnamed.iter().map(|f| {
                        let ty = &f.ty;
                        quote_spanned!(f.span() => <#ty as #postcard_schema::Schema>::SCHEMA)
                    });
                    quote! { &#postcard_schema::schema::DataModelVariant::TupleVariant(&[
                        #( #fields ),*
                    ]) }
                }
            }
            syn::Fields::Unit => {
                quote! { &#postcard_schema::schema::DataModelVariant::UnitVariant }
            }
        }
    }

    /// Add a bound `T: MaxSize` to every type parameter T.
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
