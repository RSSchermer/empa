use std::future::Future;
use std::mem::MaybeUninit;
use std::{mem, slice};

use atomic_counter::RelaxedCounter;
use lazy_static::lazy_static;
use web_sys::{GpuDevice, GpuQueue};

use crate::adapter::{Features, Limits};
use crate::buffer::{AsBuffer, Buffer};
use crate::command::{
    CommandBuffer, CommandEncoder, RenderBundleEncoder, RenderBundleEncoderDescriptor,
};
use crate::compute_pipeline::{ComputePipeline, ComputePipelineDescriptor};
use crate::query::{OcclusionQuerySet, TimestampQuerySet};
use crate::render_pipeline::{RenderPipeline, RenderPipelineDescriptor};
use crate::resource_binding::{
    BindGroup, BindGroupLayout, BindGroupLayoutEntry, BindGroupLayouts, PipelineLayout, Resources,
    TypedBindGroupLayout,
};
use crate::sampler::{
    AnisotropicSamplerDescriptor, ComparisonSampler, ComparisonSamplerDescriptor,
    NonFilteringSampler, NonFilteringSamplerDescriptor, Sampler, SamplerDescriptor,
};
use crate::shader_module::{ShaderModule, ShaderSource};
use crate::texture::format::{
    ImageData, MultisampleFormat, Texture1DFormat, Texture2DFormat, Texture3DFormat, TextureFormat,
    ViewFormats,
};
use crate::texture::{
    ImageCopySize3D, ImageDataByteLayout, ImageDataLayout, Texture1D, Texture1DDescriptor,
    Texture2D, Texture2DDescriptor, Texture3D, Texture3DDescriptor, TextureMultisampled2D,
    TextureMultisampled2DDescriptor,
};
use crate::{buffer, texture};

