use crate::buffer::{AsBuffer, Buffer};
use std::mem::MaybeUninit;

use crate::command::{CommandBuffer, CommandEncoder};
use crate::compute_pipeline::ComputePipelineDescriptor;
use crate::query::OcclusionQuerySet;
use crate::render_pipeline::{ComputePipeline, RenderPipeline, RenderPipelineDescriptor};
use crate::resource_binding::{
    BindGroup, BindGroupLayout, BindGroupLayoutEntry, PipelineLayout, Resources,
    TypedBindGroupLayout, TypedPipelineLayout,
};
use crate::sampler::{
    AnisotropicSamplerDescriptor, ComparisonSampler, ComparisonSamplerDescriptor,
    NonFilteringSampler, NonFilteringSamplerDescriptor, Sampler, SamplerDescriptor,
};
use crate::texture::format::{
    MultisampleFormat, Texture1DFormat, Texture2DFormat, Texture3DFormat, ViewFormats,
};
use crate::texture::{
    Texture1D, Texture2D, Texture2DDescriptor, Texture3D, Texture3DDescriptor,
    TextureMultisampled2D, TextureMultisampled2DDescriptor,
};
use crate::{buffer, texture};
use atomic_counter::RelaxedCounter;
use lazy_static::lazy_static;
use web_sys::{GpuDevice, GpuQueue};
use crate::adapter::{Limits, Features};

lazy_static! {
    pub(crate) static ref ID_GEN: RelaxedCounter = RelaxedCounter::new(1);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct DeviceDescriptor {
    pub required_features: Features,
    pub required_limits: Limits
}

pub struct Device {
    pub(crate) inner: GpuDevice,
}

impl Device {
    pub fn create_buffer<D, T, U>(&self, data: D) -> Buffer<T, U>
    where
        D: AsBuffer<T>,
        U: buffer::ValidUsageFlags,
    {
        data.as_buffer(self, false)
    }

    pub fn create_buffer_mapped<D, T, U>(&self, data: D) -> Buffer<T, U>
    where
        D: AsBuffer<T>,
        U: buffer::ValidUsageFlags,
    {
        data.as_buffer(self, true)
    }

    pub fn create_buffer_uninit<T, U>(&self) -> Buffer<MaybeUninit<T>, U>
    where
        U: buffer::ValidUsageFlags,
    {
        Buffer::create_uninit(self, false)
    }

    pub fn create_buffer_uninit_mapped<T, U>(&self) -> Buffer<MaybeUninit<T>, U>
    where
        U: buffer::ValidUsageFlags,
    {
        Buffer::create_uninit(self, true)
    }

    pub fn create_slice_buffer_uninit<T, U>(&self, len: usize) -> Buffer<[MaybeUninit<T>], U>
    where
        U: buffer::ValidUsageFlags,
    {
        Buffer::create_slice_uninit(self, len, false)
    }

    pub fn create_slice_buffer_uninit_mapped<T, U>(&self, len: usize) -> Buffer<[MaybeUninit<T>], U>
    where
        U: buffer::ValidUsageFlags,
    {
        Buffer::create_slice_uninit(self, len, true)
    }

    pub fn create_bind_group_layout<T>(&self) -> BindGroupLayout<T>
    where
        T: TypedBindGroupLayout,
    {
        BindGroupLayout::typed(self)
    }

    pub fn create_untyped_bind_group_layout(
        &self,
        layout: &[Option<BindGroupLayoutEntry>],
    ) -> BindGroupLayout {
        BindGroupLayout::untyped(self, layout)
    }

    pub fn create_pipeline_layout<T>(&self) -> PipelineLayout<T>
    where
        T: TypedPipelineLayout,
    {
        PipelineLayout::typed(self)
    }

    pub fn create_bind_group<T, R>(&self, layout: &BindGroupLayout<T>, resources: R) -> BindGroup<T>
    where
        T: TypedBindGroupLayout,
        R: Resources<Layout = T>,
    {
        BindGroup::new(self, layout, resources)
    }

    pub fn create_compute_pipeline<R>(
        &self,
        descriptor: &ComputePipelineDescriptor<R>,
    ) -> ComputePipeline<R> {
        ComputePipeline::new(self, descriptor)
    }

    pub fn create_render_pipeline<T, V, I, R>(
        &self,
        descriptor: &RenderPipelineDescriptor<T, V, I, R>,
    ) -> RenderPipeline<T, V, I, R> {
        RenderPipeline::new(self, descriptor)
    }

    pub fn create_sampler(&self, descriptor: &SamplerDescriptor) -> Sampler {
        Sampler::new(self, descriptor)
    }

    pub fn create_anisotropic_sampler(&self, descriptor: &AnisotropicSamplerDescriptor) -> Sampler {
        Sampler::anisotropic(self, descriptor)
    }

    pub fn create_comparison_sampler(
        &self,
        descriptor: &ComparisonSamplerDescriptor,
    ) -> ComparisonSampler {
        ComparisonSampler::new(self, descriptor)
    }

    pub fn create_non_filtering_sampler(
        &self,
        descriptor: &NonFilteringSamplerDescriptor,
    ) -> NonFilteringSampler {
        NonFilteringSampler::new(self, descriptor)
    }

    pub fn create_texture_1d<F, U, V>(&self, size: u32) -> Texture1D<F, U, V>
    where
        F: Texture1DFormat,
        U: texture::UsageFlags,
        V: ViewFormats<F>,
    {
        Texture1D::new(self, size)
    }

    pub fn create_texture_2d<F, U, V>(&self, descriptor: &Texture2DDescriptor) -> Texture2D<F, U, V>
    where
        F: Texture2DFormat,
        U: texture::UsageFlags,
        V: ViewFormats<F>,
    {
        Texture2D::new(self, descriptor)
    }

    pub fn create_texture_3d<F, U, V>(&self, descriptor: &Texture3DDescriptor) -> Texture3D<F, U, V>
    where
        F: Texture3DFormat,
        U: texture::UsageFlags,
        V: ViewFormats<F>,
    {
        Texture3D::new(self, descriptor)
    }

    pub fn create_texture_multisampled_2d<F, U, const SAMPLES: u8>(
        &self,
        descriptor: &TextureMultisampled2DDescriptor,
    ) -> TextureMultisampled2D<F, U, SAMPLES>
    where
        F: MultisampleFormat,
        U: texture::UsageFlags + texture::RenderAttachment,
    {
        TextureMultisampled2D::new(self, descriptor)
    }

    pub fn create_occlusion_query_set(&self, len: u32) -> OcclusionQuerySet {
        OcclusionQuerySet::new(self, len)
    }

    pub fn create_command_encoder(&self) -> CommandEncoder {
        CommandEncoder::new(self)
    }

    pub fn queue(&self) -> Queue {
        Queue {
            inner: self.inner.queue(),
        }
    }
}

pub struct Queue {
    inner: GpuQueue,
}

impl Queue {
    pub fn submit(&self, command_buffer: CommandBuffer) {
        self.inner.submit(command_buffer.as_web_sys().as_ref());
    }
}
