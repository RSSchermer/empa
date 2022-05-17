use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{Attribute, Data, DeriveInput, Field, Ident, Lit, Meta, NestedMeta, Type, Error};

use crate::error_log::ErrorLog;

pub fn expand_derive_pipeline_constants(input: &DeriveInput) -> Result<TokenStream, String> {
    if let Data::Struct(ref data) = input.data {
        let struct_name = &input.ident;
        let mod_path = quote!(empa::pipeline_constants);
        let mut log = ErrorLog::new();

        let mut fields: Vec<ConstantField> = Vec::new();

        'outer: for (i, field) in data.fields.iter().enumerate() {
            let field = ConstantField::from_ast(field, i, &mut log);

            for f in fields.iter() {
                if field.id == f.id {
                    log.log_error(format!(
                        "Fields `{}` and `{}` declare the same ID.",
                        &f.name,
                        &field.name
                    ));

                    continue 'outer;
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
                quote!(#mod_path::PipelineIdentifier::Number(#id))
            } else {
                quote!(#mod_path::PipelineIdentifier::Name(#field_name))
            };

            quote_spanned!(span=> {
                #pattern => Some(self.#field_ident)
            })
        });

        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

        let impl_block = quote! {
            #[automatically_derived]
            unsafe impl #impl_generics #mod_path::PipelineConstants for #struct_name #ty_generics #where_clause {
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

        let suffix = struct_name.to_string().trim_start_matches("r#").to_owned();
        let dummy_const = Ident::new(&format!("_IMPL_PIPELINE_CONSTANTS_FOR_{}", suffix), Span::call_site());

        let generated = quote! {
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const #dummy_const: () = {
                #[allow(unknown_lints)]
                #[cfg_attr(feature = "cargo-clippy", allow(useless_attribute))]
                #[allow(rust_2018_idioms)]

                #impl_block
            };
        };

        log.compile().map(|_| generated)
    } else {
        Err("`PipelineConstants` can only be derived for a struct.".into())
    }
}

struct ConstantField {
    ident: Option<Ident>,
    position: usize,
    name: String,
    ty: Type,
    id: Option<u32>,
    span: Span,
}

impl ConstantField {
    pub fn from_ast(ast: &Field, position: usize, log: &mut ErrorLog) -> Self {
        let field_name = ast
            .ident
            .clone()
            .map(|i| i.to_string())
            .unwrap_or(position.to_string());

        let id_attributes: Vec<&Attribute> = ast
            .attrs
            .iter()
            .filter(|a| a.path.is_ident("constant_id"))
            .collect();

        if id_attributes.len() > 1 {
            log.log_error(format!(
                "Multiple #[constant_id] attributes for field `{}`.",
                field_name
            ));
        }

        let mut id = None;

        if let Some(attr) = id_attributes.first() {
            match attr.parse_meta() {
                Ok(Meta::List(meta)) => {
                    if meta.nested.len() != 1 {
                        log.log_error(format!(
                            "Malformed #[constant_id] attribute for field `{}`; expected a single \
                            integer.",
                            field_name
                        ));
                    } else {
                        let nested = meta.nested.first().unwrap();

                        if let NestedMeta::Lit(Lit::Int(lit)) = nested {
                            if let Ok(parsed) = lit.base10_parse::<u32>() {
                                id = Some(parsed);
                            } else {
                                log.log_error(format!(
                                    "Malformed #[constant_id] attribute for field `{}`; expected \
                                    ID to be a positive integer.",
                                    field_name
                                ));
                            }
                        } else {
                            log.log_error(format!(
                                "Malformed #[constant_id] attribute for field `{}`; expected ID to \
                                be an integer literal.",
                                field_name
                            ));
                        }
                    }
                }
                _ => {
                    log.log_error(format!(
                        "Malformed #[constant_id] attribute for field `{}`.",
                        field_name
                    ));
                }
            }
        }

        ConstantField {
            ident: ast.ident.clone(),
            position,
            name: field_name,
            ty: ast.ty.clone(),
            id,
            span: ast.span()
        }
    }
}
