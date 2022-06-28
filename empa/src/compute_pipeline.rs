use std::marker;

use atomic_counter::AtomicCounter;
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use web_sys::{GpuComputePipeline, GpuComputePipelineDescriptor, GpuProgrammableStage};

use crate::device::{Device, ID_GEN};
use crate::pipeline_constants::PipelineConstants;
use crate::resource_binding::{PipelineLayout, ShaderStages, TypedPipelineLayout};
use crate::shader_module::{ShaderModule, ShaderSourceInternal, StaticShaderStage};

pub struct ComputePipeline<L> {
    inner: GpuComputePipeline,
    id: usize,
    _marker: marker::PhantomData<L>,
}

impl<L> ComputePipeline<L> {
    pub(crate) fn new(device: &Device, descriptor: &ComputePipelineDescriptor<L>) -> Self {
        let id = ID_GEN.get();
        let inner = device.inner.create_compute_pipeline(&descriptor.inner);

        ComputePipeline {
            inner,
            id,
            _marker: Default::default(),
        }
    }

    pub(crate) fn id(&self) -> usize {
        self.id
    }

    pub(crate) fn as_web_sys(&self) -> &GpuComputePipeline {
        &self.inner
    }
}

pub struct ComputePipelineDescriptor<L> {
    inner: GpuComputePipelineDescriptor,
    _marker: marker::PhantomData<*const L>,
}

pub struct ComputePipelineDescriptorBuilder<L, S> {
    inner: GpuComputePipelineDescriptor,
    _marker: marker::PhantomData<(*const L, *const S)>,
}

impl ComputePipelineDescriptorBuilder<(), ()> {
    pub fn begin() -> Self {
        let inner = GpuComputePipelineDescriptor::new(
            JsValue::null().unchecked_ref(),
            JsValue::null().unchecked_ref(),
        );

        ComputePipelineDescriptorBuilder {
            inner,
            _marker: Default::default(),
        }
    }

    pub fn layout<Layout>(
        mut self,
        layout: &PipelineLayout<Layout>,
    ) -> ComputePipelineDescriptorBuilder<PipelineLayout<Layout>, ()> {
        self.inner.layout(&layout.inner);

        ComputePipelineDescriptorBuilder {
            inner: self.inner,
            _marker: Default::default(),
        }
    }
}

impl<Layout: TypedPipelineLayout> ComputePipelineDescriptorBuilder<PipelineLayout<Layout>, ()> {
    pub fn compute(
        mut self,
        compute_stage: &ComputeStage,
    ) -> ComputePipelineDescriptorBuilder<PipelineLayout<Layout>, ComputeStage> {
        let layout = Layout::BIND_GROUP_LAYOUTS;

        for resource_binding in compute_stage.shader_meta.resource_bindings() {
            let group = if let Some(group) = layout.get(resource_binding.group as usize) {
                group
            } else {
                panic!("shader expects bind group `{}`", resource_binding.group);
            };

            let entry = if let Some(Some(entry)) = group.get(resource_binding.binding as usize) {
                entry
            } else {
                panic!(
                    "shader expects binding `{}` in group `{}`",
                    resource_binding.binding, resource_binding.group
                );
            };

            if !entry.visibility.intersects(ShaderStages::COMPUTE) {
                panic!(
                    "binding `{}` in group `{}` is not visible to the compute stage",
                    resource_binding.binding, resource_binding.group
                );
            }

            if entry.binding_type != resource_binding.binding_type {
                panic!("the binding type for binding `{}` in group `{}` does not match the shader type", resource_binding.binding, resource_binding.group)
            }
        }

        self.inner.compute(&compute_stage.inner);

        ComputePipelineDescriptorBuilder {
            inner: self.inner,
            _marker: Default::default(),
        }
    }
}

impl<Layout: TypedPipelineLayout>
    ComputePipelineDescriptorBuilder<PipelineLayout<Layout>, ComputeStage>
{
    pub fn finish(self) -> ComputePipelineDescriptor<Layout> {
        ComputePipelineDescriptor {
            inner: self.inner,
            _marker: Default::default(),
        }
    }
}

pub struct ComputeStage {
    pub(crate) inner: GpuProgrammableStage,
    pub(crate) shader_meta: ShaderSourceInternal,
}

pub struct ComputeStageBuilder {
    inner: GpuProgrammableStage,
    shader_meta: ShaderSourceInternal,
    entry_index: usize,
    has_constants: bool,
}

impl ComputeStageBuilder {
    pub fn begin(shader_module: &ShaderModule, entry_point: &str) -> Self {
        let inner = GpuProgrammableStage::new(entry_point, &shader_module.inner);
        let shader_meta = shader_module.meta.clone();

        let (entry_index, entry) = shader_meta
            .entry_points()
            .iter()
            .enumerate()
            .find(|(_, e)| e.name == entry_point)
            .expect("could not find entry point in shader module");

        assert!(
            entry.stage == StaticShaderStage::Compute,
            "entry point is not a compute stage"
        );

        ComputeStageBuilder {
            inner,
            shader_meta,
            entry_index,
            has_constants: false,
        }
    }

    pub fn pipeline_constants<C: PipelineConstants>(
        self,
        pipeline_constants: &C,
    ) -> ComputeStageBuilder {
        let ComputeStageBuilder {
            inner,
            shader_meta,
            entry_index,
            ..
        } = self;

        let record = shader_meta.build_constants(pipeline_constants);

        // TODO: get support for WebIDL record types in wasm bindgen
        js_sys::Reflect::set(inner.as_ref(), &JsValue::from("constants"), &record).unwrap_throw();

        ComputeStageBuilder {
            inner,
            shader_meta,
            entry_index,
            has_constants: true,
        }
    }

    pub fn finish(self) -> ComputeStage {
        let ComputeStageBuilder {
            inner,
            shader_meta,
            has_constants,
            ..
        } = self;

        if !has_constants && shader_meta.constants().iter().any(|c| c.required) {
            panic!("the shader declares pipeline constants without fallback values, but no pipeline constants were set");
        }

        ComputeStage { inner, shader_meta }
    }
}
