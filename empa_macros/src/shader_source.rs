use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::env;
use std::hash::{Hash, Hasher};
use std::ops::Range;
use std::path::Path;

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::{Error, Files, SimpleFile};
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use empa_reflect::{
    BindingType, ConstantIdentifier, ConstantType, EntryPointBinding, EntryPointBindingType,
    Interpolation, MemoryUnit, MemoryUnitLayout, Sampling, ShaderSource, ShaderStage,
    SizedBufferLayout, StorageTextureFormat, TexelType, UnsizedBufferLayout,
};
use include_preprocessor::{
    preprocess, Error as IppError, OutputSink, SearchPaths, SourceMappedChunk, SourceTracker,
};
use proc_macro::{tracked_path, Span, TokenStream};
use quote::quote;
use syn::{parse_macro_input, LitStr};

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
    let source_path = span.source_file().path();
    let source_dir = source_path.parent().unwrap();

    let mut search_paths = SearchPaths::new();
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    search_paths.push_base_path(cargo_manifest_dir);

    let source_join = source_dir.join(path.value());
    let mut source_files = SourceFiles::new();

    let output = if source_join.is_file() {
        let writer = OutputWriter::new();

        match preprocess(source_join, search_paths, writer, &mut source_files) {
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
                        panic!("adsf asdf {}", error);
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

                let config = codespan_reporting::term::Config::default();
                let writer = StandardStream::stderr(ColorChoice::Auto);

                term::emit(&mut writer.lock(), &config, &file, &diagnostic)
                    .expect("cannot write error");

                panic!("failed to compile shader source");
            }
        }
    } else {
        panic!("Entry (`{:?}`) point is not a file!", source_join);
    };

    let source_token = LitStr::new(&output.buffer, Span::call_site().into());

    let shader_source = match ShaderSource::parse(output.buffer.clone()) {
        Ok(shader_source) => shader_source,
        Err(err) => {
            let diagnostic = Diagnostic::error()
                .with_message(err.message().to_string())
                .with_labels(
                    err.labels()
                        .flat_map(|label| {
                            let source_range = label.0.clone().to_range()?;
                            let mapped_span = output.source_map.mapped_span(source_range).unwrap();

                            Some(
                                Label::primary(mapped_span.file_id, mapped_span.range.clone())
                                    .with_message(label.1.to_string()),
                            )
                        })
                        .collect(),
                );

            let config = codespan_reporting::term::Config::default();
            let writer = StandardStream::stderr(ColorChoice::Auto);

            term::emit(&mut writer.lock(), &config, &source_files, &diagnostic)
                .expect("cannot write error");

            panic!("failed to compile shader source");
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

    let tokens = quote! {
        empa::resource_binding::SizedBufferLayout(&[#(#recurse),*])
    };

    tokens
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
        empa::resource_binding::UnsizedBufferLayout {
            sized_head: &[#(#head_recurse),*],
            unsized_tail: #tail
        }
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

    match memory_unit_layout {
        MemoryUnitLayout::Float => {
            quote!(#mod_path::MemoryUnitLayout::Float)
        }
        MemoryUnitLayout::FloatArray(len) => {
            quote!(#mod_path::MemoryUnitLayout::Array {
                units: &[#mod_path::MemoryUnit {
                    offset: 0,
                    layout: #mod_path::MemoryUnitLayout::Float
                }],
                stride: 4,
                len: #len
            })
        }
        MemoryUnitLayout::FloatVector2 => {
            quote!(#mod_path::MemoryUnitLayout::FloatVector2)
        }
        MemoryUnitLayout::FloatVector2Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::Array {
                units: &[#mod_path::MemoryUnit {
                    offset: 0,
                    layout: #mod_path::MemoryUnitLayout::FloatVector2
                }],
                stride: 8,
                len: #len
            })
        }
        MemoryUnitLayout::FloatVector3 => {
            quote!(#mod_path::MemoryUnitLayout::FloatVector3)
        }
        MemoryUnitLayout::FloatVector3Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::Array {
                units: &[#mod_path::MemoryUnit {
                    offset: 0,
                    layout: #mod_path::MemoryUnitLayout::FloatVector3
                }],
                stride: 16,
                len: #len
            })
        }
        MemoryUnitLayout::FloatVector4 => {
            quote!(#mod_path::MemoryUnitLayout::FloatVector4)
        }
        MemoryUnitLayout::FloatVector4Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::Array {
                units: &[#mod_path::MemoryUnit {
                    offset: 0,
                    layout: #mod_path::MemoryUnitLayout::FloatVector4
                }],
                stride: 16,
                len: #len
            })
        }
        MemoryUnitLayout::Integer => {
            quote!(#mod_path::MemoryUnitLayout::Integer)
        }
        MemoryUnitLayout::IntegerArray(len) => {
            quote!(#mod_path::MemoryUnitLayout::Array {
                units: &[#mod_path::MemoryUnit {
                    offset: 0,
                    layout: #mod_path::MemoryUnitLayout::Integer
                }],
                stride: 4,
                len: #len
            })
        }
        MemoryUnitLayout::IntegerVector2 => {
            quote!(#mod_path::MemoryUnitLayout::IntegerVector2)
        }
        MemoryUnitLayout::IntegerVector2Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::Array {
                units: &[#mod_path::MemoryUnit {
                    offset: 0,
                    layout: #mod_path::MemoryUnitLayout::IntegerVector2
                }],
                stride: 8,
                len: #len
            })
        }
        MemoryUnitLayout::IntegerVector3 => {
            quote!(#mod_path::MemoryUnitLayout::IntegerVector3)
        }
        MemoryUnitLayout::IntegerVector3Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::Array {
                units: &[#mod_path::MemoryUnit {
                    offset: 0,
                    layout: #mod_path::MemoryUnitLayout::IntegerVector3
                }],
                stride: 16,
                len: #len
            })
        }
        MemoryUnitLayout::IntegerVector4 => {
            quote!(#mod_path::MemoryUnitLayout::IntegerVector4)
        }
        MemoryUnitLayout::IntegerVector4Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::Array {
                units: &[#mod_path::MemoryUnit {
                    offset: 0,
                    layout: #mod_path::MemoryUnitLayout::IntegerVector4
                }],
                stride: 16,
                len: #len
            })
        }
        MemoryUnitLayout::UnsignedInteger => {
            quote!(#mod_path::MemoryUnitLayout::UnsignedInteger)
        }
        MemoryUnitLayout::UnsignedIntegerArray(len) => {
            quote!(#mod_path::MemoryUnitLayout::Array {
                units: &[#mod_path::MemoryUnit {
                    offset: 0,
                    layout: #mod_path::MemoryUnitLayout::UnsignedInteger
                }],
                stride: 4,
                len: #len
            })
        }
        MemoryUnitLayout::UnsignedIntegerVector2 => {
            quote!(#mod_path::MemoryUnitLayout::UnsignedIntegerVector2)
        }
        MemoryUnitLayout::UnsignedIntegerVector2Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::Array {
                units: &[#mod_path::MemoryUnit {
                    offset: 0,
                    layout: #mod_path::MemoryUnitLayout::UnsignedIntegerVector2
                }],
                stride: 8,
                len: #len
            })
        }
        MemoryUnitLayout::UnsignedIntegerVector3 => {
            quote!(#mod_path::MemoryUnitLayout::UnsignedIntegerVector3)
        }
        MemoryUnitLayout::UnsignedIntegerVector3Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::Array {
                units: &[#mod_path::MemoryUnit {
                    offset: 0,
                    layout: #mod_path::MemoryUnitLayout::UnsignedIntegerVector3
                }],
                stride: 16,
                len: #len
            })
        }
        MemoryUnitLayout::UnsignedIntegerVector4 => {
            quote!(#mod_path::MemoryUnitLayout::UnsignedIntegerVector4)
        }
        MemoryUnitLayout::UnsignedIntegerVector4Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::Array {
                units: &[#mod_path::MemoryUnit {
                    offset: 0,
                    layout: #mod_path::MemoryUnitLayout::UnsignedIntegerVector4
                }],
                stride: 16,
                len: #len
            })
        }
        MemoryUnitLayout::Matrix2x2 => {
            quote!(#mod_path::MemoryUnitLayout::Matrix2x2)
        }
        MemoryUnitLayout::Matrix2x2Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::Array {
                units: &[#mod_path::MemoryUnit {
                    offset: 0,
                    layout: #mod_path::MemoryUnitLayout::Matrix2x2
                }],
                stride: 16,
                len: #len
            })
        }
        MemoryUnitLayout::Matrix2x3 => {
            quote!(#mod_path::MemoryUnitLayout::Matrix2x3)
        }
        MemoryUnitLayout::Matrix2x3Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::Array {
                units: &[#mod_path::MemoryUnit {
                    offset: 0,
                    layout: #mod_path::MemoryUnitLayout::Matrix2x3
                }],
                stride: 32,
                len: #len
            })
        }
        MemoryUnitLayout::Matrix2x4 => {
            quote!(#mod_path::MemoryUnitLayout::Matrix2x4)
        }
        MemoryUnitLayout::Matrix2x4Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::Array {
                units: &[#mod_path::MemoryUnit {
                    offset: 0,
                    layout: #mod_path::MemoryUnitLayout::Matrix2x4
                }],
                stride: 32,
                len: #len
            })
        }
        MemoryUnitLayout::Matrix3x2 => {
            quote!(#mod_path::MemoryUnitLayout::Matrix3x2)
        }
        MemoryUnitLayout::Matrix3x2Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::Array {
                units: &[#mod_path::MemoryUnit {
                    offset: 0,
                    layout: #mod_path::MemoryUnitLayout::Matrix3x2
                }],
                stride: 24,
                len: #len
            })
        }
        MemoryUnitLayout::Matrix3x3 => {
            quote!(#mod_path::MemoryUnitLayout::Matrix3x3)
        }
        MemoryUnitLayout::Matrix3x3Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::Array {
                units: &[#mod_path::MemoryUnit {
                    offset: 0,
                    layout: #mod_path::MemoryUnitLayout::Matrix3x3
                }],
                stride: 48,
                len: #len
            })
        }
        MemoryUnitLayout::Matrix3x4 => {
            quote!(#mod_path::MemoryUnitLayout::Matrix3x4)
        }
        MemoryUnitLayout::Matrix3x4Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::Array {
                units: &[#mod_path::MemoryUnit {
                    offset: 0,
                    layout: #mod_path::MemoryUnitLayout::Matrix3x4
                }],
                stride: 48,
                len: #len
            })
        }
        MemoryUnitLayout::Matrix4x2 => {
            quote!(#mod_path::MemoryUnitLayout::Matrix4x2)
        }
        MemoryUnitLayout::Matrix4x2Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::Array {
                units: &[#mod_path::MemoryUnit {
                    offset: 0,
                    layout: #mod_path::MemoryUnitLayout::Matrix4x2
                }],
                stride: 32,
                len: #len
            })
        }
        MemoryUnitLayout::Matrix4x3 => {
            quote!(#mod_path::MemoryUnitLayout::Matrix4x3)
        }
        MemoryUnitLayout::Matrix4x3Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::Array {
                units: &[#mod_path::MemoryUnit {
                    offset: 0,
                    layout: #mod_path::MemoryUnitLayout::Matrix4x3
                }],
                stride: 64,
                len: #len
            })
        }
        MemoryUnitLayout::Matrix4x4 => {
            quote!(#mod_path::MemoryUnitLayout::Matrix4x4)
        }
        MemoryUnitLayout::Matrix4x4Array(len) => {
            quote!(#mod_path::MemoryUnitLayout::Array {
                units: &[#mod_path::MemoryUnit {
                    offset: 0,
                    layout: #mod_path::MemoryUnitLayout::Matrix4x4
                }],
                stride: 64,
                len: #len
            })
        }
        MemoryUnitLayout::ComplexArray { units, stride, len } => {
            let recurse = units.iter().map(|unit| memory_unit_tokens(unit));

            quote!(#mod_path::MemoryUnitLayout::Array {
                units: &[#(#recurse),*],
                stride: #stride,
                len: #len,
            })
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
