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
use empa_smi::smi_to_token_stream;
use empa_smi::wgsl::{BuildSmiError, build_smi};
use include_preprocessor::{
    Error as IppError, OutputSink, SearchPaths, SourceMappedChunk, SourceTracker,
    preprocess_with_source_tracker,
};
use proc_macro::{Span, TokenStream, tracked};
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

        tracked::path(&path);
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

pub fn expand_shader_wgsl(input: TokenStream) -> TokenStream {
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

        match preprocess_with_source_tracker(&source_join, search_paths, writer, &mut source_files)
        {
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

                term::emit_to_write_style(&mut writer.lock(), &config, &file, &diagnostic)
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
        Ok(smi) => smi_to_token_stream(&smi, &quote!(empa::smi)),
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

                    term::emit_to_write_style(
                        &mut writer.lock(),
                        &config,
                        &source_files,
                        &diagnostic,
                    )
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

                    term::emit_to_write_style(
                        &mut writer.lock(),
                        &config,
                        &source_files,
                        &diagnostic,
                    )
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
