use std::borrow::Cow;
use std::collections::HashMap;
use std::marker;

use empa_smi::ShaderStage;

use crate::pipeline_constants::PipelineConstants;
use crate::render_pipeline::vertex_attribute::vertex_format_is_compatible;
use crate::render_pipeline::{TypedVertexLayout, VertexBufferLayout};
use crate::shader_module::{EntryPointExt, ShaderModule, ShaderModuleData};

pub(crate) struct VertexState {
    pub(crate) shader_module_data: ShaderModuleData,
    pub(crate) entry_point_index: usize,
    pub(crate) pipeline_constants: HashMap<String, f64>,
    pub(crate) vertex_buffer_layouts: Cow<'static, [VertexBufferLayout<'static>]>,
}

impl VertexState {
    pub(crate) fn entry_point(&self) -> EntryPointExt<'_> {
        self.shader_module_data
            .entry_point_ext(self.entry_point_index)
    }

    pub(crate) fn entry_point_name(&self) -> &str {
        self.shader_module_data.smi.entry_points[self.entry_point_index]
            .name
            .as_ref()
    }
}

pub struct VertexStage<V> {
    pub(crate) state: VertexState,
    _marker: marker::PhantomData<*const V>,
}

pub struct VertexStageBuilder<V> {
    inner: VertexStage<V>,
    has_constants: bool,
}

impl VertexStageBuilder<()> {
    pub fn begin(shader_module: &ShaderModule, entry_point: &str) -> Self {
        let shader_module_data = shader_module.data.clone();

        let entry_point_index = shader_module_data
            .resolve_entry_point_index(entry_point)
            .expect("could not find entry point in shader module");
        let stage = shader_module_data.smi.entry_points[entry_point_index].stage;

        assert_eq!(
            stage,
            ShaderStage::Vertex,
            "entry point is not a vertex stage"
        );

        VertexStageBuilder {
            inner: VertexStage {
                state: VertexState {
                    shader_module_data,
                    entry_point_index,
                    pipeline_constants: Default::default(),
                    vertex_buffer_layouts: Cow::Owned(Vec::new()),
                },
                _marker: Default::default(),
            },
            has_constants: false,
        }
    }

    pub fn vertex_layout<V: TypedVertexLayout>(mut self) -> VertexStageBuilder<V> {
        let layout = V::LAYOUT;
        let entry_point = self.inner.state.entry_point();
        let input_bindings = entry_point.input_bindings.as_ref();

        // Unclear if this can be optimized by e.g. sorting first. The default limit for attributes
        // is 16, so the upper limit would be roughly 1024 reads and comparisons on a piece of
        // data that easily fits in cache; may not be able to beat simple repeated iteration.
        'outer: for binding in input_bindings {
            let location = binding.location;

            for buffer_layout in layout {
                for attribute in buffer_layout.attributes.iter() {
                    if attribute.shader_location == location {
                        if !vertex_format_is_compatible(attribute.format, binding.binding_type) {
                            panic!(
                                "attribute for location `{}` is not compatible with the shader type",
                                location
                            );
                        }

                        continue 'outer;
                    }
                }
            }

            panic!("no attribute found for location `{}`", location);
        }

        self.inner.state.vertex_buffer_layouts = Cow::Borrowed(layout);

        VertexStageBuilder {
            inner: VertexStage {
                state: self.inner.state,
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
        self.inner.state.pipeline_constants = self
            .inner
            .state
            .entry_point()
            .build_constants(pipeline_constants);

        self
    }
}

impl<V> VertexStageBuilder<V>
where
    V: TypedVertexLayout,
{
    pub fn finish(self) -> VertexStage<V> {
        if !self.has_constants && self.inner.state.entry_point().has_required_constants() {
            panic!(
                "the shader declares pipeline constants without fallback values, but no pipeline \
                constants were set"
            );
        }

        self.inner
    }
}
