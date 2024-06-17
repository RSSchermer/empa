use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse, parse_macro_input, Ident, Token};

struct UsageFlags {
    render_attachment: bool,
    storage_binding: bool,
    texture_binding: bool,
    copy_dst: bool,
    copy_src: bool,
}

impl UsageFlags {
    fn try_add_flag(&mut self, flag_token: Ident) -> syn::Result<()> {
        if flag_token == "RenderAttachment" {
            self.render_attachment = true;
        } else if flag_token == "StorageBinding" {
            self.storage_binding = true;
        } else if flag_token == "TextureBinding" {
            self.texture_binding = true;
        } else if flag_token == "CopyDst" {
            self.copy_dst = true;
        } else if flag_token == "CopySrc" {
            self.copy_src = true;
        } else {
            return Err(parse::Error::new(
                flag_token.span(),
                "unknown usage flag, valid flags are: RenderAttachment, StorageBinding, \
         TextureBinding, CopyDst, CopySrc",
            ));
        }

        Ok(())
    }
}

impl Parse for UsageFlags {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut usage_flags = UsageFlags {
            render_attachment: false,
            storage_binding: false,
            texture_binding: false,
            copy_dst: false,
            copy_src: false,
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

pub fn expand_texture_usages(input: TokenStream) -> TokenStream {
    let usage_flags: UsageFlags = parse_macro_input!(input);

    let texture_mod = quote!(empa::texture);
    let type_flag_mod = quote!(empa::type_flag);

    let render_attachment = if usage_flags.render_attachment {
        quote!(#type_flag_mod::X)
    } else {
        quote!(#type_flag_mod::O)
    };
    let storage_binding = if usage_flags.storage_binding {
        quote!(#type_flag_mod::X)
    } else {
        quote!(#type_flag_mod::O)
    };
    let texture_binding = if usage_flags.texture_binding {
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

    let result = quote! {
        #texture_mod::Usages<
            #render_attachment,
            #storage_binding,
            #texture_binding,
            #copy_dst,
            #copy_src,
        >
    };

    result.into()
}
