use std::env;
use std::path::Path;

use empa_reflect::{
    BindingType, ConstantIdentifier, ConstantType, EntryPointBinding, EntryPointBindingType,
    Interpolation, MemoryUnit, MemoryUnitLayout, Sampling, ShaderSource, ShaderStage,
    SizedBufferLayout, StorageTextureFormat, TexelType, UnsizedBufferLayout,
};
use include_preprocessor::{preprocess, PathTracker, SearchPaths};
use proc_macro::{tracked_path, Span, TokenStream};
use quote::quote;
use syn::{parse_macro_input, LitStr};

struct ProcMacroPathTracker;

impl PathTracker for ProcMacroPathTracker {
    fn track(&mut self, path: &Path) {
        tracked_path::path(path.to_str().expect("cannot track non-unicode path"));
    }
}

pub fn expand_shader_source(input: TokenStream) -> TokenStream {
    let path = parse_macro_input!(input as LitStr);

    let span = Span::call_site();
    let source_path = span.source_file().path();
    let source_dir = source_path.parent().unwrap();

    let mut search_paths = SearchPaths::new();
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    search_paths.push_base_path(cargo_manifest_dir);

    let source_join = source_dir.join(path.value());

    let output = if source_join.is_file() {
        let buffer = String::new();

        preprocess(source_join, search_paths, buffer, &mut ProcMacroPathTracker).unwrap()
    } else {
        panic!("Entry (`{:?}`) point is not a file!", source_join);
    };

    let source_token = LitStr::new(&output, Span::call_site().into());

    let shader_source = match ShaderSource::parse(output.clone()) {
        Ok(shader_source) => shader_source,
        Err(err) => {
            panic!("{}", err.emit_to_string(&output));
        }
    };

    let mod_path = quote!(empa::shader_module);

    let resource_bindings = shader_source.resource_bindings().iter().map(|b| {
        let group = b.group();
        let binding = b.binding();
        let binding_type = binding_type_tokens(b.binding_type());

        quote! {
            #mod_path::StaticResourceBinding {
                group: #group,
                binding: #binding,
                binding_type: #binding_type
            }
        }
    });

    let constants = shader_source.constants().iter().map(|c| {
        let identifier = match c.identifier() {
            ConstantIdentifier::Number(n) => {
                quote!(empa::pipeline_constants::PipelineConstantIdentifier::Number(#n))
            }
            ConstantIdentifier::Name(n) => {
                quote!(empa::pipeline_constants::PipelineConstantIdentifier::Name(#n))
            }
        };
        let constant_type = constant_type_tokens(c.constant_type());
        let required = c.required();

        quote! {
            #mod_path::StaticConstantDescriptor {
                identifier: #identifier,
                constant_type: #constant_type,
                required: #required,
            }
        }
    });

    let entry_points = shader_source.entry_points().iter().map(|e| {
        let name = e.name();
        let stage = shader_stage_tokens(e.stage());
        let input_bindings = e.input_bindings().iter().map(entry_point_binding_tokens);
        let output_bindings = e.output_bindings().iter().map(entry_point_binding_tokens);

        quote! {
            #mod_path::StaticEntryPoint {
                name: #name,
                stage: #stage,
                input_bindings: &[#(#input_bindings),*],
                output_bindings: &[#(#output_bindings),*],
            }
        }
    });

    let result = quote! {
        #mod_path::ShaderSource::from_static(#mod_path::StaticShaderSource {
            source: #source_token,
            resource_bindings: &[#(#resource_bindings),*],
            constants: &[#(#constants),*],
            entry_points: &[#(#entry_points),*]
        })
    };

    result.into()
}

fn binding_type_tokens(binding_type: &BindingType) -> proc_macro2::TokenStream {
    let mod_path = quote!(empa::resource_binding);

    match binding_type {
        BindingType::Texture1D(texel_type) => {
            let texel_type = texel_type_tokens(*texel_type);

            quote!(#mod_path::BindingType::Texture1D(#texel_type))
        }
        BindingType::Texture2D(texel_type) => {
            let texel_type = texel_type_tokens(*texel_type);

            quote!(#mod_path::BindingType::Texture2D(#texel_type))
        }
        BindingType::Texture3D(texel_type) => {
            let texel_type = texel_type_tokens(*texel_type);

            quote!(#mod_path::BindingType::Texture3D(#texel_type))
        }
        BindingType::Texture2DArray(texel_type) => {
            let texel_type = texel_type_tokens(*texel_type);

            quote!(#mod_path::BindingType::Texture2DArray(#texel_type))
        }
        BindingType::TextureCube(texel_type) => {
            let texel_type = texel_type_tokens(*texel_type);

            quote!(#mod_path::BindingType::TextureCube(#texel_type))
        }
        BindingType::TextureCubeArray(texel_type) => {
            let texel_type = texel_type_tokens(*texel_type);

            quote!(#mod_path::BindingType::TextureCubeArray(#texel_type))
        }
        BindingType::TextureMultisampled2D(texel_type) => {
            let texel_type = texel_type_tokens(*texel_type);

            quote!(#mod_path::BindingType::TextureMultisampled2D(#texel_type))
        }
        BindingType::TextureDepth2D => {
            quote!(#mod_path::BindingType::TextureDepth2D)
        }
        BindingType::TextureDepth2DArray => {
            quote!(#mod_path::BindingType::TextureDepth2DArray)
        }
        BindingType::TextureDepthCube => {
            quote!(#mod_path::BindingType::TextureDepthCube)
        }
        BindingType::TextureDepthCubeArray => {
            quote!(#mod_path::BindingType::TextureDepthCubeArray)
        }
        BindingType::TextureDepthMultisampled2D => {
            quote!(#mod_path::BindingType::TextureDepthMultisampled2D)
        }
        BindingType::StorageTexture1D(storage_format) => {
            let storage_format = storage_format_tokens(*storage_format);

            quote!(#mod_path::BindingType::StorageTexture1D(#storage_format))
        }
        BindingType::StorageTexture2D(storage_format) => {
            let storage_format = storage_format_tokens(*storage_format);

            quote!(#mod_path::BindingType::StorageTexture2D(#storage_format))
        }
        BindingType::StorageTexture2DArray(storage_format) => {
            let storage_format = storage_format_tokens(*storage_format);

            quote!(#mod_path::BindingType::StorageTexture2DArray(#storage_format))
        }
        BindingType::StorageTexture3D(storage_format) => {
            let storage_format = storage_format_tokens(*storage_format);

            quote!(#mod_path::BindingType::StorageTexture3D(#storage_format))
        }
        BindingType::FilteringSampler => {
            quote!(#mod_path::BindingType::FilteringSampler)
        }
        BindingType::NonFilteringSampler => {
            quote!(#mod_path::BindingType::NonFilteringSampler)
        }
        BindingType::ComparisonSampler => {
            quote!(#mod_path::BindingType::ComparisonSampler)
        }
        BindingType::Uniform(layout) => {
            let layout = sized_buffer_layout_tokens(layout);

            quote!(#mod_path::BindingType::Uniform(#layout))
        }
        BindingType::Storage(layout) => {
            let layout = unsized_buffer_layout_tokens(layout);

            quote!(#mod_path::BindingType::Storage(#layout))
        }
        BindingType::ReadOnlyStorage(layout) => {
            let layout = unsized_buffer_layout_tokens(layout);

            quote!(#mod_path::BindingType::ReadOnlyStorage(#layout))
        }
    }
}

fn texel_type_tokens(texel_type: TexelType) -> proc_macro2::TokenStream {
    let mod_path = quote!(empa::resource_binding);

    match texel_type {
        TexelType::Float => {
            quote!(#mod_path::TexelType::Float)
        }
        TexelType::UnfilterableFloat => {
            quote!(#mod_path::TexelType::UnfilterableFloat)
        }
        TexelType::Integer => {
            quote!(#mod_path::TexelType::Integer)
        }
        TexelType::UnsignedInteger => {
            quote!(#mod_path::TexelType::UnsignedInteger)
        }
    }
}

fn storage_format_tokens(storage_format: StorageTextureFormat) -> proc_macro2::TokenStream {
    let mod_path = quote!(empa::texture::format);

    match storage_format {
        StorageTextureFormat::rgba8unorm => {
            quote!(<#mod_path::rgba8unorm as #mod_path::TextureFormat>::FORMAT_ID)
        }
        StorageTextureFormat::rgba8snorm => {
            quote!(<#mod_path::rgba8snorm as #mod_path::TextureFormat>::FORMAT_ID)
        }
        StorageTextureFormat::rgba8uint => {
            quote!(<#mod_path::rgba8uint as #mod_path::TextureFormat>::FORMAT_ID)
        }
        StorageTextureFormat::rgba8sint => {
            quote!(<#mod_path::rgba8sint as #mod_path::TextureFormat>::FORMAT_ID)
        }
        StorageTextureFormat::rgba16uint => {
            quote!(<#mod_path::rgba16uint as #mod_path::TextureFormat>::FORMAT_ID)
        }
        StorageTextureFormat::rgba16sint => {
            quote!(<#mod_path::rgba16sint as #mod_path::TextureFormat>::FORMAT_ID)
        }
        StorageTextureFormat::rgba16float => {
            quote!(<#mod_path::rgba16float as #mod_path::TextureFormat>::FORMAT_ID)
        }
        StorageTextureFormat::r32uint => {
            quote!(<#mod_path::r32uint as #mod_path::TextureFormat>::FORMAT_ID)
        }
        StorageTextureFormat::r32sint => {
            quote!(<#mod_path::r32sint as #mod_path::TextureFormat>::FORMAT_ID)
        }
        StorageTextureFormat::r32float => {
            quote!(<#mod_path::r32float as #mod_path::TextureFormat>::FORMAT_ID)
        }
        StorageTextureFormat::rg32uint => {
            quote!(<#mod_path::rg32uint as #mod_path::TextureFormat>::FORMAT_ID)
        }
        StorageTextureFormat::rg32sint => {
            quote!(<#mod_path::rg32sint as #mod_path::TextureFormat>::FORMAT_ID)
        }
        StorageTextureFormat::rg32float => {
            quote!(<#mod_path::rg32float as #mod_path::TextureFormat>::FORMAT_ID)
        }
        StorageTextureFormat::rgba32uint => {
            quote!(<#mod_path::rgba32uint as #mod_path::TextureFormat>::FORMAT_ID)
        }
        StorageTextureFormat::rgba32sint => {
            quote!(<#mod_path::rgba32sint as #mod_path::TextureFormat>::FORMAT_ID)
        }
        StorageTextureFormat::rgba32float => {
            quote!(<#mod_path::rgba32float as #mod_path::TextureFormat>::FORMAT_ID)
        }
    }
}

fn sized_buffer_layout_tokens(layout: &SizedBufferLayout) -> proc_macro2::TokenStream {
    let recurse = layout.memory_units().iter().map(|u| memory_unit_tokens(u));

    quote! {
        empa::resource_binding::SizedBufferLayout(&[#(#recurse),*])
    }
}

fn unsized_buffer_layout_tokens(layout: &UnsizedBufferLayout) -> proc_macro2::TokenStream {
    let head_recurse = layout.sized_head().iter().map(|u| memory_unit_tokens(u));

    let tail = if let Some(layout) = layout.unsized_tail() {
        let recurse = layout.iter().map(|u| memory_unit_tokens(u));

        quote! {
            Some(&[#(#recurse),*])
        }
    } else {
        quote!(None)
    };

    quote! {
        empa::resource_binding::SizedBufferLayout(&[#(#head_recurse),*], #tail)
    }
}

fn memory_unit_tokens(memory_unit: &MemoryUnit) -> proc_macro2::TokenStream {
    let offset = memory_unit.offset;
    let layout = memory_unit_layout_tokens(&memory_unit.layout);

    quote! {
        empa::abi::MemoryUnit {
            offset: #offset,
            layout: #layout
        }
    }
}

fn memory_unit_layout_tokens(memory_unit_layout: &MemoryUnitLayout) -> proc_macro2::TokenStream {
    let mod_path = quote!(empa::abi);

    match *memory_unit_layout {
        MemoryUnitLayout::Float => {
            quote!(#mod_path::MemoryUnitLayout::Float)
        }
        MemoryUnitLayout::FloatArray(len) => {
            quote!(#mod_path::MemoryUnitLayout::FloatArray(#len))
        }
        MemoryUnitLayout::FloatVector2 => {
            quote!(#mod_path::MemoryUnitLayout::FloatVector2)
        }
        MemoryUnitLayout::FloatVector2Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::FloatVector2Array(#len))
        }
        MemoryUnitLayout::FloatVector3 => {
            quote!(#mod_path::MemoryUnitLayout::FloatVector3)
        }
        MemoryUnitLayout::FloatVector3Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::FloatVector3Array(#len))
        }
        MemoryUnitLayout::FloatVector4 => {
            quote!(#mod_path::MemoryUnitLayout::FloatVector4)
        }
        MemoryUnitLayout::FloatVector4Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::FloatVector4Array(#len))
        }
        MemoryUnitLayout::Integer => {
            quote!(#mod_path::MemoryUnitLayout::Integer)
        }
        MemoryUnitLayout::IntegerArray(len) => {
            quote!(#mod_path::MemoryUnitLayout::IntegerArray(#len))
        }
        MemoryUnitLayout::IntegerVector2 => {
            quote!(#mod_path::MemoryUnitLayout::IntegerVector2)
        }
        MemoryUnitLayout::IntegerVector2Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::IntegerVector2Array(#len))
        }
        MemoryUnitLayout::IntegerVector3 => {
            quote!(#mod_path::MemoryUnitLayout::IntegerVector3)
        }
        MemoryUnitLayout::IntegerVector3Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::IntegerVector3Array(#len))
        }
        MemoryUnitLayout::IntegerVector4 => {
            quote!(#mod_path::MemoryUnitLayout::IntegerVector4)
        }
        MemoryUnitLayout::IntegerVector4Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::IntegerVector4Array(#len))
        }
        MemoryUnitLayout::UnsignedInteger => {
            quote!(#mod_path::MemoryUnitLayout::UnsignedInteger)
        }
        MemoryUnitLayout::UnsignedIntegerArray(len) => {
            quote!(#mod_path::MemoryUnitLayout::UnsignedIntegerArray(#len))
        }
        MemoryUnitLayout::UnsignedIntegerVector2 => {
            quote!(#mod_path::MemoryUnitLayout::UnsignedIntegerVector2)
        }
        MemoryUnitLayout::UnsignedIntegerVector2Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::UnsignedIntegerVector2Array(#len))
        }
        MemoryUnitLayout::UnsignedIntegerVector3 => {
            quote!(#mod_path::MemoryUnitLayout::UnsignedIntegerVector3)
        }
        MemoryUnitLayout::UnsignedIntegerVector3Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::UnsignedIntegerVector3Array(#len))
        }
        MemoryUnitLayout::UnsignedIntegerVector4 => {
            quote!(#mod_path::MemoryUnitLayout::UnsignedIntegerVector4)
        }
        MemoryUnitLayout::UnsignedIntegerVector4Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::UnsignedIntegerVector4Array(#len))
        }
        MemoryUnitLayout::Matrix2x2 => {
            quote!(#mod_path::MemoryUnitLayout::Matrix2x2)
        }
        MemoryUnitLayout::Matrix2x2Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::Matrix2x2Array(#len))
        }
        MemoryUnitLayout::Matrix2x3 => {
            quote!(#mod_path::MemoryUnitLayout::Matrix2x3)
        }
        MemoryUnitLayout::Matrix2x3Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::Matrix2x3Array(#len))
        }
        MemoryUnitLayout::Matrix2x4 => {
            quote!(#mod_path::MemoryUnitLayout::Matrix2x4)
        }
        MemoryUnitLayout::Matrix2x4Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::Matrix2x4Array(#len))
        }
        MemoryUnitLayout::Matrix3x2 => {
            quote!(#mod_path::MemoryUnitLayout::Matrix3x2)
        }
        MemoryUnitLayout::Matrix3x2Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::Matrix3x2Array(#len))
        }
        MemoryUnitLayout::Matrix3x3 => {
            quote!(#mod_path::MemoryUnitLayout::Matrix3x3)
        }
        MemoryUnitLayout::Matrix3x3Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::Matrix3x3Array(#len))
        }
        MemoryUnitLayout::Matrix3x4 => {
            quote!(#mod_path::MemoryUnitLayout::Matrix3x4)
        }
        MemoryUnitLayout::Matrix3x4Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::Matrix3x4Array(#len))
        }
        MemoryUnitLayout::Matrix4x2 => {
            quote!(#mod_path::MemoryUnitLayout::Matrix4x2)
        }
        MemoryUnitLayout::Matrix4x2Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::Matrix4x2Array(#len))
        }
        MemoryUnitLayout::Matrix4x3 => {
            quote!(#mod_path::MemoryUnitLayout::Matrix4x3)
        }
        MemoryUnitLayout::Matrix4x3Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::Matrix4x3Array(#len))
        }
        MemoryUnitLayout::Matrix4x4 => {
            quote!(#mod_path::MemoryUnitLayout::Matrix4x4)
        }
        MemoryUnitLayout::Matrix4x4Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::Matrix4x4Array(#len))
        }
    }
}

fn constant_type_tokens(constant_type: ConstantType) -> proc_macro2::TokenStream {
    let mod_path = quote!(empa::shader_module);

    match constant_type {
        ConstantType::Float => {
            quote!(#mod_path::StaticConstantType::Float)
        }
        ConstantType::Bool => {
            quote!(#mod_path::StaticConstantType::Bool)
        }
        ConstantType::SignedInteger => {
            quote!(#mod_path::StaticConstantType::SignedInteger)
        }
        ConstantType::UnsignedInteger => {
            quote!(#mod_path::StaticConstantType::UnsignedInteger)
        }
    }
}

fn shader_stage_tokens(shader_stage: ShaderStage) -> proc_macro2::TokenStream {
    let mod_path = quote!(empa::shader_module);

    match shader_stage {
        ShaderStage::Vertex => {
            quote!(#mod_path::StaticShaderStage::Vertex)
        }
        ShaderStage::Fragment => {
            quote!(#mod_path::StaticShaderStage::Fragment)
        }
        ShaderStage::Compute => {
            quote!(#mod_path::StaticShaderStage::Compute)
        }
    }
}

fn entry_point_binding_tokens(entry_point_binding: &EntryPointBinding) -> proc_macro2::TokenStream {
    let mod_path = quote!(empa::shader_module);

    let location = entry_point_binding.location();
    let binding_type = entry_point_binding_type_tokens(entry_point_binding.binding_type());

    let interpolation = if let Some(interpolation) = entry_point_binding.interpolation() {
        let interpolation = interpolation_tokens(interpolation);

        quote!(Some(#interpolation))
    } else {
        quote!(None)
    };

    let sampling = if let Some(sampling) = entry_point_binding.sampling() {
        let sampling = sampling_tokens(sampling);

        quote!(Some(#sampling))
    } else {
        quote!(None)
    };

    quote! {
        #mod_path::StaticEntryPointBinding {
            location: #location,
            binding_type: #binding_type,
            interpolation: #interpolation,
            sampling: #sampling,
        }
    }
}

fn entry_point_binding_type_tokens(
    binding_type: EntryPointBindingType,
) -> proc_macro2::TokenStream {
    let mod_path = quote!(empa::shader_module);

    match binding_type {
        EntryPointBindingType::SignedInteger => {
            quote!(#mod_path::StaticEntryPointBindingType::SignedInteger)
        }
        EntryPointBindingType::SignedIntegerVector2 => {
            quote!(#mod_path::StaticEntryPointBindingType::SignedIntegerVector2)
        }
        EntryPointBindingType::SignedIntegerVector3 => {
            quote!(#mod_path::StaticEntryPointBindingType::SignedIntegerVector3)
        }
        EntryPointBindingType::SignedIntegerVector4 => {
            quote!(#mod_path::StaticEntryPointBindingType::SignedIntegerVector4)
        }
        EntryPointBindingType::UnsignedInteger => {
            quote!(#mod_path::StaticEntryPointBindingType::UnsignedInteger)
        }
        EntryPointBindingType::UnsignedIntegerVector2 => {
            quote!(#mod_path::StaticEntryPointBindingType::UnsignedIntegerVector2)
        }
        EntryPointBindingType::UnsignedIntegerVector3 => {
            quote!(#mod_path::StaticEntryPointBindingType::UnsignedIntegerVector3)
        }
        EntryPointBindingType::UnsignedIntegerVector4 => {
            quote!(#mod_path::StaticEntryPointBindingType::UnsignedIntegerVector4)
        }
        EntryPointBindingType::Float => {
            quote!(#mod_path::StaticEntryPointBindingType::Float)
        }
        EntryPointBindingType::FloatVector2 => {
            quote!(#mod_path::StaticEntryPointBindingType::FloatVector2)
        }
        EntryPointBindingType::FloatVector3 => {
            quote!(#mod_path::StaticEntryPointBindingType::FloatVector3)
        }
        EntryPointBindingType::FloatVector4 => {
            quote!(#mod_path::StaticEntryPointBindingType::FloatVector4)
        }
        EntryPointBindingType::HalfFloat => {
            quote!(#mod_path::StaticEntryPointBindingType::HalfFloat)
        }
        EntryPointBindingType::HalfFloatVector2 => {
            quote!(#mod_path::StaticEntryPointBindingType::HalfFloatVector2)
        }
        EntryPointBindingType::HalfFloatVector3 => {
            quote!(#mod_path::StaticEntryPointBindingType::HalfFloatVector3)
        }
        EntryPointBindingType::HalfFloatVector4 => {
            quote!(#mod_path::StaticEntryPointBindingType::HalfFloatVector4)
        }
    }
}

fn interpolation_tokens(interpolation: Interpolation) -> proc_macro2::TokenStream {
    let mod_path = quote!(empa::shader_module);

    match interpolation {
        Interpolation::Perspective => {
            quote!(#mod_path::StaticInterpolation::Perspective)
        }
        Interpolation::Linear => {
            quote!(#mod_path::StaticInterpolation::Linear)
        }
        Interpolation::Flat => {
            quote!(#mod_path::StaticInterpolation::Flat)
        }
    }
}

fn sampling_tokens(sampling: Sampling) -> proc_macro2::TokenStream {
    let mod_path = quote!(empa::shader_module);

    match sampling {
        Sampling::Center => {
            quote!(#mod_path::StaticSampling::Center)
        }
        Sampling::Centroid => {
            quote!(#mod_path::StaticSampling::Centroid)
        }
        Sampling::Sample => {
            quote!(#mod_path::StaticSampling::Sample)
        }
    }
}
