use indexmap::IndexMap;
use proc_macro2::Span;
use quote::{ToTokens, quote, quote_spanned};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Expr, Field, Ident, Lit, Meta, Token, Type};

pub fn expand_derive_vertex(input: &DeriveInput) -> proc_macro::TokenStream {
    if let Data::Struct(ref data) = input.data {
        let struct_name = &input.ident;
        let mod_path = quote!(empa::render_pipeline);

        let mut errors = Vec::new();
        let mut per_instance = false;

        for attribute in &input.attrs {
            if attribute.path().is_ident("vertex_per_instance") {
                if let Err(err) = attribute.meta.require_path_only() {
                    errors.push(err);
                } else {
                    per_instance = true;
                }
            }
        }

        let step_mode = if per_instance {
            quote!(#mod_path::VertexStepMode::Instance)
        } else {
            quote!(#mod_path::VertexStepMode::Vertex)
        };

        let mut vertex_attributes: IndexMap<u32, AttributeField> = Default::default();

        for (position, field) in data.fields.iter().enumerate() {
            let field = VertexField::from_ast(field, position, &mut errors);

            if let VertexField::Attribute(attr) = field {
                if let Some(other) = vertex_attributes.get(&attr.location) {
                    errors.push(syn::Error::new(
                        attr.span,
                        format!(
                            "cannot declare the same location as field `{}`",
                            other.field_name()
                        ),
                    ));
                }

                vertex_attributes.insert(attr.location, attr);
            }
        }

        vertex_attributes.sort_keys();

        let recurse = vertex_attributes.values().map(|a| {
            let field_name = a
                .ident
                .clone()
                .map(|i| i.into_token_stream())
                .unwrap_or(a.position.into_token_stream());
            let location = a.location as u32;
            let ty = &a.ty;
            let span = a.span;
            let format_kind = {
                let ident = Ident::new(a.format.as_str(), Span::call_site()).into_token_stream();

                quote_spanned!(span=> {
                    {
                        assert_format_compatible::<#ty, #ident>();

                        <#ident as VertexAttributeFormat>::FORMAT
                    }
                })
            };

            quote! {
                #mod_path::VertexAttribute {
                    shader_location: #location,
                    format: #format_kind,
                    offset: empa::offset_of!(#struct_name, #field_name)
                }
            }
        });

        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

        let impl_block = quote! {
            const ATTRIBUTES: &'static [#mod_path::VertexAttribute] = &[
                #(#recurse),*
            ];

            #[automatically_derived]
            unsafe impl #impl_generics #mod_path::Vertex for #struct_name #ty_generics #where_clause {
                const LAYOUT: #mod_path::VertexBufferLayout<'static> = #mod_path::VertexBufferLayout {
                    array_stride: std::mem::size_of::<#struct_name #ty_generics>(),
                    step_mode: #step_mode,
                    attributes: std::borrow::Cow::Borrowed(ATTRIBUTES)
                };
            }
        };

        let errors = errors.iter().map(|err| err.to_compile_error());

        quote! {
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const _: () = {
                #[allow(unknown_lints)]
                #[allow(clippy::useless_attribute)]
                use #mod_path::vertex_attribute::*;

                const fn assert_format_compatible<T, F>()
                where
                    T: #mod_path::vertex_attribute::VertexAttributeFormatCompatible<F>,
                    F: #mod_path::vertex_attribute::VertexAttributeFormat
                {}

                #impl_block

                #(#errors;)*
            };
        }
        .into()
    } else {
        quote! {
            compile_error!("`Vertex` can only be derived for a struct");
        }
        .into()
    }
}

enum VertexField {
    Attribute(AttributeField),
    Excluded,
}

impl VertexField {
    pub fn from_ast(ast: &Field, position: usize, errors: &mut Vec<syn::Error>) -> Self {
        let mut vertex_attributes = ast
            .attrs
            .iter()
            .filter(|a| a.path().is_ident("vertex_attribute"));

        if let Some(attr) = vertex_attributes.next() {
            while let Some(attr) = vertex_attributes.next() {
                errors.push(syn::Error::new(
                    attr.span(),
                    "attribute may only be declared once per field",
                ));
            }

            let mut location = None;
            let mut format = None;

            match attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) {
                Ok(nested) => {
                    for meta in nested {
                        if meta.path().is_ident("location") {
                            match parse_location(meta) {
                                Ok(v) => {
                                    location = Some(v);
                                }
                                Err(err) => errors.push(err),
                            }
                        } else if meta.path().is_ident("format") {
                            match parse_format(meta) {
                                Ok(v) => {
                                    format = Some(v);
                                }
                                Err(err) => errors.push(err),
                            }
                        }
                    }
                }
                Err(err) => errors.push(err),
            }

            if location.is_none() {
                errors.push(syn::Error::new(
                    attr.span(),
                    "vertex attribute must declare a `location` argument",
                ));
            }

            if format.is_none() {
                errors.push(syn::Error::new(
                    attr.span(),
                    "vertex attribute must declare a `format` argument",
                ));
            }

            if location.is_some() && format.is_some() {
                let location = location.unwrap();
                let format = format.unwrap();

                return VertexField::Attribute(AttributeField {
                    ident: ast.ident.clone(),
                    ty: ast.ty.clone(),
                    position,
                    location,
                    format,
                    span: ast.span(),
                });
            }
        }

        VertexField::Excluded
    }
}

fn parse_location(meta: Meta) -> syn::Result<u32> {
    let meta = meta.require_name_value()?;

    if let Expr::Lit(expr) = &meta.value
        && let Lit::Int(lit) = &expr.lit
    {
        lit.base10_parse::<u32>()
    } else {
        Err(syn::Error::new(
            meta.value.span(),
            "expected an integer literal",
        ))
    }
}

fn parse_format(meta: Meta) -> syn::Result<String> {
    let meta = meta.require_name_value()?;

    if let Expr::Lit(expr) = &meta.value
        && let Lit::Str(lit) = &expr.lit
    {
        Ok(lit.value())
    } else {
        Err(syn::Error::new(
            meta.value.span(),
            "expected a format string",
        ))
    }
}

struct AttributeField {
    ident: Option<Ident>,
    ty: Type,
    position: usize,
    location: u32,
    format: String,
    span: Span,
}

impl AttributeField {
    fn field_name(&self) -> String {
        if let Some(ident) = &self.ident {
            ident.to_string()
        } else {
            format!("{}", self.position)
        }
    }
}
