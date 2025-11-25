use proc_macro2::Span;
use quote::{ToTokens, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Field, Ident, LitInt, Type};

pub fn expand_derive_pipeline_constants(input: &DeriveInput) -> proc_macro::TokenStream {
    if let Data::Struct(ref data) = input.data {
        let struct_name = &input.ident;
        let mod_path = quote!(empa::pipeline_constants);

        let mut errors = Vec::new();
        let mut fields: Vec<ConstantField> = Vec::new();

        for (i, field) in data.fields.iter().enumerate() {
            let field = ConstantField::from_ast(field, i, &mut errors);

            for other in &fields {
                if field.id.is_some() && field.id == other.id {
                    errors.push(syn::Error::new(
                        field.span,
                        format!("cannot declare the same ID as field `{}`", other.name),
                    ));
                }
            }

            fields.push(field);
        }

        let recurse = fields.iter().map(|field| {
            let ty = &field.ty;
            let field_name = &field.name;
            let field_ident = field
                .ident
                .clone()
                .map(|i| i.into_token_stream())
                .unwrap_or(field.position.into_token_stream());
            let span = field.span;

            let pattern = if let Some(id) = field.id {
                quote!(#mod_path::PipelineConstantIdentifier::Number(#id))
            } else {
                quote!(#mod_path::PipelineConstantIdentifier::Name(#field_name))
            };

            quote_spanned!(span=>
                #pattern => Some(<#ty as #mod_path::PipelineConstant>::to_value(&self.#field_ident))
            )
        });

        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

        let impl_block = quote! {
            #[automatically_derived]
            impl #impl_generics #mod_path::PipelineConstants for #struct_name #ty_generics #where_clause {
                fn lookup(
                    &self,
                    identifier: #mod_path::PipelineConstantIdentifier
                ) -> Option<#mod_path::PipelineConstantValue> {
                    match identifier {
                        #(#recurse,)*
                        _ => None
                    }
                }
            }
        };

        let errors = errors.iter().map(|e| e.to_compile_error());

        quote! {
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const _: () = {
                #[allow(unknown_lints)]
                #[allow(clippy::useless_attribute)]
                #[allow(rust_2018_idioms)]

                #impl_block

                #(#errors;)*
            };
        }
        .into()
    } else {
        quote! {
            compile_error!("`PipelineConstants` can only be derived for a struct");
        }
        .into()
    }
}

struct ConstantField {
    ident: Option<Ident>,
    position: usize,
    name: String,
    ty: Type,
    id: Option<u16>,
    span: Span,
}

impl ConstantField {
    pub fn from_ast(ast: &Field, position: usize, errors: &mut Vec<syn::Error>) -> Self {
        let field_name = ast
            .ident
            .clone()
            .map(|i| i.to_string())
            .unwrap_or(position.to_string());

        let mut id_attributes = ast
            .attrs
            .iter()
            .filter(|a| a.path().is_ident("constant_id"));

        let mut id = None;

        if let Some(attr) = id_attributes.next() {
            while let Some(attr) = id_attributes.next() {
                errors.push(syn::Error::new(
                    attr.span(),
                    "attribute may only be declared once per field",
                ));
            }

            if let Ok(lit) = attr.parse_args::<LitInt>() {
                if let Ok(parsed) = lit.base10_parse::<u16>() {
                    id = Some(parsed);
                } else {
                    errors.push(syn::Error::new(
                        lit.span(),
                        "expected an integer between 0 and 65535 (inclusive)",
                    ));
                }
            } else {
                errors.push(syn::Error::new(attr.span(), "expected an integer argument"));
            }
        }

        ConstantField {
            ident: ast.ident.clone(),
            position,
            name: field_name,
            ty: ast.ty.clone(),
            id,
            span: ast.span(),
        }
    }
}
