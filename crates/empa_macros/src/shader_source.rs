use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::env;
use std::error::Error as _;
use std::hash::{Hash, Hasher};
use std::ops::Range;
use std::path::Path;

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::{Error, Files, SimpleFile};
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use empa_smi::wgsl::{BuildSmiError, build_smi};
use empa_smi::{
    ArrayLayout, EntryPoint, Interpolate, InterpolationType, IoBinding, IoBindingType, MemoryUnit,
    MemoryUnitLayout, OverridableConstant, OverridableConstantType, ResourceBinding, ResourceType,
    Sampling, ShaderModuleInterface, ShaderStage, SizedBufferLayout, StorageTextureFormat,
    TexelType, UnsizedBufferLayout, UnsizedTailLayout,
};
use include_preprocessor::{
    Error as IppError, OutputSink, SearchPaths, SourceMappedChunk, SourceTracker, preprocess,
};
use proc_macro::{Span, TokenStream, tracked_path};
use quote::{quote, quote_spanned};
use syn::{LitStr, parse_macro_input};

fn gen_file_id(path: &Path) -> u64 {
    let mut hasher = DefaultHasher::new();

    path.hash(&mut hasher);

    hasher.finish()
}

struct SourceFiles {
    map: HashMap<u64, SimpleFile<String, String>>,
}

impl SourceFiles {
    fn new() -> Self {
        SourceFiles {
            map: Default::default(),
        }
    }
}

impl SourceTracker for SourceFiles {
    fn track(&mut self, path: &Path, source: &str) {
        let id = gen_file_id(path);
        let path = path
            .to_str()
            .expect("cannot track non-unicode path")
            .to_string();
        let source = source.to_string();

        tracked_path::path(&path);
        self.map.insert(id, SimpleFile::new(path, source));
    }
}

impl<'a> Files<'a> for SourceFiles {
    type FileId = u64;
    type Name = &'a str;
    type Source = &'a str;

    fn name(&'a self, id: Self::FileId) -> Result<Self::Name, Error> {
        self.map
            .get(&id)
            .ok_or(Error::FileMissing)
            .map(|file| file.name().as_str())
    }

    fn source(&'a self, id: Self::FileId) -> Result<Self::Source, Error> {
        self.map
            .get(&id)
            .ok_or(Error::FileMissing)
            .map(|file| file.source().as_str())
    }

    fn line_index(&'a self, id: Self::FileId, byte_index: usize) -> Result<usize, Error> {
        self.map
            .get(&id)
            .ok_or(Error::FileMissing)
            .and_then(|file| file.line_index((), byte_index))
    }

    fn line_range(&'a self, id: Self::FileId, line_index: usize) -> Result<Range<usize>, Error> {
        self.map
            .get(&id)
            .ok_or(Error::FileMissing)
            .and_then(|file| file.line_range((), line_index))
    }
}

struct SourceSpan {
    source_range: Range<usize>,
    file_id: u64,
    mapped_range: Range<usize>,
}

struct SourceMappedSpan {
    file_id: u64,
    range: Range<usize>,
}

struct SourceMap {
    spans: Vec<SourceSpan>,
}

impl SourceMap {
    fn new() -> Self {
        SourceMap { spans: Vec::new() }
    }

    fn mapped_span(&self, source_range: Range<usize>) -> Option<SourceMappedSpan> {
        let start = source_range.start;

        for span in &self.spans {
            if span.source_range.contains(&start) {
                let span_size = usize::min(source_range.len(), span.source_range.end - start);
                let offset = source_range.start - span.source_range.start;
                let start = span.mapped_range.start + offset;
                let end = start + span_size;

                return Some(SourceMappedSpan {
                    file_id: span.file_id,
                    range: start..end,
                });
            }
        }

        None
    }
}

struct OutputWriter {
    buffer: String,
    source_map: SourceMap,
    current_byte_offset: usize,
}

impl OutputWriter {
    fn new() -> Self {
        OutputWriter {
            buffer: String::new(),
            source_map: SourceMap::new(),
            current_byte_offset: 0,
        }
    }
}

impl OutputSink for OutputWriter {
    fn sink(&mut self, chunk: &str) {
        self.current_byte_offset += chunk.len();
        self.buffer.push_str(chunk);
    }

