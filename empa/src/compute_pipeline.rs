use std::marker;

use atomic_counter::AtomicCounter;
use empa_reflect::ShaderStage;
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use web_sys::{GpuComputePipeline, GpuComputePipelineDescriptor, GpuProgrammableStage};

use crate::device::{Device, ID_GEN};
use crate::pipeline_constants::PipelineConstants;
use crate::resource_binding::{PipelineLayout, ShaderStages, TypedPipelineLayout};
use crate::shader_module::{ShaderModule, ShaderSourceInternal};

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
                panic!(
                    "the binding type for binding `{}` in group `{}` does not match the shader \
                type (shader: {:#?}, actual: {:#?})",
                    resource_binding.binding,
                    resource_binding.group,
                    &resource_binding.binding_type,
                    &entry.binding_type
                )
            }
        }

        self.inner.compute(&compute_stage.inner);

        ComputePipelineDescriptorBuilder {
            inner: self.inner,
            _marker: Default::default(),
        }
    }
}

impl<Layout> ComputePipelineDescriptorBuilder<PipelineLayout<Layout>, ()> {
    pub unsafe fn compute_unchecked(
        mut self,
        compute_stage: &ComputeStage,
    ) -> ComputePipelineDescriptorBuilder<PipelineLayout<Layout>, ComputeStage> {
        self.inner.compute(&compute_stage.inner);

        ComputePipelineDescriptorBuilder {
            inner: self.inner,
            _marker: Default::default(),
        }
    }
}

impl<Layout> ComputePipelineDescriptorBuilder<PipelineLayout<Layout>, ComputeStage> {
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

        let entry_index = shader_meta
            .resolve_entry_point_index(entry_point)
            .expect("could not find entry point in shader module");
        let stage = shader_meta.entry_point_stage(entry_index);

        assert!(
            stage == Some(ShaderStage::Compute),
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

        if !has_constants && shader_meta.has_required_constants() {
            panic!("the shader declares pipeline constants without fallback values, but no pipeline constants were set");
        }

        ComputeStage { inner, shader_meta }
    }
}
