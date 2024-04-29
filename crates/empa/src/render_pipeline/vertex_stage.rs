use std::marker;

use empa_reflect::ShaderStage;
use wasm_bindgen::{JsValue, UnwrapThrowExt};
use web_sys::{GpuVertexBufferLayout, GpuVertexState};

use crate::pipeline_constants::PipelineConstants;
use crate::render_pipeline::TypedVertexLayout;
use crate::shader_module::{ShaderModule, ShaderSourceInternal};

pub struct VertexStage<V> {
    pub(crate) inner: GpuVertexState,
    pub(crate) shader_meta: ShaderSourceInternal,
    entry_index: usize,
    _marker: marker::PhantomData<*const V>,
}

pub struct VertexStageBuilder<V> {
    inner: GpuVertexState,
    shader_meta: ShaderSourceInternal,
    entry_index: usize,
    has_constants: bool,
    _marker: marker::PhantomData<*const V>,
}

impl VertexStageBuilder<()> {
    pub fn begin(shader_module: &ShaderModule, entry_point: &str) -> Self {
        let inner = GpuVertexState::new(entry_point, &shader_module.inner);
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
            inner,
            shader_meta,
            entry_index,
            has_constants: false,
            _marker: Default::default(),
        }
    }

    pub fn vertex_layout<V: TypedVertexLayout>(self) -> VertexStageBuilder<V> {
        let VertexStageBuilder {
            mut inner,
            shader_meta,
            entry_index,
            has_constants,
            ..
        } = self;

        let layout = V::LAYOUT;

        let input_bindings = shader_meta.entry_point_input_bindings(entry_index).unwrap();

        // Unclear if this can be optimized by e.g. sorting first. The default limit for attributes
        // is 16, so the upper limit would be roughly 1024 reads and comparisons on a piece of
        // data that easily fits in cache; may not be able to beat simple repeated iteration.
        'outer: for binding in input_bindings {
            let location = binding.location();

            for descriptor in layout {
                for attribute in descriptor.attribute_descriptors {
                    if attribute.shader_location == location {
                        if !attribute.format.is_compatible(binding.binding_type()) {
                            panic!("attribute for location `{}` is not compatible with the shader type", location);
                        }

                        continue 'outer;
                    }
                }
            }

            panic!("no attribute found for location `{}`", location);
        }

        let layout_array = js_sys::Array::new();

        for descriptor in layout {
            let attributes: js_sys::Array = descriptor
                .attribute_descriptors
                .iter()
                .map(|a| a.to_web_sys())
                .collect();
            let mut buffer_layout =
                GpuVertexBufferLayout::new(descriptor.array_stride as f64, attributes.as_ref());

            buffer_layout.step_mode(descriptor.input_rate.to_web_sys());

            layout_array.push(buffer_layout.as_ref());
        }

        inner.buffers(layout_array.as_ref());

        VertexStageBuilder {
            inner,
            shader_meta,
            entry_index,
            has_constants,
            _marker: Default::default(),
        }
    }
}

impl<V> VertexStageBuilder<V> {
    pub fn pipeline_constants<C: PipelineConstants>(
        self,
        pipeline_constants: &C,
    ) -> VertexStageBuilder<V> {
        let VertexStageBuilder {
            inner,
            shader_meta,
            entry_index,
            ..
        } = self;

        let record = shader_meta.build_constants(pipeline_constants);

        // TODO: get support for WebIDL record types in wasm bindgen
        js_sys::Reflect::set(inner.as_ref(), &JsValue::from("constants"), &record).unwrap_throw();

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

        if !has_constants && shader_meta.has_required_constants() {
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