    fn sink_source_mapped(&mut self, source_mapped_chunk: SourceMappedChunk) {
        let start = self.current_byte_offset;

        self.current_byte_offset += source_mapped_chunk.text().len();
        self.buffer.push_str(source_mapped_chunk.text());
        self.source_map.spans.push(SourceSpan {
            source_range: start..self.current_byte_offset,
            file_id: gen_file_id(source_mapped_chunk.source_path()),
            mapped_range: source_mapped_chunk.source_range(),
        });
    }
}

pub fn expand_shader_source(input: TokenStream) -> TokenStream {
    let path = parse_macro_input!(input as LitStr);

    let span = Span::call_site();
    let source_path = span.local_file().unwrap();
    let source_dir = source_path.parent().unwrap();

    let mut search_paths = SearchPaths::new();
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    search_paths.push_base_path(cargo_manifest_dir);

    let source_join = source_dir.join(path.value());
    let mut source_files = SourceFiles::new();

    let output = if source_join.is_file() {
        let writer = OutputWriter::new();

        match preprocess(&source_join, search_paths, writer, &mut source_files) {
            Ok(output) => output,
            Err(error) => {
                let (file, diagnostic) = match error {
                    IppError::FileNotFound(error) => {
                        let file = SimpleFile::new(
                            error.source_file().to_string_lossy().to_string(),
                            error.source().to_string(),
                        );
                        let range = file.line_range((), error.line_number()).unwrap();

                        // I don't quite understand if this is a bug in `codespan_reporting` or
                        // if I'm doing something wrong that necessitates this correction
                        let range = range.start..range.end.saturating_sub(1);

                        let label = Label::primary((), range);
                        let diagnostic = Diagnostic::error()
                            .with_message(format!(
                                "Could not find file: {}",
                                error.included_path().to_string_lossy()
                            ))
                            .with_labels(vec![label]);

                        (file, diagnostic)
                    }
                    IppError::IO(error) => {
                        panic!("{}", error);
                    }
                    IppError::Parse(error) => {
                        let file = SimpleFile::new(
                            error.source_file().to_string_lossy().to_string(),
                            error.source().to_string(),
                        );
                        let range = file.line_range((), error.line_number()).unwrap();

                        // I don't quite understand if this is a bug in `codespan_reporting` or
                        // if I'm doing something wrong that necessitates this correction
                        let range = range.start..range.end.saturating_sub(1);

                        let label = Label::primary((), range);
                        let diagnostic = Diagnostic::error()
                            .with_message(error.message().to_string())
                            .with_labels(vec![label]);

                        (file, diagnostic)
                    }
                };

                let config = term::Config::default();
                let writer = StandardStream::stderr(ColorChoice::Auto);

                term::emit(&mut writer.lock(), &config, &file, &diagnostic)
                    .expect("cannot write error");

                return quote! {
                    compile_error!("failed to preprocess shader module; see errors reported above");
                }
                .into();
            }
        }
    } else {
        let span = path.span();

        return quote_spanned! {span=>
            compile_error!("the given path does not resolve to a valid file");
        }
        .into();
    };

    let source_token = LitStr::new(&output.buffer, Span::call_site().into());

    let smi = match build_smi(&output.buffer) {
        Ok(smi) => shader_module_interface_to_tokens(&smi),
        Err(err) => {
            match err {
                BuildSmiError::Parse(err) => {
                    let diagnostic = Diagnostic::error()
                        .with_message(err.message().to_string())
                        .with_labels(
                            err.labels()
                                .flat_map(|label| {
                                    let source_range = label.0.clone().to_range()?;
                                    let mapped_span =
                                        output.source_map.mapped_span(source_range).unwrap();

                                    Some(
                                        Label::primary(
                                            mapped_span.file_id,
                                            mapped_span.range.clone(),
                                        )
                                        .with_message(label.1.to_string()),
                                    )
                                })
                                .collect(),
                        );

                    let config = codespan_reporting::term::Config::default();
                    let writer = StandardStream::stderr(ColorChoice::Auto);

                    term::emit(&mut writer.lock(), &config, &source_files, &diagnostic)
                        .expect("cannot write error");
                }
                BuildSmiError::Validation(err) => {
                    let mut diagnostic =
                        Diagnostic::error().with_message(err.as_inner().to_string());

                    if let Some(location) = err.location(&output.buffer) {
                        let start = location.offset as usize;
                        let end = start + location.length as usize;

                        let mapped_span = output.source_map.mapped_span(start..end).unwrap();

                        let mut label =
                            Label::primary(mapped_span.file_id, mapped_span.range.clone());

                        if let Some(source) = err.source() {
                            label = label.with_message(source.to_string())
                        }

                        diagnostic = diagnostic.with_labels(vec![label])
                    }

                    let config = codespan_reporting::term::Config::default();
                    let writer = StandardStream::stderr(ColorChoice::Auto);

                    term::emit(&mut writer.lock(), &config, &source_files, &diagnostic)
                        .expect("cannot write error");
                }
            }

            return quote! {
                compile_error!("invalid shader module; see errors reported above");
            }
            .into();
        }
    };

    let result = quote! {
        empa::shader_module::ShaderSource::from_static_unchecked(#source_token, &const {#smi})
    };

    result.into()
}

fn shader_module_interface_to_tokens(smi: &ShaderModuleInterface) -> proc_macro2::TokenStream {
    let resource_bindings = smi.resource_bindings.iter().map(resource_binding_to_tokens);
    let overridable_constants = smi
        .overridable_constants
        .iter()
        .map(overridable_constant_to_tokens);
    let entry_points = smi.entry_points.iter().map(entry_point_to_tokens);

    quote! {
        empa::smi::ShaderModuleInterface {
            resource_bindings: std::borrow::Cow::Borrowed(&[#(#resource_bindings),*]),
            overridable_constants: std::borrow::Cow::Borrowed(&[#(#overridable_constants),*]),
            entry_points: std::borrow::Cow::Borrowed(&[#(#entry_points),*]),
        }
    }
}

fn resource_binding_to_tokens(resource_binding: &ResourceBinding) -> proc_macro2::TokenStream {
    let group = resource_binding.group;
    let binding = resource_binding.binding;
    let resource_type = resource_type_to_tokens(&resource_binding.resource_type);

    quote! {
        empa::smi::ResourceBinding {
            group: #group,
            binding: #binding,
            resource_type: #resource_type
        }
    }
}

fn overridable_constant_to_tokens(
    overridable_constant: &OverridableConstant,
) -> proc_macro2::TokenStream {
    let id = if let Some(id) = overridable_constant.id {
        quote!(Some(#id))
    } else {
        quote!(None)
    };
    let name = overridable_constant.name.as_ref();
    let constant_type = constant_type_to_tokens(overridable_constant.constant_type);
    let required = overridable_constant.required;

    quote! {
        empa::smi::OverridableConstant {
            id: #id,
            name: std::borrow::Cow::Borrowed(#name),
            constant_type: #constant_type,
            required: #required,
        }
    }
}

fn entry_point_to_tokens(entry_point: &EntryPoint) -> proc_macro2::TokenStream {
    let name = entry_point.name.as_ref();
    let stage = shader_stage_to_tokens(entry_point.stage);
    let input_bindings = entry_point.input_bindings.iter().map(io_binding_to_tokens);
    let output_bindings = entry_point.output_bindings.iter().map(io_binding_to_tokens);
    let overridable_constants = entry_point.overridable_constants.iter();
    let resource_bindings = entry_point.resource_bindings.iter();

    quote! {
        empa::smi::EntryPoint {
            name: std::borrow::Cow::Borrowed(#name),
            stage: #stage,
            input_bindings: std::borrow::Cow::Borrowed(&[#(#input_bindings),*]),
            output_bindings: std::borrow::Cow::Borrowed(&[#(#output_bindings),*]),
            overridable_constants: std::borrow::Cow::Borrowed(&[#(#overridable_constants),*]),
            resource_bindings: std::borrow::Cow::Borrowed(&[#(#resource_bindings),*]),
        }
    }
}

fn resource_type_to_tokens(resource_type: &ResourceType) -> proc_macro2::TokenStream {
    match resource_type {
        ResourceType::Texture1D(texel_type) => {
            let texel_type = texel_type_to_tokens(*texel_type);

            quote!(empa::smi::ResourceType::Texture1D(#texel_type))
        }
        ResourceType::Texture2D(texel_type) => {
            let texel_type = texel_type_to_tokens(*texel_type);

            quote!(empa::smi::ResourceType::Texture2D(#texel_type))
        }
        ResourceType::Texture3D(texel_type) => {
            let texel_type = texel_type_to_tokens(*texel_type);

            quote!(empa::smi::ResourceType::Texture3D(#texel_type))
        }
        ResourceType::Texture2DArray(texel_type) => {
            let texel_type = texel_type_to_tokens(*texel_type);

            quote!(empa::smi::ResourceType::Texture2DArray(#texel_type))
        }
        ResourceType::TextureCube(texel_type) => {
            let texel_type = texel_type_to_tokens(*texel_type);

            quote!(empa::smi::ResourceType::TextureCube(#texel_type))
        }
        ResourceType::TextureCubeArray(texel_type) => {
            let texel_type = texel_type_to_tokens(*texel_type);

            quote!(empa::smi::ResourceType::TextureCubeArray(#texel_type))
        }
        ResourceType::TextureMultisampled2D(texel_type) => {
            let texel_type = texel_type_to_tokens(*texel_type);

            quote!(empa::smi::ResourceType::TextureMultisampled2D(#texel_type))
        }
        ResourceType::TextureDepth2D => {
            quote!(empa::smi::ResourceType::TextureDepth2D)
        }
        ResourceType::TextureDepth2DArray => {
            quote!(empa::smi::ResourceType::TextureDepth2DArray)
        }
        ResourceType::TextureDepthCube => {
            quote!(empa::smi::ResourceType::TextureDepthCube)
        }
        ResourceType::TextureDepthCubeArray => {
            quote!(empa::smi::ResourceType::TextureDepthCubeArray)
        }
        ResourceType::TextureDepthMultisampled2D => {
            quote!(empa::smi::ResourceType::TextureDepthMultisampled2D)
        }
        ResourceType::StorageTexture1D(storage_format) => {
            let storage_format = storage_texture_format_to_tokens(*storage_format);

            quote!(empa::smi::ResourceType::StorageTexture1D(#storage_format))
        }
        ResourceType::StorageTexture2D(storage_format) => {
            let storage_format = storage_texture_format_to_tokens(*storage_format);

            quote!(empa::smi::ResourceType::StorageTexture2D(#storage_format))
        }
        ResourceType::StorageTexture2DArray(storage_format) => {
            let storage_format = storage_texture_format_to_tokens(*storage_format);

            quote!(empa::smi::ResourceType::StorageTexture2DArray(#storage_format))
        }
        ResourceType::StorageTexture3D(storage_format) => {
            let storage_format = storage_texture_format_to_tokens(*storage_format);

            quote!(empa::smi::ResourceType::StorageTexture3D(#storage_format))
        }
        ResourceType::FilteringSampler => {
            quote!(empa::smi::ResourceType::FilteringSampler)
        }
        ResourceType::NonFilteringSampler => {
            quote!(empa::smi::ResourceType::NonFilteringSampler)
        }
        ResourceType::ComparisonSampler => {
            quote!(empa::smi::ResourceType::ComparisonSampler)
        }
        ResourceType::Uniform(layout) => {
            let layout = sized_buffer_layout_to_tokens(layout);

            quote!(empa::smi::ResourceType::Uniform(#layout))
        }
        ResourceType::StorageRead(layout) => {
            let layout = unsized_buffer_layout_to_tokens(layout);

            quote!(empa::smi::ResourceType::StorageRead(#layout))
        }
        ResourceType::StorageReadWrite(layout) => {
            let layout = unsized_buffer_layout_to_tokens(layout);

            quote!(empa::smi::ResourceType::StorageReadWrite(#layout))
        }
    }
}

fn texel_type_to_tokens(texel_type: TexelType) -> proc_macro2::TokenStream {
    match texel_type {
        TexelType::Float => {
            quote!(empa::smi::TexelType::Float)
        }
        TexelType::UnfilterableFloat => {
            quote!(empa::smi::TexelType::UnfilterableFloat)
        }
        TexelType::Integer => {
            quote!(empa::smi::TexelType::Integer)
        }
        TexelType::UnsignedInteger => {
            quote!(empa::smi::TexelType::UnsignedInteger)
        }
    }
}

fn storage_texture_format_to_tokens(
    storage_format: StorageTextureFormat,
) -> proc_macro2::TokenStream {
    match storage_format {
        StorageTextureFormat::rgba8unorm => {
            quote!(empa::smi::StorageTextureFormat::rgba8unorm)
        }
        StorageTextureFormat::rgba8snorm => {
            quote!(empa::smi::StorageTextureFormat::rgba8snorm)
        }
        StorageTextureFormat::rgba8uint => {
            quote!(empa::smi::StorageTextureFormat::rgba8uint)
        }
        StorageTextureFormat::rgba8sint => {
            quote!(empa::smi::StorageTextureFormat::rgba8sint)
        }
        StorageTextureFormat::rgba16uint => {
            quote!(empa::smi::StorageTextureFormat::rgba16uint)
        }
        StorageTextureFormat::rgba16sint => {
            quote!(empa::smi::StorageTextureFormat::rgba16sint)
        }
        StorageTextureFormat::rgba16float => {
            quote!(empa::smi::StorageTextureFormat::rgba16float)
        }
        StorageTextureFormat::r32uint => {
            quote!(empa::smi::StorageTextureFormat::r32uint)
        }
        StorageTextureFormat::r32sint => {
            quote!(empa::smi::StorageTextureFormat::r32sint)
        }
        StorageTextureFormat::r32float => {
            quote!(empa::smi::StorageTextureFormat::r32float)
        }
        StorageTextureFormat::rg32uint => {
            quote!(empa::smi::StorageTextureFormat::rg32uint)
        }
        StorageTextureFormat::rg32sint => {
            quote!(empa::smi::StorageTextureFormat::rg32sint)
        }
        StorageTextureFormat::rg32float => {
            quote!(empa::smi::StorageTextureFormat::rg32float)
        }
        StorageTextureFormat::rgba32uint => {
            quote!(empa::smi::StorageTextureFormat::rgba32uint)
        }
        StorageTextureFormat::rgba32sint => {
            quote!(empa::smi::StorageTextureFormat::rgba32sint)
        }
        StorageTextureFormat::rgba32float => {
            quote!(empa::smi::StorageTextureFormat::rgba32float)
        }
    }
}

fn sized_buffer_layout_to_tokens(layout: &SizedBufferLayout) -> proc_macro2::TokenStream {
    let memory_units = layout.memory_units.iter().map(memory_unit_to_tokens);

    quote! {
        empa::smi::SizedBufferLayout {
            memory_units: std::borrow::Cow::Borrowed(&[#(#memory_units),*]),
        }
    }
}

fn unsized_buffer_layout_to_tokens(layout: &UnsizedBufferLayout) -> proc_macro2::TokenStream {
    let sized_head = layout.sized_head.iter().map(memory_unit_to_tokens);
    let unsized_tail = if let Some(unsized_tail) = &layout.unsized_tail {
        let unsized_tail = unsized_tail_layout_to_tokens(unsized_tail);

        quote!(Some(#unsized_tail))
    } else {
        quote!(None)
    };

    quote! {
        empa::smi::UnsizedBufferLayout {
            sized_head: std::borrow::Cow::Borrowed(&[#(#sized_head),*]),
            unsized_tail: #unsized_tail,
        }
    }
}

fn unsized_tail_layout_to_tokens(layout: &UnsizedTailLayout) -> proc_macro2::TokenStream {
    let offset = layout.offset;
    let element_layout = layout.element_layout.iter().map(memory_unit_to_tokens);
    let stride = layout.stride;

    quote! {
        empa::smi::UnsizedTailLayout {
            offset: #offset,
            element_layout: std::borrow::Cow::Borrowed(&[#(#element_layout),*]),
            stride: #stride,
        }
    }
}

fn memory_unit_to_tokens(memory_unit: &MemoryUnit) -> proc_macro2::TokenStream {
    let offset = memory_unit.offset;
    let layout = memory_unit_layout_to_tokens(&memory_unit.layout);

    quote! {
        empa::smi::MemoryUnit {
            offset: #offset,
            layout: #layout
        }
    }
}

fn memory_unit_layout_to_tokens(memory_unit_layout: &MemoryUnitLayout) -> proc_macro2::TokenStream {
    match memory_unit_layout {
        MemoryUnitLayout::Float => {
            quote!(empa::smi::MemoryUnitLayout::Float)
        }
        MemoryUnitLayout::FloatVector2 => {
            quote!(empa::smi::MemoryUnitLayout::FloatVector2)
        }
        MemoryUnitLayout::FloatVector3 => {
            quote!(empa::smi::MemoryUnitLayout::FloatVector3)
        }
        MemoryUnitLayout::FloatVector4 => {
            quote!(empa::smi::MemoryUnitLayout::FloatVector4)
        }
        MemoryUnitLayout::Integer => {
            quote!(empa::smi::MemoryUnitLayout::Integer)
        }
        MemoryUnitLayout::IntegerVector2 => {
            quote!(empa::smi::MemoryUnitLayout::IntegerVector2)
        }
        MemoryUnitLayout::IntegerVector3 => {
            quote!(empa::smi::MemoryUnitLayout::IntegerVector3)
        }
        MemoryUnitLayout::IntegerVector4 => {
            quote!(empa::smi::MemoryUnitLayout::IntegerVector4)
        }
        MemoryUnitLayout::UnsignedInteger => {
            quote!(empa::smi::MemoryUnitLayout::UnsignedInteger)
        }
        MemoryUnitLayout::UnsignedIntegerVector2 => {
            quote!(empa::smi::MemoryUnitLayout::UnsignedIntegerVector2)
        }
        MemoryUnitLayout::UnsignedIntegerVector3 => {
            quote!(empa::smi::MemoryUnitLayout::UnsignedIntegerVector3)
        }
        MemoryUnitLayout::UnsignedIntegerVector4 => {
            quote!(empa::smi::MemoryUnitLayout::UnsignedIntegerVector4)
        }
        MemoryUnitLayout::Matrix2x2 => {
            quote!(empa::smi::MemoryUnitLayout::Matrix2x2)
        }
        MemoryUnitLayout::Matrix2x3 => {
            quote!(empa::smi::MemoryUnitLayout::Matrix2x3)
        }
        MemoryUnitLayout::Matrix2x4 => {
            quote!(empa::smi::MemoryUnitLayout::Matrix2x4)
        }
        MemoryUnitLayout::Matrix3x2 => {
            quote!(empa::smi::MemoryUnitLayout::Matrix3x2)
        }
        MemoryUnitLayout::Matrix3x3 => {
            quote!(empa::smi::MemoryUnitLayout::Matrix3x3)
        }
        MemoryUnitLayout::Matrix3x4 => {
            quote!(empa::smi::MemoryUnitLayout::Matrix3x4)
        }
        MemoryUnitLayout::Matrix4x2 => {
            quote!(empa::smi::MemoryUnitLayout::Matrix4x2)
        }
        MemoryUnitLayout::Matrix4x3 => {
            quote!(empa::smi::MemoryUnitLayout::Matrix4x3)
        }
        MemoryUnitLayout::Matrix4x4 => {
            quote!(empa::smi::MemoryUnitLayout::Matrix4x4)
        }
        MemoryUnitLayout::Array(array_layout) => {
            let array_layout = array_layout_to_tokens(array_layout);

            quote!(empa::smi::MemoryUnitLayout::Array(#array_layout))
        }
    }
}

fn array_layout_to_tokens(array_layout: &ArrayLayout) -> proc_macro2::TokenStream {
    let element_layout = array_layout
        .element_layout
        .iter()
        .map(memory_unit_to_tokens);
    let stride = array_layout.stride;
    let len = array_layout.len;

    quote! {
        empa::smi::ArrayLayout {
            element_layout: std::borrow::Cow::Borrowed(&[#(#element_layout),*]),
            stride: #stride,
            len: #len,
        }
    }
}

fn constant_type_to_tokens(constant_type: OverridableConstantType) -> proc_macro2::TokenStream {
    match constant_type {
        OverridableConstantType::Float => {
            quote!(empa::smi::OverridableConstantType::Float)
        }
        OverridableConstantType::Bool => {
            quote!(empa::smi::OverridableConstantType::Bool)
        }
        OverridableConstantType::SignedInteger => {
            quote!(empa::smi::OverridableConstantType::SignedInteger)
        }
        OverridableConstantType::UnsignedInteger => {
            quote!(empa::smi::OverridableConstantType::UnsignedInteger)
        }
    }
}

fn shader_stage_to_tokens(shader_stage: ShaderStage) -> proc_macro2::TokenStream {
    match shader_stage {
        ShaderStage::Vertex => {
            quote!(empa::smi::ShaderStage::Vertex)
        }
        ShaderStage::Fragment => {
            quote!(empa::smi::ShaderStage::Fragment)
        }
        ShaderStage::Compute => {
            quote!(empa::smi::ShaderStage::Compute)
        }
    }
}

fn io_binding_to_tokens(io_binding: &IoBinding) -> proc_macro2::TokenStream {
    let location = io_binding.location;
    let binding_type = io_binding_type_to_tokens(io_binding.binding_type);

    let interpolate = if let Some(interpolate) = &io_binding.interpolate {
        let interpolate = interpolate_to_tokens(interpolate);

        quote!(Some(#interpolate))
    } else {
        quote!(None)
    };

    quote! {
        empa::smi::IoBinding {
            location: #location,
            binding_type: #binding_type,
            interpolate: #interpolate,
        }
    }
}

fn io_binding_type_to_tokens(binding_type: IoBindingType) -> proc_macro2::TokenStream {
    match binding_type {
        IoBindingType::SignedInteger => {
            quote!(empa::smi::IoBindingType::SignedInteger)
        }
        IoBindingType::SignedIntegerVector2 => {
            quote!(empa::smi::IoBindingType::SignedIntegerVector2)
        }
        IoBindingType::SignedIntegerVector3 => {
            quote!(empa::smi::IoBindingType::SignedIntegerVector3)
        }
        IoBindingType::SignedIntegerVector4 => {
            quote!(empa::smi::IoBindingType::SignedIntegerVector4)
        }
        IoBindingType::UnsignedInteger => {
            quote!(empa::smi::IoBindingType::UnsignedInteger)
        }
        IoBindingType::UnsignedIntegerVector2 => {
            quote!(empa::smi::IoBindingType::UnsignedIntegerVector2)
        }
        IoBindingType::UnsignedIntegerVector3 => {
            quote!(empa::smi::IoBindingType::UnsignedIntegerVector3)
        }
        IoBindingType::UnsignedIntegerVector4 => {
            quote!(empa::smi::IoBindingType::UnsignedIntegerVector4)
        }
        IoBindingType::Float => {
            quote!(empa::smi::IoBindingType::Float)
        }
        IoBindingType::FloatVector2 => {
            quote!(empa::smi::IoBindingType::FloatVector2)
        }
        IoBindingType::FloatVector3 => {
            quote!(empa::smi::IoBindingType::FloatVector3)
        }
        IoBindingType::FloatVector4 => {
            quote!(empa::smi::IoBindingType::FloatVector4)
        }
        IoBindingType::HalfFloat => {
            quote!(empa::smi::IoBindingType::HalfFloat)
        }
        IoBindingType::HalfFloatVector2 => {
            quote!(empa::smi::IoBindingType::HalfFloatVector2)
        }
        IoBindingType::HalfFloatVector3 => {
            quote!(empa::smi::IoBindingType::HalfFloatVector3)
        }
        IoBindingType::HalfFloatVector4 => {
            quote!(empa::smi::IoBindingType::HalfFloatVector4)
        }
    }
}

fn interpolate_to_tokens(interpolate: &Interpolate) -> proc_macro2::TokenStream {
    let interpolation_type = interpolation_type_to_tokens(interpolate.interpolation_type);
    let sampling = if let Some(sampling) = interpolate.sampling {
        let sampling = sampling_to_tokens(sampling);

        quote!(Some(#sampling))
    } else {
        quote!(None)
    };

    quote! {
        empa::smi::Interpolate {
            interpolation_type: #interpolation_type,
            sampling: #sampling,
        }
    }
}

fn interpolation_type_to_tokens(interpolation: InterpolationType) -> proc_macro2::TokenStream {
    match interpolation {
        InterpolationType::Perspective => {
            quote!(empa::smi::InterpolationType::Perspective)
        }
        InterpolationType::Linear => {
            quote!(empa::smi::InterpolationType::Linear)
        }
        InterpolationType::Flat => {
            quote!(empa::smi::InterpolationType::Flat)
        }
    }
}

fn sampling_to_tokens(sampling: Sampling) -> proc_macro2::TokenStream {
    match sampling {
        Sampling::Center => {
            quote!(empa::smi::Sampling::Center)
        }
        Sampling::Centroid => {
            quote!(empa::smi::Sampling::Centroid)
        }
        Sampling::Sample => {
            quote!(empa::smi::Sampling::Sample)
        }
        Sampling::First => {
            quote!(empa::smi::Sampling::First)
        }
        Sampling::Either => {
            quote!(empa::smi::Sampling::Either)
        }
    }
}
