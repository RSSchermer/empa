use std::collections::HashMap;
use std::future::Future;
use std::marker;

use atomic_counter::AtomicCounter;
use empa_reflect::ShaderStage;
use futures::FutureExt;

use crate::device::{Device, ID_GEN};
use crate::driver;
use crate::driver::{Device as _, Driver, Dvr};
use crate::pipeline_constants::PipelineConstants;
use crate::resource_binding::{PipelineLayout, TypedPipelineLayout};
use crate::shader_module::{ShaderModule, ShaderSourceInternal};

pub struct ComputePipeline<L> {
    pub(crate) handle: <Dvr as Driver>::ComputePipelineHandle,
    id: usize,
    _marker: marker::PhantomData<*const L>,
}

impl<L> ComputePipeline<L> {
    pub(crate) fn new_sync(device: &Device, descriptor: &ComputePipelineDescriptor<L>) -> Self {
        let desc = driver::ComputePipelineDescriptor {
            layout: &descriptor.layout,
            shader_module: &descriptor.compute_stage.shader_module,
            entry_point: &descriptor.compute_stage.entry_point,
            constants: &descriptor.compute_stage.pipeline_constants,
        };

        let id = ID_GEN.get();
        let handle = device.handle.create_compute_pipeline(&desc);

        ComputePipeline {
            handle,
            id,
            _marker: Default::default(),
        }
    }

    pub(crate) fn new_async(
        device: &Device,
        descriptor: &ComputePipelineDescriptor<L>,
    ) -> impl Future<Output = Self> {
        let desc = driver::ComputePipelineDescriptor {
            layout: &descriptor.layout,
            shader_module: &descriptor.compute_stage.shader_module,
            entry_point: &descriptor.compute_stage.entry_point,
            constants: &descriptor.compute_stage.pipeline_constants,
        };

        device
            .handle
            .create_compute_pipeline_async(&desc)
            .map(|handle| {
                let id = ID_GEN.get();

                ComputePipeline {
                    handle,
                    id,
                    _marker: Default::default(),
                }
            })
    }

    pub(crate) fn id(&self) -> usize {
        self.id
    }
}

pub struct ComputePipelineDescriptor<L> {
    compute_stage: ComputeStage,
    layout: <Dvr as Driver>::PipelineLayoutHandle,
    _marker: marker::PhantomData<*const L>,
}

pub struct ComputePipelineDescriptorBuilder<L, S> {
    compute_stage: Option<ComputeStage>,
    layout: Option<<Dvr as Driver>::PipelineLayoutHandle>,
    _marker: marker::PhantomData<(*const L, *const S)>,
}

impl ComputePipelineDescriptorBuilder<(), ()> {
    pub fn begin() -> Self {
        ComputePipelineDescriptorBuilder {
            compute_stage: None,
            layout: None,
            _marker: Default::default(),
        }
    }

    pub fn layout<Layout>(
        self,
        layout: &PipelineLayout<Layout>,
    ) -> ComputePipelineDescriptorBuilder<PipelineLayout<Layout>, ()> {
        ComputePipelineDescriptorBuilder {
            compute_stage: self.compute_stage,
            layout: Some(layout.handle.clone()),
            _marker: Default::default(),
        }
    }
}

impl<Layout: TypedPipelineLayout> ComputePipelineDescriptorBuilder<PipelineLayout<Layout>, ()> {
    pub fn compute(
        self,
        compute_stage: ComputeStage,
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

            if !entry.visibility.contains(driver::ShaderStage::Compute) {
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

        ComputePipelineDescriptorBuilder {
            compute_stage: Some(compute_stage),
            layout: self.layout,
            _marker: Default::default(),
        }
    }
}

impl<Layout> ComputePipelineDescriptorBuilder<PipelineLayout<Layout>, ()> {
    pub unsafe fn compute_unchecked(
        self,
        compute_stage: ComputeStage,
    ) -> ComputePipelineDescriptorBuilder<PipelineLayout<Layout>, ComputeStage> {
        ComputePipelineDescriptorBuilder {
            compute_stage: Some(compute_stage),
            layout: self.layout,
            _marker: Default::default(),
        }
    }
}

impl<Layout> ComputePipelineDescriptorBuilder<PipelineLayout<Layout>, ComputeStage> {
    pub fn finish(self) -> ComputePipelineDescriptor<Layout> {
        ComputePipelineDescriptor {
            compute_stage: self.compute_stage.unwrap(),
            layout: self.layout.unwrap(),
            _marker: Default::default(),
        }
    }
}

pub struct ComputeStage {
    pub(crate) shader_module: <Dvr as Driver>::ShaderModuleHandle,
    pub(crate) entry_point: String,
    pub(crate) pipeline_constants: HashMap<String, f64>,
    pub(crate) shader_meta: ShaderSourceInternal,
}

pub struct ComputeStageBuilder {
    compute_stage: ComputeStage,
    has_constants: bool,
}

impl ComputeStageBuilder {
    pub fn begin(shader_module: &ShaderModule, entry_point: &str) -> Self {
        let shader_meta = shader_module.meta.clone();

        let entry_index = shader_meta
            .resolve_entry_point_index(entry_point)
            .expect("could not find entry point in shader module");
        let stage = shader_meta.entry_point_stage(entry_index);

        assert!(
            stage == Some(ShaderStage::Compute),
            "entry point is not a compute stage"
        );

        let compute_stage = ComputeStage {
            shader_module: shader_module.handle.clone(),
            entry_point: entry_point.to_string(),
            pipeline_constants: Default::default(),
            shader_meta,
        };

        ComputeStageBuilder {
            compute_stage,
            has_constants: false,
        }
    }

    pub fn pipeline_constants<C: PipelineConstants>(
        mut self,
        pipeline_constants: &C,
    ) -> ComputeStageBuilder {
        self.compute_stage.pipeline_constants = self
            .compute_stage
            .shader_meta
            .build_constants(pipeline_constants);

        self
    }

    pub fn finish(self) -> ComputeStage {
        if !self.has_constants && self.compute_stage.shader_meta.has_required_constants() {
            panic!("the shader declares pipeline constants without fallback values, but no pipeline constants were set");
        }

        self.compute_stage
    }
}
