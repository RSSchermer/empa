use quote::{ToTokens, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput};

pub fn expand_derive_sized(input: &DeriveInput) -> proc_macro::TokenStream {
    if let Data::Struct(data) = &input.data {
        let struct_name = &input.ident;

        let recurse_len = data.fields.iter().map(|field| {
            let ty = &field.ty;
            let span = field.span();

            quote_spanned! {span=>
                <#ty as empa::abi::Sized>::LAYOUT.len()
            }
        });

        let recurse_array = data.fields.iter().enumerate().map(|(position, field)| {
            let ty = &field.ty;
            let ident = field
                .ident
                .clone()
                .map(|i| i.into_token_stream())
                .unwrap_or(syn::Index::from(position).into_token_stream());
            let span = field.span();

            quote_spanned! {span=>
                let base_offset = empa::offset_of!(#struct_name, #ident) as u64;
                let memory_units = <#ty as empa::abi::Sized>::LAYOUT;
                let mut j = 0;

                while j < memory_units.len() {
                    let memory_unit = &memory_units[j];

                    array[i].write(empa::smi::MemoryUnit {
                        offset: base_offset + memory_unit.offset,
                        layout: empa::smi::clone_memory_unit_layout(&memory_unit.layout),
                    });

                    i += 1;
                    j += 1;
                }
            }
        });

        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

        let impl_block = quote! {
            #[automatically_derived]
            unsafe impl #impl_generics empa::abi::Sized for #struct_name #ty_generics #where_clause {
                const LAYOUT: &'static [empa::smi::MemoryUnit] = &{
                    const LEN: usize = #(#recurse_len)+*;

                    let array: std::mem::MaybeUninit<[empa::smi::MemoryUnit; LEN]> =
                        std::mem::MaybeUninit::uninit();
                    let mut array = array.transpose();

                    let mut i = 0;

                    #(#recurse_array)*

                    unsafe {
                        std::mem::MaybeUninit::array_assume_init(array)
                    }
                };
            }
        };

        quote! {
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const _: () = {
                #[allow(unknown_lints)]
                #[allow(clippy::useless_attribute)]
                #[allow(rust_2018_idioms)]

                #impl_block
            };
        }
        .into()
    } else {
        quote! {
            compile_error!("`Sized` can only be derived for a struct");
        }
        .into()
    }
}
