use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{Attribute, Data, DeriveInput, Field, Ident, Lit, Meta, NestedMeta, Type};

use crate::error_log::ErrorLog;
use std::cmp::max;
use std::collections::HashMap;

pub fn expand_derive_resources(input: &DeriveInput) -> Result<TokenStream, String> {
    if let Data::Struct(ref data) = input.data {
        let struct_name = &input.ident;
        let mod_path = quote!(empa::resource_binding);
        let mut log = ErrorLog::new();

        let mut resource_fields: HashMap<usize, ResourceField> = HashMap::new();
        let mut max_binding = 0;

        for (position, field) in data.fields.iter().enumerate() {
            match ResourcesField::from_ast(field, position, &mut log) {
                ResourcesField::Resource(resource_field) => {
                    for field in resource_fields.values() {
                        if field.binding == resource_field.binding {
                            log.log_error(format!(
                                "Fields `{}` and `{}` cannot both use binding `{}`.",
                                field.name, resource_field.name, field.binding
                            ));
                        }
                    }

                    max_binding = max(max_binding, resource_field.binding);
                    resource_fields.insert(resource_field.binding, resource_field);
                }
                ResourcesField::Excluded => (),
            };
        }

        let mut bindings = Vec::with_capacity(max_binding as usize);
        let mut entries = Vec::with_capacity(max_binding as usize);

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

            let tokens = if let Some(field) = resource_fields.get(&i) {
                let ty = &field.ty;
                let field_name = field
                    .ident
                    .clone()
                    .map(|i| i.into_token_stream())
                    .unwrap_or(field.position.into_token_stream());
                let span = field.span;

                quote_spanned! {span=>
                    Some(<#ty as #mod_path::Resource>::to_entry(&self.#field_name))
                }
            } else {
                quote!(None)
            };

            entries.push(tokens);
        }

        let iter_len = max_binding + 1;
        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

        let impl_block = quote! {
            #[automatically_derived]
            unsafe impl #impl_generics #mod_path::Resources for #struct_name #ty_generics #where_clause {
                type Layout = (#(#bindings,)*);

                type ToEntries = <[Option<#mod_path::BindGroupEntry>; #iter_len] as IntoIterator>::IntoIter;

                fn to_entries(&self) -> Self::ToEntries {
                    [#(#entries,)*].into_iter()
                }
            }
        };

        let suffix = struct_name.to_string().trim_start_matches("r#").to_owned();
        let dummy_const = Ident::new(
            &format!("_IMPL_RESOURCES_FOR_{}", suffix),
            Span::call_site(),
        );

        let generated = quote! {
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const #dummy_const: () = {
                #[allow(unknown_lints)]
                #[cfg_attr(feature = "cargo-clippy", allow(useless_attribute))]
                #[allow(rust_2018_idioms)]

                use #mod_path::Resource;

                #impl_block
            };
        };

        log.compile().map(|_| generated)
    } else {
        Err("`Resources` can only be derived for a struct.".into())
    }
}

enum ResourcesField {
    Resource(ResourceField),
    Excluded,
}

impl ResourcesField {
    pub fn from_ast(ast: &Field, position: usize, log: &mut ErrorLog) -> Self {
        let field_name = ast
            .ident
            .clone()
            .map(|i| i.to_string())
            .unwrap_or(position.to_string());

        let mut iter = ast.attrs.iter().filter(|a| is_resource_attribute(a));

        if let Some(attr) = iter.next() {
            if iter.next().is_some() {
                log.log_error(format!(
                    "Cannot add more than 1 #[resource] attribute to field `{}`.",
                    field_name
                ));

                return ResourcesField::Excluded;
            }

            let meta_items: Vec<NestedMeta> = match attr.parse_meta() {
                Ok(Meta::List(list)) => list.nested.iter().cloned().collect(),
                Ok(Meta::Path(path)) if path.is_ident("resource") => Vec::new(),
                _ => {
                    log.log_error(format!(
                        "Malformed #[resource] attribute for field `{}`.",
                        field_name
                    ));

                    Vec::new()
                }
            };

            let mut binding = None;
            let mut visibility = Visibility {
                vertex: false,
                fragment: false,
                compute: false,
            };

            for meta_item in meta_items.into_iter() {
                match meta_item {
                    NestedMeta::Meta(Meta::NameValue(m)) if m.path.is_ident("binding") => {
                        if let Lit::Int(i) = &m.lit {
                            if let Ok(value) = i.base10_parse::<usize>() {
                                binding = Some(value);
                            } else {
                                log.log_error(format!(
                                    "Malformed #[resource] attribute for field `{}`: \
                                    expected `binding` to be representable as a u32.",
                                    field_name
                                ));
                            }
                        } else {
                            log.log_error(format!(
                                "Malformed #[resource] attribute for field `{}`: \
                                 expected `binding` to be a positive integer.",
                                field_name
                            ));
                        };
                    }
                    NestedMeta::Meta(Meta::NameValue(ref m)) if m.path.is_ident("visibility") => {
                        if let Lit::Str(n) = &m.lit {
                            for segment in n.value().split('|') {
                                match segment.trim() {
                                    "VERTEX" => {
                                        if visibility.vertex {
                                            log.log_error(format!(
                                                "Malformed #[resource] attribute for field `{}`: \
                                            `visibility` contains `VERTEX` twice.",
                                                field_name
                                            ));

                                            break;
                                        } else {
                                            visibility.vertex = true;
                                        }
                                    }
                                    "FRAGMENT" => {
                                        if visibility.fragment {
                                            log.log_error(format!(
                                                "Malformed #[resource] attribute for field `{}`: \
                                                `visibility` contains `FRAGMENT` twice.",
                                                field_name
                                            ));

                                            break;
                                        } else {
                                            visibility.fragment = true;
                                        }
                                    }
                                    "COMPUTE" => {
                                        if visibility.compute {
                                            log.log_error(format!(
                                                "Malformed #[resource] attribute for field `{}`: \
                                                `visibility` contains `COMPUTE` twice.",
                                                field_name
                                            ));

                                            break;
                                        } else {
                                            visibility.compute = true;
                                        }
                                    }
                                    unknown => {
                                        log.log_error(format!(
                                            "Malformed #[resource] attribute for field `{}`: \
                                             unknown `visiblity` token `{}`; must be `VERTEX`, \
                                             `FRAGMENT` or `COMPUTE, or a combination separated by \
                                             pipes (e.g. `VERTEX|FRAGMENT`).",
                                            field_name, unknown
                                        ));

                                        break;
                                    }
                                }
                            }
                        } else {
                            log.log_error(format!(
                                "Malformed #[resource] attribute for field `{}`: \
                                 expected `visibility` to be a string.",
                                field_name
                            ));
                        };
                    }
                    _ => log.log_error(format!(
                        "Malformed #[resource] attribute for field `{}`: unrecognized \
                         option `{}`.",
                        field_name,
                        meta_item.into_token_stream()
                    )),
                }
            }

            if binding.is_none() {
                log.log_error(format!(
                    "Field `{}` is marked with #[resource], but does not declare a `binding` \
                     index.",
                    field_name
                ));
            }

            if binding.is_some() {
                let binding = binding.unwrap();

                ResourcesField::Resource(ResourceField {
                    name: field_name,
                    ident: ast.ident.clone(),
                    ty: ast.ty.clone(),
                    position,
                    binding,
                    visibility,
                    span: ast.span(),
                })
            } else {
                ResourcesField::Excluded
            }
        } else {
            ResourcesField::Excluded
        }
    }
}

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
    binding: usize,
    visibility: Visibility,
    span: Span,
}

fn is_resource_attribute(attribute: &Attribute) -> bool {
    attribute.path.segments[0].ident == "resource"
}
