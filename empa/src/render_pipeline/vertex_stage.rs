use crate::pipeline_constants::PipelineConstants;
use crate::render_pipeline::TypedVertexLayout;
use crate::shader_module::{ShaderModule, ShaderSourceInternal, StaticShaderStage};
use std::marker;
use std::sync::Arc;
use wasm_bindgen::JsValue;
use web_sys::GpuVertexState;

pub struct VertexStage<O> {
    pub(crate) inner: GpuVertexState,
    pub(crate) shader_meta: Arc<ShaderSourceInternal>,
    entry_index: usize,
    _marker: marker::PhantomData<*const O>,
}

pub struct VertexStageBuilder<O> {
    inner: GpuVertexState,
    shader_meta: Arc<ShaderSourceInternal>,
    entry_index: usize,
    has_constants: bool,
    _marker: marker::PhantomData<*const O>,
}

impl VertexStageBuilder<()> {
    pub fn begin(shader_module: &ShaderModule, entry_point: &str) -> Self {
        let inner = GpuVertexState::new(entry_point, &shader_module.inner);
        let shader_meta = shader_module.meta.clone();

        let (entry_index, entry) = shader_meta
            .entry_points()
            .iter()
            .enumerate()
            .find(|(_, e)| e.name == entry_point)
            .expect("could not find entry point in shader module");

        assert!(
            entry.stage == StaticShaderStage::Vertex,
            "entry point is not a vertex stage"
        );

        VertexStageBuilder {
            inner,
            shader_meta,
            entry_index,
            has_constants: false,
            _marker: Default::default(),
        }
    }

    pub fn vertex_layout<V: TypedVertexLayout>(self, color_outputs: V) -> VertexStageBuilder<V> {
        let VertexStageBuilder {
            inner,
            shader_meta,
            entry_index,
            has_constants,
            ..
        } = self;

        let layout = V::LAYOUT;
        let entry = &shader_meta.entry_points()[entry_index];

        // Unclear if this can be optimized by e.g. sorting first. The default limit for attributes
        // is 16, so the upper limit would be roughly 1024 reads and comparisons on a piece of
        // data that easily fits in cache; may not be able to beat simple repeated iteration.
        'outer: for binding in entry.input_bindings {
            for descriptor in layout {
                for attribute in descriptor.attribute_descriptors {
                    if attribute.shader_location == binding.location {
                        if !attribute.format.is_compatible(binding.binding_type) {
                            panic!("attribute for location `{}` is not compatible with the shader type", binding.location);
                        }

                        continue 'outer;
                    }
                }
            }

            panic!("no attribute found for location `{}`", binding.location);
        }

        VertexStageBuilder {
            inner,
            shader_meta,
            entry_index,
            has_constants,
            _marker: Default::default(),
        }
    }
}

impl<O> VertexStageBuilder<O> {
    pub fn pipeline_constants<C: PipelineConstants>(
        self,
        pipeline_constants: &C,
    ) -> VertexStageBuilder<O> {
        let VertexStageBuilder {
            inner,
            shader_meta,
            entry_index,
            ..
        } = self;

        let record = shader_meta.build_constants(pipeline_constants);

        // TODO: update web_sys; currently no way to actually set constants
        todo!();

        VertexStageBuilder {
            inner,
            shader_meta,
            entry_index,
            has_constants: true,
            _marker: Default::default(),
        }
    }
}

impl<V> VertexStageBuilder<V>
where
    V: TypedVertexLayout,
{
    pub fn finish(self) -> VertexStage<V> {
        let VertexStageBuilder {
            inner,
            shader_meta,
            entry_index,
            has_constants,
            ..
        } = self;

        if !has_constants && shader_meta.constants().iter().any(|c| c.required) {
            panic!("the shader declares pipeline constants without fallback values, but no pipeline constants were set");
        }

        VertexStage {
            inner,
            shader_meta,
            entry_index,
            _marker: Default::default(),
        }
    }
}
