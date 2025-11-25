use std::cmp::max;

use indexmap::IndexMap;
use proc_macro2::Span;
use quote::{ToTokens, quote, quote_spanned};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Expr, Field, Ident, Lit, Meta, Token, Type};

pub fn expand_derive_resources(input: &DeriveInput) -> proc_macro::TokenStream {
    if let Data::Struct(ref data) = input.data {
        let struct_name = &input.ident;
        let mod_path = quote!(empa::resource_binding);

        let mut errors = Vec::new();
        let mut resource_fields: IndexMap<u32, ResourceField> = Default::default();
        let mut max_binding = 0;

        for (position, field) in data.fields.iter().enumerate() {
            match ResourcesField::from_ast(field, position, &mut errors) {
                ResourcesField::Resource(resource_field) => {
                    if let Some(other) = resource_fields.get(&resource_field.binding) {
                        errors.push(syn::Error::new(
                            resource_field.span,
                            format!(
                                "cannot declare the same binding index as field `{}`",
                                other.name
                            ),
                        ));
                    }

                    max_binding = max(max_binding, resource_field.binding);
                    resource_fields.insert(resource_field.binding, resource_field);
                }
                ResourcesField::Excluded => (),
            };
        }

        resource_fields.sort_keys();

        let mut bindings = Vec::new();

        for i in 0..=max_binding {
            let tokens = if let Some(field) = resource_fields.get(&i) {
                let ty = &field.ty;
                let span = field.span;

                let vertex_visible = if field.visibility.vertex {
                    quote!(X)
                } else {
                    quote!(O)
                };

                let fragment_visible = if field.visibility.fragment {
                    quote!(X)
                } else {
                    quote!(O)
                };

                let compute_visible = if field.visibility.compute {
                    quote!(X)
                } else {
                    quote!(O)
                };

                quote_spanned! {span=>
                    <<#ty as #mod_path::Resource>::Binding as #mod_path::typed_bind_group_entry::TypedSlotBinding>::WithVisibility<
                        #mod_path::typed_bind_group_entry::ShaderStages<
                            empa::type_flag::#compute_visible,
                            empa::type_flag::#fragment_visible,
                            empa::type_flag::#vertex_visible,
                        >
                    >
                }
            } else {
                quote!(())
            };

            bindings.push(tokens);
        }

        let mut entries = Vec::new();

        for (binding, field) in resource_fields.iter() {
            let ty = &field.ty;
            let field_name = field
                .ident
                .clone()
                .map(|i| i.into_token_stream())
                .unwrap_or(field.position.into_token_stream());
            let span = field.span;

            let tokens = quote_spanned! {span=>
                #mod_path::BindGroupEntry {
                    binding: #binding as u32,
                    resource: <#ty as #mod_path::Resource>::to_encoding(&self.#field_name)
                }
            };

            entries.push(tokens);
        }

        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
        let len = entries.len();

        let impl_block = quote! {
            #[automatically_derived]
            unsafe impl #impl_generics #mod_path::Resources for #struct_name #ty_generics #where_clause {
                type Layout = (#(#bindings,)*);

                type ToEntries<'__a> = [#mod_path::BindGroupEntry<'__a>; #len] where Self: '__a;

                fn to_entries<'__a>(&'__a self) -> Self::ToEntries<'__a> {
                    [#(#entries,)*]
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
            compile_error!("`Resources` can only be derived for a struct");
        }
        .into()
    }
}

enum ResourcesField {
    Resource(ResourceField),
    Excluded,
}

impl ResourcesField {
    pub fn from_ast(ast: &Field, position: usize, errors: &mut Vec<syn::Error>) -> Self {
        let field_name = ast
            .ident
            .clone()
            .map(|i| i.to_string())
            .unwrap_or(position.to_string());

        let mut resource_attributes = ast.attrs.iter().filter(|a| a.path().is_ident("resource"));

        if let Some(attr) = resource_attributes.next() {
            while let Some(attr) = resource_attributes.next() {
                errors.push(syn::Error::new(
                    attr.span(),
                    "attribute may only be declared once per field",
                ));
            }

            let mut binding = None;
            let mut visibility = Visibility {
                vertex: false,
                fragment: false,
                compute: false,
            };

            match attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) {
                Ok(nested) => {
                    for meta in nested {
                        if meta.path().is_ident("binding") {
                            match parse_binding(meta) {
                                Ok(v) => {
                                    binding = Some(v);
                                }
                                Err(err) => errors.push(err),
                            }
                        } else if meta.path().is_ident("visibility") {
                            match parse_visibility(meta) {
                                Ok(v) => {
                                    visibility = v;
                                }
                                Err(err) => errors.push(err),
                            }
                        }
                    }
                }
                Err(err) => errors.push(err),
            }

            if let Some(binding) = binding {
                return ResourcesField::Resource(ResourceField {
                    name: field_name,
                    ident: ast.ident.clone(),
                    ty: ast.ty.clone(),
                    position,
                    binding,
                    visibility,
                    span: ast.span(),
                });
            } else {
                errors.push(syn::Error::new(
                    attr.span(),
                    "resource must declare a `binding` argument",
                ));
            }
        }

        ResourcesField::Excluded
    }
}

fn parse_binding(meta: Meta) -> syn::Result<u32> {
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

fn parse_visibility(meta: Meta) -> syn::Result<Visibility> {
    let meta = meta.require_name_value()?;

    if let Expr::Lit(expr) = &meta.value
        && let Lit::Str(lit) = &expr.lit
    {
        let value = lit.value();

        let mut visibility = Visibility::default();

        for segment in value.split("|") {
            match segment.trim() {
                "VERTEX" => {
                    if visibility.vertex {
                        return Err(syn::Error::new(lit.span(), "contains `VERTEX` twice"));
                    } else {
                        visibility.vertex = true;
                    }
                }
                "FRAGMENT" => {
                    if visibility.fragment {
                        return Err(syn::Error::new(lit.span(), "contains `FRAGMENT` twice"));
                    } else {
                        visibility.fragment = true;
                    }
                }
                "COMPUTE" => {
                    if visibility.compute {
                        return Err(syn::Error::new(lit.span(), "contains `COMPUTE` twice"));
                    } else {
                        visibility.compute = true;
                    }
                }
                v => {
                    return Err(syn::Error::new(
                        lit.span(),
                        format!(
                            "unknown value `{}`; expected one or more of VERTEX, FRAGMENT, or \
                            COMPUTE separated by |",
                            v
                        ),
                    ));
                }
            }
        }

        Ok(visibility)
    } else {
        Err(syn::Error::new(
            meta.value.span(),
            "expected a string literal",
        ))
    }
}

#[derive(Clone, Copy, Default)]
struct Visibility {
    vertex: bool,
    fragment: bool,
    compute: bool,
}

struct ResourceField {
    name: String,
    ident: Option<Ident>,
    ty: Type,
    position: usize,
    binding: u32,
    visibility: Visibility,
    span: Span,
}
