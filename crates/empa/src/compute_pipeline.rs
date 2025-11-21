use std::collections::HashMap;
use std::future::Future;
use std::marker;

use atomic_counter::AtomicCounter;
use empa_smi::ShaderStage;
use futures::FutureExt;

use crate::device::{Device, ID_GEN};
use crate::driver;
use crate::driver::{Device as _, Driver, Dvr};
use crate::pipeline_constants::PipelineConstants;
use crate::resource_binding::{PipelineLayout, TypedPipelineLayout};
use crate::shader_module::{EntryPointExt, ShaderModule, ShaderModuleData};

pub struct ComputePipeline<L> {
    pub(crate) handle: <Dvr as Driver>::ComputePipelineHandle,
    id: usize,
    _marker: marker::PhantomData<*const L>,
}

impl<L> ComputePipeline<L> {
    pub(crate) fn new_sync(device: &Device, descriptor: &ComputePipelineDescriptor<L>) -> Self {
        let entry_point = descriptor.compute_stage.entry_point();

        let desc = driver::ComputePipelineDescriptor {
            layout: &descriptor.layout,
            shader_module: &descriptor.compute_stage.shader_module_data.handle,
            entry_point: entry_point.name.as_ref(),
            constants: &descriptor.compute_stage.pipeline_constants,
        };

        let id = ID_GEN.get();
        let handle = device.device_handle.create_compute_pipeline(&desc);

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
            shader_module: &descriptor.compute_stage.shader_module_data.handle,
            entry_point: descriptor.compute_stage.entry_point_name(),
            constants: &descriptor.compute_stage.pipeline_constants,
        };

        device
            .device_handle
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

        for resource_binding in compute_stage.entry_point().resource_bindings() {
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

            if entry.resource_type != resource_binding.resource_type {
                panic!(
                    "the binding type for binding `{}` in group `{}` does not match the shader \
                    type (shader: {:#?}, actual: {:#?})",
                    resource_binding.binding,
                    resource_binding.group,
                    &resource_binding.resource_type,
                    &entry.resource_type
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
    pub(crate) shader_module_data: ShaderModuleData,
    pub(crate) entry_point_index: usize,
    pub(crate) pipeline_constants: HashMap<String, f64>,
}

impl ComputeStage {
    fn entry_point(&self) -> EntryPointExt<'_> {
        self.shader_module_data
            .entry_point_ext(self.entry_point_index)
    }

    pub(crate) fn entry_point_name(&self) -> &str {
        self.shader_module_data.smi.entry_points[self.entry_point_index]
            .name
            .as_ref()
    }
}

pub struct ComputeStageBuilder {
    compute_stage: ComputeStage,
    has_constants: bool,
}

impl ComputeStageBuilder {
    pub fn begin(shader_module: &ShaderModule, entry_point: &str) -> Self {
        let shader_module_data = shader_module.data.clone();
        let entry_point_index = shader_module_data
            .resolve_entry_point_index(entry_point)
            .expect("could not find entry point in shader module");
        let stage = shader_module_data.smi.entry_points[entry_point_index].stage;

        assert_eq!(
            stage,
            ShaderStage::Compute,
            "entry point is not a compute stage"
        );

        let compute_stage = ComputeStage {
            shader_module_data,
            entry_point_index,
            pipeline_constants: Default::default(),
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
            .entry_point()
            .build_constants(pipeline_constants);

        self
    }

    pub fn finish(self) -> ComputeStage {
        if !self.has_constants && self.compute_stage.entry_point().has_required_constants() {
            panic!(
                "the shader accesses pipeline constants without fallback values, but no pipeline \
                constants were set"
            );
        }

        self.compute_stage
    }
}
