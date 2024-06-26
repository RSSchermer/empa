#![feature(proc_macro_span, track_path)]

mod abi_sized;
mod buffer_usage;
mod error_log;
mod pipeline_constants;
mod resources;
mod shader_source;
mod texture_usage;
mod vertex;

use proc_macro::TokenStream;
use proc_macro2;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro]
#[allow(nonstandard_style)]
pub fn BufferUsages(input: TokenStream) -> TokenStream {
    buffer_usage::expand_buffer_usages(input)
}

#[proc_macro]
#[allow(nonstandard_style)]
pub fn TextureUsages(input: TokenStream) -> TokenStream {
    texture_usage::expand_texture_usages(input)
}

#[proc_macro]
pub fn shader_source(input: TokenStream) -> TokenStream {
    shader_source::expand_shader_source(input)
}

#[proc_macro_derive(PipelineConstants, attributes(constant_id))]
pub fn derive_pipeline_constants(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    pipeline_constants::expand_derive_pipeline_constants(&input)
        .unwrap_or_else(compile_error)
        .into()
}

#[proc_macro_derive(Resources, attributes(resource))]
pub fn derive_resources(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    resources::expand_derive_resources(&input)
        .unwrap_or_else(compile_error)
        .into()
}

#[proc_macro_derive(Sized)]
pub fn derive_sized(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    abi_sized::expand_derive_sized(&input)
        .unwrap_or_else(compile_error)
        .into()
}

#[proc_macro_derive(Vertex, attributes(vertex_per_instance, vertex_attribute))]
pub fn derive_vertex(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let tokens = vertex::expand_derive_vertex(&input)
        .unwrap_or_else(compile_error)
        .into();

    tokens
}

fn compile_error(message: String) -> proc_macro2::TokenStream {
    quote! {
        compile_error!(#message);
    }
}
