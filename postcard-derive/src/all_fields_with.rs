use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::*;
use syn::punctuated::*;
use syn::*;

pub fn all_fields_with(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let gen = match modify_ast_to_add_with_attr(attr.into(), ast) {
        Ok(g) => g,
        Err(e) => return e.to_compile_error().into(),
    };
    gen.into_token_stream().into()
}

fn modify_ast_to_add_with_attr(with: TokenStream, ast: DeriveInput) -> Result<impl ToTokens> {
    let new_data: Data = match ast.data {
        syn::Data::Struct(DataStruct {
            struct_token,
            fields,
            semi_token,
        }) => match fields {
            fields => modify_fields_to_add_with_attr(with, struct_token, semi_token, fields)?,
        },
        data => data,
    };
    Ok(DeriveInput {
        attrs: ast.attrs,
        vis: ast.vis,
        ident: ast.ident,
        generics: ast.generics,
        data: new_data,
    })
}

pub fn modify_fields_to_add_with_attr(
    with: TokenStream,
    struct_token: syn::token::Struct,
    semi_token: Option<syn::token::Semi>,
    fields: Fields,
) -> Result<Data> {
    let mut fields = fields;
    let attr = create_field_attribute(with);
    fields.iter_mut().for_each(|f| f.attrs.push(attr.clone()));
    Ok(Data::Struct(DataStruct {
        struct_token,
        fields: fields,
        semi_token,
    }))
}

fn create_field_attribute(with: TokenStream) -> Attribute {
    let stream = quote!((with = #with));
    let mut segments: Punctuated<PathSegment, Token![::]> = Punctuated::new();
    segments.push(PathSegment {
        ident: Ident::new("serde", Span::call_site()),
        arguments: PathArguments::None,
    });
    Attribute {
        pound_token: syn::token::Pound::default(),
        style: syn::AttrStyle::Outer,
        bracket_token: syn::token::Bracket::default(),
        path: Path {
            leading_colon: None,
            segments,
        },
        tokens: stream,
    }
}
