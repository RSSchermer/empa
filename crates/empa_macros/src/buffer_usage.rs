use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse, parse_macro_input, Ident, Token};

struct UsageFlags {
    query_resolve: bool,
    indirect: bool,
    storage_binding: bool,
    uniform_binding: bool,
    vertex: bool,
    index: bool,
    copy_dst: bool,
    copy_src: bool,
    map_write: bool,
    map_read: bool,
}

impl UsageFlags {
    fn try_add_flag(&mut self, flag_token: Ident) -> syn::Result<()> {
        if flag_token == "Indirect" {
            self.indirect = true;
        } else if flag_token == "StorageBinding" {
            self.storage_binding = true;
        } else if flag_token == "UniformBinding" {
            self.uniform_binding = true;
        } else if flag_token == "Vertex" {
            self.vertex = true;
        } else if flag_token == "Index" {
            self.index = true;
        } else if flag_token == "CopyDst" {
            self.copy_dst = true;
        } else if flag_token == "CopySrc" {
            self.copy_src = true;
        } else if flag_token == "MapWrite" {
            self.map_write = true;
        } else if flag_token == "MapRead" {
            self.map_read = true;
        } else {
            return Err(parse::Error::new(
                flag_token.span(),
                "unknown usage flag, valid flags are: Indirect, StorageBinding, UniformBinding, \
         Vertex, Index, CopyDst, CopySrc, MapWrite, MapRead",
            ));
        }

        Ok(())
    }
}

impl Parse for UsageFlags {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut usage_flags = UsageFlags {
            query_resolve: false,
            indirect: false,
            storage_binding: false,
            uniform_binding: false,
            vertex: false,
            index: false,
            copy_dst: false,
            copy_src: false,
            map_write: false,
            map_read: false,
        };

        if input.is_empty() {
            return Ok(usage_flags);
        }

        let next = input.parse()?;

        usage_flags.try_add_flag(next)?;

        while !input.is_empty() {
            input.parse::<Token!(|)>()?;

            let next = input.parse()?;

            usage_flags.try_add_flag(next)?;
        }

        Ok(usage_flags)
    }
}

pub fn expand_buffer_usages(input: TokenStream) -> TokenStream {
    let usage_flags: UsageFlags = parse_macro_input!(input);

    let buffer_mod = quote!(empa::buffer);
    let type_flag_mod = quote!(empa::type_flag);

    let query_resolve = if usage_flags.query_resolve {
        quote!(#type_flag_mod::X)
    } else {
        quote!(#type_flag_mod::O)
    };
    let indirect = if usage_flags.indirect {
        quote!(#type_flag_mod::X)
    } else {
        quote!(#type_flag_mod::O)
    };
    let storage_binding = if usage_flags.storage_binding {
        quote!(#type_flag_mod::X)
    } else {
        quote!(#type_flag_mod::O)
    };
    let uniform_binding = if usage_flags.uniform_binding {
        quote!(#type_flag_mod::X)
    } else {
        quote!(#type_flag_mod::O)
    };
    let vertex = if usage_flags.vertex {
        quote!(#type_flag_mod::X)
    } else {
        quote!(#type_flag_mod::O)
    };
    let index = if usage_flags.index {
        quote!(#type_flag_mod::X)
    } else {
        quote!(#type_flag_mod::O)
    };
    let copy_dst = if usage_flags.copy_dst {
        quote!(#type_flag_mod::X)
    } else {
        quote!(#type_flag_mod::O)
    };
    let copy_src = if usage_flags.copy_src {
        quote!(#type_flag_mod::X)
    } else {
        quote!(#type_flag_mod::O)
    };
    let map_write = if usage_flags.map_write {
        quote!(#type_flag_mod::X)
    } else {
        quote!(#type_flag_mod::O)
    };
    let map_read = if usage_flags.map_read {
        quote!(#type_flag_mod::X)
    } else {
        quote!(#type_flag_mod::O)
    };

    let result = quote! {
        #buffer_mod::Usages<
            #query_resolve,
            #indirect,
            #storage_binding,
            #uniform_binding,
            #vertex,
            #index,
            #copy_dst,
            #copy_src,
            #map_write,
            #map_read,
        >
    };

    result.into()
}