lazy_static! {
    pub(crate) static ref ID_GEN: RelaxedCounter = RelaxedCounter::new(1);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct DeviceDescriptor {
    pub required_features: Features,
    pub required_limits: Limits,
}

#[derive(Clone)]
pub struct Device {
    pub(crate) inner: GpuDevice,
}

impl Device {
    #[doc(hidden)]
    pub fn as_web_sys(&self) -> &GpuDevice {
        &self.inner
    }

    pub fn create_buffer<D, T, U>(&self, data: D, usage: U) -> Buffer<T, U>
    where
        D: AsBuffer<T>,
        T: ?Sized,
        U: buffer::ValidUsageFlags,
    {
        data.as_buffer(self, false, usage)
    }

    pub fn create_buffer_mapped<D, T, U>(&self, data: D, usage: U) -> Buffer<T, U>
    where
        D: AsBuffer<T>,
        U: buffer::ValidUsageFlags,
    {
        data.as_buffer(self, true, usage)
    }

    pub fn create_buffer_uninit<T, U>(&self, usage: U) -> Buffer<MaybeUninit<T>, U>
    where
        U: buffer::ValidUsageFlags,
    {
        Buffer::create_uninit(self, false, usage)
    }

    pub fn create_buffer_uninit_mapped<T, U>(&self, usage: U) -> Buffer<MaybeUninit<T>, U>
    where
        U: buffer::ValidUsageFlags,
    {
        Buffer::create_uninit(self, true, usage)
    }

    #[cfg(feature = "bytemuck")]
    pub fn create_buffer_zeroed<T, U>(&self, usage: U) -> Buffer<T, U>
    where
        T: bytemuck::Zeroable,
        U: buffer::ValidUsageFlags,
    {
        unsafe { Buffer::create_uninit(self, false, usage).assume_init() }
    }

    #[cfg(feature = "bytemuck")]
    pub fn create_buffer_zeroed_mapped<T, U>(&self, usage: U) -> Buffer<T, U>
    where
        T: bytemuck::Zeroable,
        U: buffer::ValidUsageFlags,
    {
        unsafe { Buffer::create_uninit(self, true, usage).assume_init() }
    }

    pub fn create_slice_buffer_uninit<T, U>(
        &self,
        len: usize,
        usage: U,
    ) -> Buffer<[MaybeUninit<T>], U>
    where
        U: buffer::ValidUsageFlags,
    {
        Buffer::create_slice_uninit(self, len, false, usage)
    }

    pub fn create_slice_buffer_uninit_mapped<T, U>(
        &self,
        len: usize,
        usage: U,
    ) -> Buffer<[MaybeUninit<T>], U>
    where
        U: buffer::ValidUsageFlags,
    {
        Buffer::create_slice_uninit(self, len, true, usage)
    }

    #[cfg(feature = "bytemuck")]
    pub fn create_slice_buffer_zeroed<T, U>(&self, len: usize, usage: U) -> Buffer<[T], U>
    where
        T: bytemuck::Zeroable,
        U: buffer::ValidUsageFlags,
    {
        unsafe { Buffer::create_slice_uninit(self, len, false, usage).assume_init() }
    }

    #[cfg(feature = "bytemuck")]
    pub fn create_slice_buffer_zeroed_mapped<T, U>(&self, len: usize, usage: U) -> Buffer<[T], U>
    where
        T: bytemuck::Zeroable,
        U: buffer::ValidUsageFlags,
    {
        unsafe { Buffer::create_slice_uninit(self, len, true, usage).assume_init() }
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

    pub fn create_pipeline_layout<B>(
        &self,
        bind_group_layouts: B,
    ) -> PipelineLayout<B::PipelineLayout>
    where
        B: BindGroupLayouts,
    {
        PipelineLayout::typed(self, bind_group_layouts)
    }

    pub fn create_bind_group<T, R>(&self, layout: &BindGroupLayout<T>, resources: R) -> BindGroup<T>
    where
        T: TypedBindGroupLayout,
        R: Resources<Layout = T>,
    {
        BindGroup::new(self, layout, resources)
    }

    pub fn create_shader_module(&self, source: &ShaderSource) -> ShaderModule {
        ShaderModule::new(self, source)
    }

    pub fn create_compute_pipeline<R>(
        &self,
        descriptor: &ComputePipelineDescriptor<R>,
    ) -> impl Future<Output = ComputePipeline<R>> {
        ComputePipeline::new_async(self, descriptor)
    }

    pub fn create_compute_pipeline_sync<R>(
        &self,
        descriptor: &ComputePipelineDescriptor<R>,
    ) -> ComputePipeline<R> {
        ComputePipeline::new(self, descriptor)
    }

    pub fn create_render_pipeline<T, V, I, R>(
        &self,
        descriptor: &RenderPipelineDescriptor<T, V, I, R>,
    ) -> impl Future<Output = RenderPipeline<T, V, I, R>> {
        RenderPipeline::new_async(self, descriptor)
    }

    pub fn create_render_pipeline_sync<T, V, I, R>(
        &self,
        descriptor: &RenderPipelineDescriptor<T, V, I, R>,
    ) -> RenderPipeline<T, V, I, R> {
        RenderPipeline::new_sync(self, descriptor)
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

    pub fn create_texture_1d<F, U, V>(
        &self,
        descriptor: &Texture1DDescriptor<F, U, V>,
    ) -> Texture1D<F, U>
    where
        F: Texture1DFormat,
        U: texture::UsageFlags,
        V: ViewFormats<F>,
    {
        Texture1D::new(self, descriptor)
    }

    pub fn create_texture_2d<F, U, V>(
        &self,
        descriptor: &Texture2DDescriptor<F, U, V>,
    ) -> Texture2D<F, U>
    where
        F: Texture2DFormat,
        U: texture::UsageFlags,
        V: ViewFormats<F>,
    {
        Texture2D::new(self, descriptor)
    }

    pub fn create_texture_3d<F, U, V>(
        &self,
        descriptor: &Texture3DDescriptor<F, U, V>,
    ) -> Texture3D<F, U>
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

    pub fn create_timestamp_query_set(&self, len: u32) -> TimestampQuerySet {
        TimestampQuerySet::new(self, len)
    }

    pub fn create_command_encoder(&self) -> CommandEncoder {
        CommandEncoder::new(self)
    }

    pub fn create_render_bundle_encoder<T>(
        &self,
        descriptor: &RenderBundleEncoderDescriptor<T>,
    ) -> RenderBundleEncoder<T, (), (), (), ()> {
        RenderBundleEncoder::new(self, descriptor)
    }

    pub fn queue(&self) -> Queue {
        Queue {
            inner: self.inner.queue(),
        }
    }
}

pub struct Queue {
    pub(crate) inner: GpuQueue,
}

impl Queue {
    pub fn submit(&self, command_buffer: CommandBuffer) {
        let array = js_sys::Array::new();

        array.push(command_buffer.as_web_sys().as_ref());

        self.inner.submit(array.as_ref());
    }

    pub fn write_buffer<T, U>(&self, dst: buffer::View<T, U>, data: &T)
    where
        T: Copy + 'static,
        U: buffer::CopyDst,
    {
        let ptr = data as *const T as *const u8;
        let len = mem::size_of::<T>();

        let bytes = unsafe { slice::from_raw_parts(ptr, len) };

        self.inner.write_buffer_with_u32_and_u8_array(
            dst.as_web_sys(),
            dst.offset_in_bytes() as u32,
            bytes,
        );
    }

    pub fn write_buffer_slice<T, U>(&self, dst: buffer::View<[T], U>, data: &[T])
    where
        T: Copy + 'static,
        U: buffer::CopyDst,
    {
        assert_eq!(
            dst.len(),
            data.len(),
            "the size of the buffer view `len` does not match the size of the data"
        );

        let ptr = data as *const [T] as *const u8;
        let len = mem::size_of::<T>() * data.len();

        let bytes = unsafe { slice::from_raw_parts(ptr, len) };

        self.inner.write_buffer_with_u32_and_u8_array(
            dst.as_web_sys(),
            dst.offset_in_bytes() as u32,
            bytes,
        );
    }

    fn write_texture_internal<F, T>(
        &self,
        dst: &texture::ImageCopyTexture<F>,
        data: &[T],
        layout: ImageDataByteLayout,
        size: ImageCopySize3D,
    ) {
        let ImageCopySize3D {
            width,
            height,
            depth_or_layers,
        } = size;

        let [block_width, block_height] = dst.block_size;

        let width_in_blocks = width / block_width;

        assert!(
            layout.blocks_per_row >= width_in_blocks,
            "blocks per row must be at least the copy width in blocks (`{}`)",
            width_in_blocks
        );

        let height_in_blocks = height / block_height;

        assert!(
            layout.rows_per_image >= height_in_blocks,
            "rows per image must be at least the copy height in blocks (`{}`)",
            height_in_blocks
        );

        let min_size = layout.blocks_per_row * layout.rows_per_image * depth_or_layers;

        assert!(
            data.len() >= min_size as usize,
            "data slice must contains enough elements for the copy size (`{}` blocks)",
            min_size
        );

        let ptr = data as *const [T] as *const u8;
        let len = mem::size_of::<T>() * data.len();

        let bytes = unsafe { slice::from_raw_parts(ptr, len) };

        self.inner
            .write_texture_with_u8_array_and_gpu_extent_3d_dict(
                &dst.to_web_sys(),
                bytes,
                &layout.to_web_sys(),
                &size.to_web_sys(),
            );
    }

    pub fn write_texture<F, T>(
        &self,
        dst: &texture::ImageCopyDst<F>,
        data: &[T],
        layout: ImageDataLayout,
    ) where
        T: ImageData<F>,
        F: TextureFormat,
    {
        let size = ImageCopySize3D {
            width: dst.inner.width,
            height: dst.inner.height,
            depth_or_layers: dst.inner.depth_or_layers,
        };
        let byte_layout = layout.to_byte_layout(mem::size_of::<T>() as u32);

        self.write_texture_internal(&dst.inner, data, byte_layout, size);
    }

    pub fn write_texture_sub_image<F, T>(
        &self,
        dst: &texture::SubImageCopyDst<F>,
        data: &[T],
        layout: ImageDataLayout,
        size: ImageCopySize3D,
    ) where
        T: ImageData<F>,
        F: TextureFormat,
    {
        size.validate_with_block_size(dst.inner.block_size);
        dst.inner.validate_dst_with_size(size);

        let byte_layout = layout.to_byte_layout(mem::size_of::<T>() as u32);

        self.write_texture_internal(&dst.inner, data, byte_layout, size);
    }

    fn write_texture_raw_internal<F>(
        &self,
        dst: &texture::ImageCopyTexture<F>,
        bytes: &[u8],
        layout: ImageDataByteLayout,
        size: ImageCopySize3D,
    ) {
        assert!(
            layout.bytes_per_block == dst.bytes_per_block,
            "`layout` bytes per block does not match `dst` bytes per block"
        );

        let ImageCopySize3D {
            width,
            height,
            depth_or_layers,
        } = size;

        let [block_width, block_height] = dst.block_size;

        let width_in_blocks = width / block_width;

        assert!(
            layout.blocks_per_row >= width_in_blocks,
            "blocks per row must be at least the copy width in blocks (`{}`)",
            width_in_blocks
        );

        let height_in_blocks = height / block_height;

        assert!(
            layout.rows_per_image >= height_in_blocks,
            "rows per image must be at least the copy height in blocks (`{}`)",
            height_in_blocks
        );

        let min_size = layout.bytes_per_block
            * layout.blocks_per_row
            * layout.rows_per_image
            * depth_or_layers;

        assert!(
            bytes.len() >= min_size as usize,
            "data slice must contains enough elements for the copy size (`{}` blocks)",
            min_size
        );

        self.inner
            .write_texture_with_u8_array_and_gpu_extent_3d_dict(
                &dst.to_web_sys(),
                bytes,
                &layout.to_web_sys(),
                &size.to_web_sys(),
            );
    }

    pub fn write_texture_raw<F>(
        &self,
        dst: &texture::ImageCopyDst<F>,
        bytes: &[u8],
        layout: ImageDataByteLayout,
    ) where
        F: TextureFormat,
    {
        let size = ImageCopySize3D {
            width: dst.inner.width,
            height: dst.inner.height,
            depth_or_layers: dst.inner.depth_or_layers,
        };

        self.write_texture_raw_internal(&dst.inner, bytes, layout, size);
    }

    pub fn write_texture_sub_image_raw<F>(
        &self,
        dst: &texture::SubImageCopyDst<F>,
        bytes: &[u8],
        layout: ImageDataByteLayout,
        size: ImageCopySize3D,
    ) where
        F: TextureFormat,
    {
        size.validate_with_block_size(dst.inner.block_size);
        dst.inner.validate_dst_with_size(size);

        self.write_texture_raw_internal(&dst.inner, bytes, layout, size);
    }
}
