use std::borrow::Cow;
use std::collections::HashMap;
use std::marker;

use empa_reflect::ShaderStage;

use crate::driver::{Driver, Dvr};
use crate::pipeline_constants::PipelineConstants;
use crate::render_pipeline::vertex_attribute::vertex_format_is_compatible;
use crate::render_pipeline::{TypedVertexLayout, VertexBufferLayout};
use crate::shader_module::{ShaderModule, ShaderSourceInternal};

pub(crate) struct VertexState {
    pub(crate) shader_module: <Dvr as Driver>::ShaderModuleHandle,
    pub(crate) entry_point: String,
    pub(crate) constants: HashMap<String, f64>,
    pub(crate) vertex_buffer_layouts: Cow<'static, [VertexBufferLayout<'static>]>,
}

pub struct VertexStage<V> {
    pub(crate) vertex_state: VertexState,
    pub(crate) shader_meta: ShaderSourceInternal,
    entry_index: usize,
    _marker: marker::PhantomData<*const V>,
}

pub struct VertexStageBuilder<V> {
    inner: VertexStage<V>,
    has_constants: bool,
}

impl VertexStageBuilder<()> {
    pub fn begin(shader_module: &ShaderModule, entry_point: &str) -> Self {
        let shader_meta = shader_module.meta.clone();

        let entry_index = shader_meta
            .resolve_entry_point_index(entry_point)
            .expect("could not find entry point in shader module");
        let stage = shader_meta.entry_point_stage(entry_index);

        assert!(
            stage == Some(ShaderStage::Vertex),
            "entry point is not a vertex stage"
        );

        VertexStageBuilder {
            inner: VertexStage {
                vertex_state: VertexState {
                    shader_module: shader_module.handle.clone(),
                    entry_point: entry_point.to_string(),
                    constants: Default::default(),
                    vertex_buffer_layouts: Cow::Owned(Vec::new()),
                },
                shader_meta,
                entry_index,
                _marker: Default::default(),
            },
            has_constants: false,
        }
    }

    pub fn vertex_layout<V: TypedVertexLayout>(mut self) -> VertexStageBuilder<V> {
        let layout = V::LAYOUT;

        let input_bindings = self
            .inner
            .shader_meta
            .entry_point_input_bindings(self.inner.entry_index)
            .unwrap();

        // Unclear if this can be optimized by e.g. sorting first. The default limit for attributes
        // is 16, so the upper limit would be roughly 1024 reads and comparisons on a piece of
        // data that easily fits in cache; may not be able to beat simple repeated iteration.
        'outer: for binding in input_bindings {
            let location = binding.location();

            for buffer_layout in layout {
                for attribute in buffer_layout.attributes.iter() {
                    if attribute.shader_location == location {
                        if !vertex_format_is_compatible(attribute.format, binding.binding_type()) {
                            panic!("attribute for location `{}` is not compatible with the shader type", location);
                        }

                        continue 'outer;
                    }
                }
            }

            panic!("no attribute found for location `{}`", location);
        }

        self.inner.vertex_state.vertex_buffer_layouts = Cow::Borrowed(layout);

        VertexStageBuilder {
            inner: VertexStage {
                vertex_state: self.inner.vertex_state,
                shader_meta: self.inner.shader_meta,
                entry_index: self.inner.entry_index,
                _marker: Default::default(),
            },
            has_constants: self.has_constants,
        }
    }
}

impl<V> VertexStageBuilder<V> {
    pub fn pipeline_constants<C: PipelineConstants>(
        mut self,
        pipeline_constants: &C,
    ) -> VertexStageBuilder<V> {
        self.inner.vertex_state.constants =
            self.inner.shader_meta.build_constants(pipeline_constants);

        self
    }
}

impl<V> VertexStageBuilder<V>
where
    V: TypedVertexLayout,
{
    pub fn finish(self) -> VertexStage<V> {
        if !self.has_constants && self.inner.shader_meta.has_required_constants() {
            panic!("the shader declares pipeline constants without fallback values, but no pipeline constants were set");
        }

        self.inner
    }
}
