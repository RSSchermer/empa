use std::borrow::Borrow;
use std::ops::Rem;
use std::sync::Arc;
use std::{marker, mem};

use staticvec::StaticVec;
use wasm_bindgen::JsValue;
use web_sys::{
    GpuColorDict, GpuCommandBuffer, GpuCommandEncoder, GpuComputePassEncoder, GpuExtent3dDict,
    GpuRenderBundle, GpuRenderBundleEncoder, GpuRenderBundleEncoderDescriptor,
    GpuRenderPassDescriptor, GpuRenderPassEncoder,
};

use crate::abi::{MemoryUnit, MemoryUnitLayout};
use crate::buffer::BufferHandle;
use crate::command::{
    BindGroupEncoding, BindGroups, IndexBuffer, IndexBufferEncoding, VertexBufferEncoding,
    VertexBuffers,
};
use crate::compute_pipeline::ComputePipeline;
use crate::device::Device;
use crate::query::{OcclusionQuerySet, QuerySetHandle, TimestampQuerySet};
use crate::render_pipeline::{PipelineIndexFormat, PipelineIndexFormatCompatible, RenderPipeline};
use crate::render_target::{
    MultisampleRenderLayout, ReadOnly, RenderLayout, RenderLayoutCompatible, RenderTargetEncoding,
    TypedColorLayout, TypedMultisampleColorLayout, ValidRenderTarget,
};
use crate::resource_binding::BindGroupResource;
use crate::texture::format::{DepthStencilRenderable, ImageData, TextureFormat};
use crate::texture::{ImageCopySize3D, TextureHandle};
use crate::type_flag::{TypeFlag, O, X};
use crate::{abi, buffer, texture};

enum ResourceHandle {
    Buffer(Arc<BufferHandle>),
    Texture(Arc<TextureHandle>),
    BindGroup(Arc<Vec<BindGroupResource>>),
    RenderTarget(Arc<StaticVec<Arc<TextureHandle>, 9>>),
    RenderBundle(Arc<Vec<ResourceHandle>>),
    QuerySet(Arc<QuerySetHandle>),
}

impl From<Arc<BufferHandle>> for ResourceHandle {
    fn from(resource_handle: Arc<BufferHandle>) -> Self {
        ResourceHandle::Buffer(resource_handle)
    }
}

impl From<Arc<TextureHandle>> for ResourceHandle {
    fn from(resource_handle: Arc<TextureHandle>) -> Self {
        ResourceHandle::Texture(resource_handle)
    }
}

impl From<Arc<Vec<BindGroupResource>>> for ResourceHandle {
    fn from(bind_group_resources: Arc<Vec<BindGroupResource>>) -> Self {
        ResourceHandle::BindGroup(bind_group_resources)
    }
}

impl From<Arc<StaticVec<Arc<TextureHandle>, 9_usize>>> for ResourceHandle {
    fn from(render_target_resources: Arc<StaticVec<Arc<TextureHandle>, 9>>) -> Self {
        ResourceHandle::RenderTarget(render_target_resources)
    }
}

impl From<Arc<Vec<ResourceHandle>>> for ResourceHandle {
    fn from(render_bundle_resources: Arc<Vec<ResourceHandle>>) -> Self {
        ResourceHandle::RenderBundle(render_bundle_resources)
    }
}

impl From<Arc<QuerySetHandle>> for ResourceHandle {
    fn from(resource_handle: Arc<QuerySetHandle>) -> Self {
        ResourceHandle::QuerySet(resource_handle)
    }
}

pub struct CommandBuffer {
    inner: GpuCommandBuffer,
    _resource_handles: Vec<ResourceHandle>,
}

impl CommandBuffer {
    pub(crate) fn as_web_sys(&self) -> &GpuCommandBuffer {
        &self.inner
    }
}

pub struct CommandEncoder {
    inner: GpuCommandEncoder,
    _resource_handles: Vec<ResourceHandle>,
}

impl CommandEncoder {
    pub(crate) fn new(device: &Device) -> Self {
        CommandEncoder {
            inner: device.inner.create_command_encoder(),
            _resource_handles: Vec::new(),
        }
    }

    pub fn clear_buffer<T, U>(mut self, buffer: buffer::View<T, U>) -> CommandEncoder
    where
        U: buffer::CopyDst + 'static,
    {
        let size = buffer.size_in_bytes() as u32;
        let offset = buffer.offset_in_bytes() as u32;

        assert!(
            size.rem(4) == 0,
            "cleared region's size in bytes must be a multiple of `4`"
        );
        assert!(
            offset.rem(4) == 0,
            "cleared region's offset in bytes must be a multiple of `8`"
        );

        self.inner
            .clear_buffer_with_u32_and_u32(buffer.as_web_sys(), offset, size);

        self._resource_handles
            .push(buffer.buffer.inner.clone().into());

        self
    }

    pub fn clear_buffer_slice<T, U>(mut self, buffer: buffer::View<[T], U>) -> CommandEncoder
    where
        U: buffer::CopyDst + 'static,
    {
        let size = buffer.size_in_bytes() as u32;
        let offset = buffer.offset_in_bytes() as u32;

        assert!(
            size.rem(4) == 0,
            "cleared region's size in bytes must be a multiple of `4`"
        );
        assert!(
            offset.rem(4) == 0,
            "cleared region's offset in bytes must be a multiple of `8`"
        );

        self.inner
            .clear_buffer_with_u32_and_u32(buffer.as_web_sys(), offset, size);

        self._resource_handles
            .push(buffer.buffer.inner.clone().into());

        self
    }

    pub fn copy_buffer_to_buffer<T, U0, U1>(
        mut self,
        src: buffer::View<T, U0>,
        dst: buffer::View<T, U1>,
    ) -> CommandEncoder
    where
        U0: buffer::CopySrc + 'static,
        U1: buffer::CopyDst + 'static,
        T: 'static,
    {
        let src_offset = src.offset_in_bytes() as u32;
        let dst_offset = dst.offset_in_bytes() as u32;
        let size = mem::size_of::<T>();

        assert!(
            size.rem(4) == 0,
            "copied size in bytes must be a multiple of `4`"
        );

        // This may be redundant, because the offset must be sized aligned anyway?
        assert!(
            src_offset.rem(4) == 0,
            "`src` view's offset in bytes must be a multiple of `8`"
        );
        assert!(
            dst_offset.rem(4) == 0,
            "`dst` view's offset in bytes must be a multiple of `8`"
        );

        self.inner.copy_buffer_to_buffer_with_u32_and_u32_and_u32(
            src.as_web_sys(),
            src_offset,
            dst.as_web_sys(),
            dst_offset,
            size as u32,
        );

        self._resource_handles.push(src.buffer.inner.clone().into());
        self._resource_handles.push(dst.buffer.inner.clone().into());

        self
    }

    pub fn copy_buffer_to_buffer_slice<T, U0, U1>(
        mut self,
        src: buffer::View<[T], U0>,
        dst: buffer::View<[T], U1>,
    ) -> CommandEncoder
    where
        U0: buffer::CopySrc + 'static,
        U1: buffer::CopyDst + 'static,
        T: 'static,
    {
        assert!(
            src.len() == dst.len(),
            "source and destination have different lengths"
        );

        let src_offset = src.offset_in_bytes() as u32;
        let dst_offset = dst.offset_in_bytes() as u32;

        debug_assert!(src.size_in_bytes() == dst.size_in_bytes());

        let size = src.size_in_bytes();

        assert!(
            size.rem(4) == 0,
            "copied size in bytes must be a multiple of `4`"
        );
        assert!(
            src_offset.rem(4) == 0,
            "`src` view's offset in bytes must be a multiple of `8`"
        );
        assert!(
            dst_offset.rem(4) == 0,
            "`dst` view's offset in bytes must be a multiple of `8`"
        );

        self.inner.copy_buffer_to_buffer_with_u32_and_u32_and_u32(
            src.as_web_sys(),
            src_offset,
            dst.as_web_sys(),
            dst_offset,
            size as u32,
        );

        self._resource_handles.push(src.buffer.inner.clone().into());
        self._resource_handles.push(dst.buffer.inner.clone().into());

        self
    }

    fn image_copy_buffer_to_texture_internal<F>(
        mut self,
        src: &buffer::ImageCopyBuffer,
        dst: &texture::ImageCopyDst<F>,
    ) -> Self {
        let width = dst.inner.width;
        let height = dst.inner.height;
        let depth_or_layers = dst.inner.depth_or_layers;

        src.validate_with_size_and_block_size(
            ImageCopySize3D {
                width,
                height,
                depth_or_layers,
            },
            dst.inner.block_size,
        );

        let mut extent = GpuExtent3dDict::new(width);

        extent.height(height);
        extent.depth_or_array_layers(depth_or_layers);

        self.inner.copy_buffer_to_texture_with_gpu_extent_3d_dict(
            &src.to_web_sys(),
            &dst.inner.to_web_sys(),
            &extent,
        );

        self._resource_handles.push(src.buffer.clone().into());
        self._resource_handles
            .push(dst.inner.texture.clone().into());

        self
    }

    pub fn image_copy_buffer_to_texture<T, F>(
        self,
        src: &buffer::ImageCopySrc<T>,
        dst: &texture::ImageCopyDst<F>,
    ) -> Self
    where
        T: ImageData<F>,
        F: TextureFormat,
    {
        self.image_copy_buffer_to_texture_internal(&src.inner, dst)
    }

    pub fn image_copy_buffer_to_texture_raw<F>(
        self,
        src: &buffer::ImageCopySrcRaw,
        dst: &texture::ImageCopyDst<F>,
    ) -> Self {
        assert!(
            src.inner.bytes_per_block == dst.inner.bytes_per_block,
            "`src` bytes per block does not match `dst` bytes per block"
        );

        self.image_copy_buffer_to_texture_internal(&src.inner, dst)
    }

    fn sub_image_copy_buffer_to_texture_internal<F>(
        mut self,
        src: &buffer::ImageCopyBuffer,
        dst: &texture::SubImageCopyDst<F>,
        size: ImageCopySize3D,
    ) -> Self {
        size.validate_with_block_size(dst.inner.block_size);
        src.validate_with_size_and_block_size(size, dst.inner.block_size);
        dst.inner.validate_dst_with_size(size);

        self.inner.copy_buffer_to_texture_with_gpu_extent_3d_dict(
            &src.to_web_sys(),
            &dst.inner.to_web_sys(),
            &size.to_web_sys(),
        );

        self._resource_handles.push(src.buffer.clone().into());
        self._resource_handles
            .push(dst.inner.texture.clone().into());

        self
    }

    pub fn sub_image_copy_buffer_to_texture<T, F>(
        self,
        src: &buffer::ImageCopySrc<T>,
        dst: &texture::SubImageCopyDst<F>,
        size: ImageCopySize3D,
    ) -> Self
    where
        T: ImageData<F>,
        F: TextureFormat,
    {
        self.sub_image_copy_buffer_to_texture_internal(&src.inner, dst, size)
    }

    pub fn sub_image_copy_buffer_to_texture_raw<F>(
        self,
        src: &buffer::ImageCopySrcRaw,
        dst: &texture::SubImageCopyDst<F>,
        size: ImageCopySize3D,
    ) -> Self {
        assert!(
            src.inner.bytes_per_block == dst.inner.bytes_per_block,
            "`src` bytes per block does not match `dst` bytes per block"
        );

        self.sub_image_copy_buffer_to_texture_internal(&src.inner, dst, size)
    }

    fn image_copy_texture_to_buffer_internal<F>(
        mut self,
        src: &texture::ImageCopySrc<F>,
        dst: &buffer::ImageCopyBuffer,
    ) -> Self {
        let width = src.inner.width;
        let height = src.inner.height;
        let depth_or_layers = src.inner.depth_or_layers;

        dst.validate_with_size_and_block_size(
            ImageCopySize3D {
                width,
                height,
                depth_or_layers,
            },
            src.inner.block_size,
        );

        let mut extent = GpuExtent3dDict::new(width);

        extent.height(height);
        extent.depth_or_array_layers(depth_or_layers);

        self.inner.copy_texture_to_buffer_with_gpu_extent_3d_dict(
            &src.inner.to_web_sys(),
            &dst.to_web_sys(),
            &extent,
        );

        self._resource_handles
            .push(src.inner.texture.clone().into());
        self._resource_handles.push(dst.buffer.clone().into());

        self
    }

    pub fn image_copy_texture_to_buffer<F, T>(
        self,
        src: &texture::ImageCopySrc<F>,
        dst: &buffer::ImageCopyDst<T>,
    ) -> Self
    where
        F: TextureFormat,
        T: ImageData<F>,
    {
        self.image_copy_texture_to_buffer_internal(src, &dst.inner)
    }

    pub fn image_copy_texture_to_buffer_raw<F>(
        self,
        src: &texture::ImageCopySrc<F>,
        dst: &buffer::ImageCopyDstRaw,
    ) -> Self {
        assert!(
            src.inner.bytes_per_block == dst.inner.bytes_per_block,
            "`src` bytes per block does not match `dst` bytes per block"
        );

        self.image_copy_texture_to_buffer_internal(src, &dst.inner)
    }

    fn sub_image_copy_texture_to_buffer_internal<F>(
        mut self,
        src: &texture::SubImageCopySrc<F>,
        dst: &buffer::ImageCopyBuffer,
        size: ImageCopySize3D,
    ) -> Self {
        size.validate_with_block_size(src.inner.block_size);
        src.inner.validate_src_with_size(size);
        dst.validate_with_size_and_block_size(size, src.inner.block_size);

        self.inner.copy_texture_to_buffer_with_gpu_extent_3d_dict(
            &src.inner.to_web_sys(),
            &dst.to_web_sys(),
            &size.to_web_sys(),
        );

        self._resource_handles
            .push(src.inner.texture.clone().into());
        self._resource_handles.push(dst.buffer.clone().into());

        self
    }

    pub fn sub_image_copy_texture_to_buffer<F, T>(
        self,
        src: &texture::SubImageCopySrc<F>,
        dst: &buffer::ImageCopyDst<T>,
        size: ImageCopySize3D,
    ) -> Self
    where
        F: TextureFormat,
        T: ImageData<F>,
    {
        self.sub_image_copy_texture_to_buffer_internal(src, &dst.inner, size)
    }

    pub fn sub_image_copy_texture_to_buffer_raw<F>(
        self,
        src: &texture::SubImageCopySrc<F>,
        dst: &buffer::ImageCopyDstRaw,
        size: ImageCopySize3D,
    ) -> Self {
        assert!(
            src.inner.bytes_per_block == dst.inner.bytes_per_block,
            "`src` bytes per block does not match `dst` bytes per block"
        );

        self.sub_image_copy_texture_to_buffer_internal(src, &dst.inner, size)
    }

    pub fn image_copy_texture_to_texture<F>(
        mut self,
        src: &texture::ImageCopyToTextureSrc<F>,
        dst: &texture::ImageCopyFromTextureDst<F>,
    ) -> Self {
        assert!(
            src.inner.width == dst.inner.width,
            "`src` and `dst` widths must match"
        );
        assert!(
            src.inner.height == dst.inner.height,
            "`src` and `dst` heights must match"
        );
        assert!(
            src.inner.depth_or_layers == dst.inner.depth_or_layers,
            "`src` and `dst` depth/layers must match"
        );

        let mut extent = GpuExtent3dDict::new(src.inner.width);

        extent.height(src.inner.height);
        extent.depth_or_array_layers(src.inner.depth_or_layers);

        self.inner.copy_texture_to_texture_with_gpu_extent_3d_dict(
            &src.inner.to_web_sys(),
            &dst.inner.to_web_sys(),
            &extent,
        );

        self._resource_handles
            .push(src.inner.texture.clone().into());
        self._resource_handles
            .push(dst.inner.texture.clone().into());

        self
    }

    pub fn sub_image_copy_texture_to_texture<F>(
        self,
        src: &texture::SubImageCopyToTextureSrc<F>,
        dst: &texture::SubImageCopyFromTextureDst<F>,
        size: ImageCopySize3D,
    ) -> Self {
        size.validate_with_block_size(src.inner.block_size);
        src.inner.validate_src_with_size(size);
        dst.inner.validate_dst_with_size(size);

        self.inner.copy_texture_to_texture_with_gpu_extent_3d_dict(
            &src.inner.to_web_sys(),
            &dst.inner.to_web_sys(),
            &size.to_web_sys(),
        );

        self
    }

    pub fn image_copy_texture_to_texture_multisample<F, const SAMPLES: u8>(
        mut self,
        src: &texture::ImageCopyToTextureSrcMultisample<F, SAMPLES>,
        dst: &texture::ImageCopyToTextureDstMultisample<F, SAMPLES>,
    ) -> Self {
        assert!(
            src.inner.width == dst.inner.width,
            "`src` and `dst` widths must match"
        );
        assert!(
            src.inner.height == dst.inner.height,
            "`src` and `dst` heights must match"
        );

        let mut extent = GpuExtent3dDict::new(src.inner.width);

        extent.height(src.inner.height);

        self.inner.copy_texture_to_texture_with_gpu_extent_3d_dict(
            &src.inner.to_web_sys(),
            &dst.inner.to_web_sys(),
            &extent,
        );

        self._resource_handles
            .push(src.inner.texture.clone().into());
        self._resource_handles
            .push(dst.inner.texture.clone().into());

        self
    }

    pub fn begin_compute_pass(self) -> ComputePassEncoder<(), ()> {
        let inner = self.inner.begin_compute_pass();

        ComputePassEncoder {
            inner,
            command_encoder: self,
            current_pipeline_id: None,
            current_bind_group_ids: [None; 4],
            _marker: Default::default(),
        }
    }

    pub fn begin_render_pass<T, Q>(
        mut self,
        descriptor: &RenderPassDescriptor<T, Q>,
    ) -> ClearRenderPassEncoder<T, Q> {
        let inner = self.inner.begin_render_pass(&descriptor.inner);

        self._resource_handles
            .push(descriptor._texture_handles.clone().into());

        if let Some(query_set_handle) = &descriptor._occlusion_query_set_handle {
            self._resource_handles.push(query_set_handle.clone().into());
        }

        RenderPassEncoder {
            inner,
            command_encoder: self,
            current_pipeline_id: None,
            current_vertex_buffers: [None; 8],
            current_index_buffer: None,
            current_bind_group_ids: [None; 4],
            _marker: Default::default(),
        }
    }

    pub fn write_timestamp(mut self, query_set: &TimestampQuerySet, index: u32) -> Self {
        assert!(index < query_set.len(), "index out of bounds");

        self.inner.write_timestamp(query_set.as_web_sys(), index);

        self._resource_handles.push(query_set.inner.clone().into());

        self
    }

    pub fn resolve_occlusion_query_set<U>(
        mut self,
        query_set: &OcclusionQuerySet,
        offset: u32,
        view: buffer::View<[u64], U>,
    ) -> Self
    where
        U: buffer::QueryResolve,
    {
        assert!(
            offset + view.len() as u32 <= query_set.len(),
            "resolve range out of bounds"
        );

        self.inner.resolve_query_set_with_u32(
            query_set.as_web_sys(),
            offset,
            view.len() as u32,
            view.as_web_sys(),
            view.offset_in_bytes() as u32,
        );

        self._resource_handles.push(query_set.inner.clone().into());
        self._resource_handles
            .push(view.buffer.inner.clone().into());

        self
    }

    pub fn resolve_timestamp_query_set<U>(
        mut self,
        query_set: &TimestampQuerySet,
        offset: u32,
        view: buffer::View<[u64], U>,
    ) -> Self
    where
        U: buffer::QueryResolve,
    {
        assert!(
            offset + view.len() as u32 <= query_set.len(),
            "resolve range out of bounds"
        );

        self.inner.resolve_query_set_with_u32(
            query_set.as_web_sys(),
            offset,
            view.len() as u32,
            view.as_web_sys(),
            view.offset_in_bytes() as u32,
        );

        self._resource_handles.push(query_set.inner.clone().into());
        self._resource_handles
            .push(view.buffer.inner.clone().into());

        self
    }

    pub fn finish(self) -> CommandBuffer {
        let CommandEncoder {
            inner,
            _resource_handles,
        } = self;

        CommandBuffer {
            inner: inner.finish(),
            _resource_handles,
        }
    }
}

mod resource_binding_command_encoder_seal {
    pub trait Seal {}
}

pub trait ResourceBindingCommandEncoder {
    type WithResources<RNew>;

    fn set_bind_groups<RNew>(self, bind_groups: RNew) -> Self::WithResources<RNew>
    where
        RNew: BindGroups;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct DispatchWorkgroups {
    pub count_x: u32,
    pub count_y: u32,
    pub count_z: u32,
}

unsafe impl abi::Sized for DispatchWorkgroups {
    const LAYOUT: &'static [MemoryUnit] = &[
        MemoryUnit {
            offset: 0,
            layout: MemoryUnitLayout::UnsignedInteger,
        },
        MemoryUnit {
            offset: 4,
            layout: MemoryUnitLayout::UnsignedInteger,
        },
        MemoryUnit {
            offset: 8,
            layout: MemoryUnitLayout::UnsignedInteger,
        },
    ];
}

pub struct ComputePassEncoder<Pipeline, Resources> {
    inner: GpuComputePassEncoder,
    command_encoder: CommandEncoder,
    current_pipeline_id: Option<usize>,
    current_bind_group_ids: [Option<usize>; 4],
    _marker: marker::PhantomData<(*const Pipeline, *const Resources)>,
}

impl<P, R> resource_binding_command_encoder_seal::Seal for ComputePassEncoder<P, R> {}
impl<P, R> ResourceBindingCommandEncoder for ComputePassEncoder<P, R> {
    type WithResources<RNew> = ComputePassEncoder<P, RNew>;

    fn set_bind_groups<RNew>(self, bind_groups: RNew) -> Self::WithResources<RNew>
    where
        RNew: BindGroups,
    {
        let ComputePassEncoder {
            inner,
            mut command_encoder,
            current_pipeline_id,
            mut current_bind_group_ids,
            ..
        } = self;

        for (i, encoding) in bind_groups.encodings().enumerate() {
            let BindGroupEncoding {
                bind_group,
                id,
                _resource_handles,
            } = encoding;

            if current_bind_group_ids[i] != Some(id) {
                inner.set_bind_group(i as u32, Some(&bind_group));
                command_encoder
                    ._resource_handles
                    .push(_resource_handles.into());

                current_bind_group_ids[i] = Some(id);
            }
        }

        ComputePassEncoder {
            inner,
            command_encoder,
            current_pipeline_id,
            current_bind_group_ids,
            _marker: Default::default(),
        }
    }
}

impl<P, R> ComputePassEncoder<P, R> {
    pub fn set_pipeline<PR>(
        self,
        pipeline: &ComputePipeline<PR>,
    ) -> ComputePassEncoder<ComputePipeline<PR>, R> {
        let ComputePassEncoder {
            inner,
            command_encoder,
            current_pipeline_id,
            current_bind_group_ids,
            ..
        } = self;

        if Some(pipeline.id()) != current_pipeline_id {
            inner.set_pipeline(pipeline.as_web_sys());
        }

        ComputePassEncoder {
            inner,
            command_encoder,
            current_pipeline_id: Some(pipeline.id()),
            current_bind_group_ids,
            _marker: Default::default(),
        }
    }

    pub fn end(self) -> CommandEncoder {
        self.inner.end();

        self.command_encoder
    }
}

impl<RLayout, R> ComputePassEncoder<ComputePipeline<RLayout>, R>
where
    R: BindGroups<Layout = RLayout>,
{
    pub fn dispatch_workgroups(self, dispatch_workgroups: DispatchWorkgroups) -> Self {
        let DispatchWorkgroups {
            count_x,
            count_y,
            count_z,
        } = dispatch_workgroups;

        self.inner
            .dispatch_workgroups_with_workgroup_count_y_and_workgroup_count_z(
                count_x, count_y, count_z,
            );

        self
    }

    pub fn dispatch_workgroups_indirect<U>(self, view: buffer::View<DispatchWorkgroups, U>) -> Self
    where
        U: buffer::Indirect,
    {
        self.inner.dispatch_workgroups_indirect_with_u32(
            view.as_web_sys(),
            view.offset_in_bytes() as u32,
        );

        self
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct Draw {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub first_instance: u32,
}

unsafe impl abi::Sized for Draw {
    const LAYOUT: &'static [MemoryUnit] = &[
        MemoryUnit {
            offset: 0,
            layout: MemoryUnitLayout::UnsignedInteger,
        },
        MemoryUnit {
            offset: 4,
            layout: MemoryUnitLayout::UnsignedInteger,
        },
        MemoryUnit {
            offset: 8,
            layout: MemoryUnitLayout::UnsignedInteger,
        },
        MemoryUnit {
            offset: 12,
            layout: MemoryUnitLayout::UnsignedInteger,
        },
    ];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct DrawIndexed {
    pub index_count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub base_vertex: u32,
    pub first_instance: u32,
}

unsafe impl abi::Sized for DrawIndexed {
    const LAYOUT: &'static [MemoryUnit] = &[
        MemoryUnit {
            offset: 0,
            layout: MemoryUnitLayout::UnsignedInteger,
        },
        MemoryUnit {
            offset: 4,
            layout: MemoryUnitLayout::UnsignedInteger,
        },
        MemoryUnit {
            offset: 8,
            layout: MemoryUnitLayout::UnsignedInteger,
        },
        MemoryUnit {
            offset: 12,
            layout: MemoryUnitLayout::UnsignedInteger,
        },
        MemoryUnit {
            offset: 16,
            layout: MemoryUnitLayout::UnsignedInteger,
        },
    ];
}

mod render_state_encoder_seal {
    pub trait Seal {}
}

pub trait RenderStateEncoder<T>: render_state_encoder_seal::Seal {
    type WithPipeline<P>;

    type WithVertexBuffers<V>;

    type WithIndexBuffer<I>;

    fn set_pipeline<PT, PV, PI, PR>(
        self,
        pipeline: &RenderPipeline<PT, PV, PI, PR>,
    ) -> Self::WithPipeline<RenderPipeline<PT, PV, PI, PR>>
    where
        PT: RenderLayoutCompatible<T>;

    fn set_vertex_buffers<V>(self, vertex_buffers: V) -> Self::WithVertexBuffers<V>
    where
        V: VertexBuffers;

    fn set_index_buffer<I>(self, index_buffer: I) -> Self::WithIndexBuffer<I>
    where
        I: IndexBuffer;
}

mod draw_command_encoder_seal {
    pub trait Seal {}
}

pub trait DrawCommandEncoder: draw_command_encoder_seal::Seal {
    fn draw(self, draw: Draw) -> Self;

    fn draw_indirect<U>(self, view: buffer::View<Draw, U>) -> Self
    where
        U: buffer::Indirect;
}

mod draw_indexed_command_encoder_seal {
    pub trait Seal {}
}

pub trait DrawIndexedCommandEncoder: draw_indexed_command_encoder_seal::Seal {
    fn draw_indexed(self, draw_indexed: DrawIndexed) -> Self;

    fn draw_indexed_indirect<U>(self, view: buffer::View<DrawIndexed, U>) -> Self
    where
        U: buffer::Indirect;
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub max_depth: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ScissorRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BlendConstant {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct CurrentBufferRange {
    id: usize,
    offset: u32,
    size: u32,
}

pub struct OcclusionQueryState<T>
where
    T: TypeFlag,
{
    _marker: marker::PhantomData<T>,
}

mod begin_occlusion_query_seal {
    pub trait Seal {}
}

pub trait BeginOcclusionQuery: begin_occlusion_query_seal::Seal {}

impl begin_occlusion_query_seal::Seal for OcclusionQueryState<O> {}
impl BeginOcclusionQuery for OcclusionQueryState<O> {}

mod end_occlusion_query_seal {
    pub trait Seal {}
}

pub trait EndOcclusionQuery: end_occlusion_query_seal::Seal {}

impl end_occlusion_query_seal::Seal for OcclusionQueryState<X> {}
impl EndOcclusionQuery for OcclusionQueryState<X> {}

mod end_render_pass_seal {
    pub trait Seal {}
}

pub trait EndRenderPass: end_render_pass_seal::Seal {}

impl end_render_pass_seal::Seal for OcclusionQueryState<O> {}
impl EndRenderPass for OcclusionQueryState<O> {}

impl end_render_pass_seal::Seal for () {}
impl EndRenderPass for () {}

pub struct RenderPassDescriptor<RenderTarget, OcclusionQueryState> {
    inner: GpuRenderPassDescriptor,
    _texture_handles: Arc<StaticVec<Arc<TextureHandle>, 9>>,
    _occlusion_query_set_handle: Option<Arc<QuerySetHandle>>,
    _marker: marker::PhantomData<(*const RenderTarget, OcclusionQueryState)>,
}

impl RenderPassDescriptor<(), ()> {
    pub fn new<T: ValidRenderTarget>(
        render_target: &T,
    ) -> RenderPassDescriptor<T::RenderLayout, ()> {
        let RenderTargetEncoding {
            color_attachments,
            depth_stencil_attachment,
        } = render_target.encoding();

        let width;
        let height;

        if let Some(attachment) = &depth_stencil_attachment {
            width = attachment.width;
            height = attachment.height;
        } else {
            let first = color_attachments.first().expect(
                "target must specify either at least 1 color attachment or a depth-stencil \
                attachment",
            );

            width = first.width;
            height = first.height;
        }

        for attachment in color_attachments.iter() {
            if attachment.width != width || attachment.height != height {
                panic!("all attachment dimensions must match")
            }
        }

        if let Some(attachment) = &depth_stencil_attachment {
            if attachment.width != width || attachment.height != height {
                panic!("all attachment dimensions must match")
            }
        }

        let color_array = js_sys::Array::new();
        let mut texture_handles = StaticVec::new();

        for attachment in color_attachments {
            color_array.push(attachment.inner.as_ref());
            texture_handles.push(attachment._texture_handle);
        }

        let mut inner = GpuRenderPassDescriptor::new(&color_array);

        if let Some(depth_stencil_attachment) = depth_stencil_attachment {
            inner.depth_stencil_attachment(&depth_stencil_attachment.inner);
            texture_handles.push(depth_stencil_attachment._texture_handle);
        }

        RenderPassDescriptor {
            inner,
            _texture_handles: Arc::new(texture_handles),
            _occlusion_query_set_handle: None,
            _marker: Default::default(),
        }
    }
}

impl<T> RenderPassDescriptor<T, ()> {
    pub fn occlusion_query_set(
        mut self,
        occlusion_query_set: &OcclusionQuerySet,
    ) -> RenderPassDescriptor<T, OcclusionQueryState<O>> {
        self.inner
            .occlusion_query_set(occlusion_query_set.as_web_sys());

        RenderPassDescriptor {
            inner: self.inner,
            _texture_handles: self._texture_handles,
            _occlusion_query_set_handle: Some(occlusion_query_set.inner.clone()),
            _marker: Default::default(),
        }
    }
}

pub type ClearRenderPassEncoder<Target, Q> = RenderPassEncoder<Target, (), (), (), (), Q>;

pub struct RenderPassEncoder<Target, Pipeline, Vertex, Index, Resources, OcclusionQueryState> {
    inner: GpuRenderPassEncoder,
    command_encoder: CommandEncoder,
    current_pipeline_id: Option<usize>,
    current_vertex_buffers: [Option<CurrentBufferRange>; 8],
    current_index_buffer: Option<CurrentBufferRange>,
    current_bind_group_ids: [Option<usize>; 4],
    _marker: marker::PhantomData<(
        *const Target,
        *const Pipeline,
        *const Vertex,
        *const Index,
        *const Resources,
        OcclusionQueryState,
    )>,
}

impl<T, P, V, I, R, Q> resource_binding_command_encoder_seal::Seal
    for RenderPassEncoder<T, P, V, I, R, Q>
{
}
impl<T, P, V, I, R, Q> ResourceBindingCommandEncoder for RenderPassEncoder<T, P, V, I, R, Q> {
    type WithResources<RNew> = RenderPassEncoder<T, P, V, I, RNew, Q>;

    fn set_bind_groups<RNew>(self, bind_groups: RNew) -> Self::WithResources<RNew>
    where
        RNew: BindGroups,
    {
        let RenderPassEncoder {
            inner,
            mut command_encoder,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            mut current_bind_group_ids,
            ..
        } = self;

        for (i, encoding) in bind_groups.encodings().enumerate() {
            let BindGroupEncoding {
                bind_group,
                id,
                _resource_handles,
            } = encoding;

            if current_bind_group_ids[i] != Some(id) {
                inner.set_bind_group(i as u32, Some(&bind_group));
                command_encoder
                    ._resource_handles
                    .push(_resource_handles.into());

                current_bind_group_ids[i] = Some(id);
            }
        }

        RenderPassEncoder {
            inner,
            command_encoder,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            _marker: Default::default(),
        }
    }
}

impl<T, P, V, I, R, Q> render_state_encoder_seal::Seal for RenderPassEncoder<T, P, V, I, R, Q> {}
impl<T, P, V, I, R, Q> RenderStateEncoder<T> for RenderPassEncoder<T, P, V, I, R, Q> {
    type WithPipeline<PNew> = RenderPassEncoder<T, PNew, V, I, R, Q>;
    type WithVertexBuffers<VNew> = RenderPassEncoder<T, P, VNew, I, R, Q>;
    type WithIndexBuffer<INew> = RenderPassEncoder<T, P, V, INew, R, Q>;

    fn set_pipeline<PT, PV, PI, PR>(
        self,
        pipeline: &RenderPipeline<PT, PV, PI, PR>,
    ) -> Self::WithPipeline<RenderPipeline<PT, PV, PI, PR>>
    where
        PT: RenderLayoutCompatible<T>,
    {
        let RenderPassEncoder {
            inner,
            command_encoder,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            ..
        } = self;

        if Some(pipeline.id()) != current_pipeline_id {
            inner.set_pipeline(pipeline.as_web_sys());
        }

        RenderPassEncoder {
            inner,
            command_encoder,
            current_pipeline_id: Some(pipeline.id()),
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            _marker: Default::default(),
        }
    }

    fn set_vertex_buffers<VNew>(self, vertex_buffers: VNew) -> Self::WithVertexBuffers<VNew>
    where
        VNew: VertexBuffers,
    {
        let RenderPassEncoder {
            inner,
            mut command_encoder,
            current_pipeline_id,
            mut current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            ..
        } = self;

        for (i, encoding) in vertex_buffers.encodings().enumerate() {
            let VertexBufferEncoding {
                buffer,
                id,
                offset,
                size,
            } = encoding;

            let range = CurrentBufferRange { id, offset, size };

            if current_vertex_buffers[i] != Some(range) {
                inner.set_vertex_buffer_with_u32_and_u32(i as u32, Some(&buffer.buffer), offset, size);
                command_encoder._resource_handles.push(buffer.into());

                current_vertex_buffers[i] = Some(range);
            }
        }

        RenderPassEncoder {
            inner,
            command_encoder,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            _marker: Default::default(),
        }
    }

    fn set_index_buffer<INew>(self, index_buffer: INew) -> Self::WithIndexBuffer<INew>
    where
        INew: IndexBuffer,
    {
        let RenderPassEncoder {
            inner,
            mut command_encoder,
            current_pipeline_id,
            current_vertex_buffers,
            mut current_index_buffer,
            current_bind_group_ids,
            ..
        } = self;

        let IndexBufferEncoding {
            buffer,
            id,
            format,
            offset,
            size,
        } = index_buffer.to_encoding();

        let range = CurrentBufferRange { id, offset, size };

        if current_index_buffer != Some(range) {
            inner.set_index_buffer_with_u32_and_u32(&buffer.buffer, format, offset, size);
            command_encoder._resource_handles.push(buffer.into());

            current_index_buffer = Some(range);
        }

        RenderPassEncoder {
            inner,
            command_encoder,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            _marker: Default::default(),
        }
    }
}

impl<T, P, V, I, R, Q> RenderPassEncoder<T, P, V, I, R, Q> {
    pub fn set_viewport(self, viewport: Viewport) -> Self {
        let Viewport {
            x,
            y,
            width,
            height,
            min_depth,
            max_depth,
        } = viewport;

        self.inner
            .set_viewport(x, y, width, height, min_depth, max_depth);

        self
    }

    pub fn set_scissor_rect(self, scissor_rect: ScissorRect) -> Self {
        let ScissorRect {
            x,
            y,
            width,
            height,
        } = scissor_rect;

        self.inner.set_scissor_rect(x, y, width, height);

        self
    }

    pub fn set_blend_constant(self, blend_constant: BlendConstant) -> Self {
        let BlendConstant { r, g, b, a } = blend_constant;

        let color = GpuColorDict::new(r as f64, g as f64, b as f64, a as f64);

        self.inner.set_blend_constant_with_gpu_color_dict(&color);

        self
    }

    pub fn set_stencil_reference(self, stencil_reference: u32) -> Self {
        self.inner.set_stencil_reference(stencil_reference);

        self
    }

    pub fn clear_state(self) -> ClearRenderPassEncoder<T, Q> {
        let RenderPassEncoder {
            inner,
            command_encoder,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            ..
        } = self;

        RenderPassEncoder {
            inner,
            command_encoder,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            _marker: Default::default(),
        }
    }

    pub fn execute_bundle(self, render_bundle: &RenderBundle<T>) -> ClearRenderPassEncoder<T, Q> {
        let RenderPassEncoder {
            inner,
            mut command_encoder,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            ..
        } = self;

        let array = js_sys::Array::new();

        array.push(render_bundle.inner.as_ref());

        inner.execute_bundles(array.as_ref());

        command_encoder
            ._resource_handles
            .push(render_bundle._resource_handles.clone().into());

        RenderPassEncoder {
            inner,
            command_encoder,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            _marker: Default::default(),
        }
    }

    pub fn execute_bundles<B>(self, render_bundles: B) -> ClearRenderPassEncoder<T, Q>
    where
        B: IntoIterator,
        B::Item: Borrow<RenderBundle<T>>,
    {
        let RenderPassEncoder {
            inner,
            mut command_encoder,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            ..
        } = self;

        let array = js_sys::Array::new();

        for bundle in render_bundles {
            let bundle = bundle.borrow();

            array.push(bundle.inner.as_ref());

            command_encoder
                ._resource_handles
                .push(bundle._resource_handles.clone().into());
        }

        inner.execute_bundles(array.as_ref());

        RenderPassEncoder {
            inner,
            command_encoder,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            _marker: Default::default(),
        }
    }
}

impl<T, P, V, I, R, Q> draw_command_encoder_seal::Seal for RenderPassEncoder<T, P, V, I, R, Q> {}
impl<T, PT, PV, PI, PR, V, I, R, Q> DrawCommandEncoder
    for RenderPassEncoder<T, RenderPipeline<PT, PV, PI, PR>, V, I, R, Q>
where
    V: VertexBuffers<Layout = PV>,
    R: BindGroups<Layout = PR>,
{
    fn draw(self, draw: Draw) -> Self {
        let Draw {
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        } = draw;

        self.inner
            .draw_with_instance_count_and_first_vertex_and_first_instance(
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            );

        self
    }

    fn draw_indirect<U>(self, view: buffer::View<Draw, U>) -> Self
    where
        U: buffer::Indirect,
    {
        self.inner
            .draw_indirect_with_u32(view.as_web_sys(), view.offset_in_bytes() as u32);

        self
    }
}

impl<T, P, V, I, R, Q> draw_indexed_command_encoder_seal::Seal
    for RenderPassEncoder<T, P, V, I, R, Q>
{
}
impl<T, PT, PV, PI, PR, V, I, R, Q> DrawIndexedCommandEncoder
    for RenderPassEncoder<T, RenderPipeline<PT, PV, PI, PR>, V, I, R, Q>
where
    PI: PipelineIndexFormat,
    V: VertexBuffers<Layout = PV>,
    I: IndexBuffer,
    I::IndexData: PipelineIndexFormatCompatible<PI>,
    R: BindGroups<Layout = PR>,
{
    fn draw_indexed(self, draw_indexed: DrawIndexed) -> Self {
        let DrawIndexed {
            index_count,
            instance_count,
            first_index,
            base_vertex,
            first_instance,
        } = draw_indexed;

        // TODO: base_vertex in specced to be a signed integer here, but specced to be an unsigned
        // integer in the indirect version. Going with unsigned for both for now (what's the
        // use-case for a negative base vertex number?), but should investigate.
        self.inner
            .draw_indexed_with_instance_count_and_first_index_and_base_vertex_and_first_instance(
                index_count,
                instance_count,
                first_index,
                base_vertex as i32,
                first_instance,
            );

        self
    }

    fn draw_indexed_indirect<U>(self, view: buffer::View<DrawIndexed, U>) -> Self
    where
        U: buffer::Indirect,
    {
        self.inner
            .draw_indexed_indirect_with_u32(view.as_web_sys(), view.offset_in_bytes() as u32);

        self
    }
}

impl<T, P, V, I, R, Q> RenderPassEncoder<T, P, V, I, R, Q>
where
    Q: BeginOcclusionQuery,
{
    pub fn begin_occlusion_query(self, query_index: u32) -> RenderPassEncoder<T, P, V, I, R, X> {
        let RenderPassEncoder {
            inner,
            command_encoder,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            ..
        } = self;

        inner.begin_occlusion_query(query_index);

        RenderPassEncoder {
            inner,
            command_encoder,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            _marker: Default::default(),
        }
    }
}

impl<T, P, V, I, R, Q> RenderPassEncoder<T, P, V, I, R, Q>
where
    Q: EndOcclusionQuery,
{
    pub fn end_occlusion_query(self) -> RenderPassEncoder<T, P, V, I, R, X> {
        let RenderPassEncoder {
            inner,
            command_encoder,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            ..
        } = self;

        inner.end_occlusion_query();

        RenderPassEncoder {
            inner,
            command_encoder,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            _marker: Default::default(),
        }
    }
}

impl<T, P, V, I, R, Q> RenderPassEncoder<T, P, V, I, R, Q>
where
    Q: EndRenderPass,
{
    pub fn end(self) -> CommandEncoder {
        self.inner.end();

        self.command_encoder
    }
}

pub struct RenderBundle<Target> {
    inner: GpuRenderBundle,
    _resource_handles: Arc<Vec<ResourceHandle>>,
    _marker: marker::PhantomData<Target>,
}

pub struct RenderBundleEncoderDescriptor<Target> {
    inner: GpuRenderBundleEncoderDescriptor,
    _marker: marker::PhantomData<Target>,
}

impl RenderBundleEncoderDescriptor<()> {
    pub fn new<C>() -> RenderBundleEncoderDescriptor<RenderLayout<C, ()>>
    where
        C: TypedColorLayout,
    {
        let color_formats = js_sys::Array::new();

        for format in C::COLOR_FORMATS {
            color_formats.push(&JsValue::from(format.as_str()));
        }

        let inner = GpuRenderBundleEncoderDescriptor::new(&color_formats);

        RenderBundleEncoderDescriptor {
            inner,
            _marker: Default::default(),
        }
    }

    pub fn multisample<C, const SAMPLES: u8>(
    ) -> RenderBundleEncoderDescriptor<MultisampleRenderLayout<C, (), SAMPLES>>
    where
        C: TypedMultisampleColorLayout,
    {
        let color_formats = js_sys::Array::new();

        for format in C::COLOR_FORMATS {
            color_formats.push(&JsValue::from(format.as_str()));
        }

        let mut inner = GpuRenderBundleEncoderDescriptor::new(&color_formats);

        inner.sample_count(SAMPLES as u32);

        RenderBundleEncoderDescriptor {
            inner,
            _marker: Default::default(),
        }
    }
}

impl<C> RenderBundleEncoderDescriptor<RenderLayout<C, ()>> {
    pub fn depth_stencil_format<Ds>(self) -> RenderBundleEncoderDescriptor<RenderLayout<C, Ds>>
    where
        Ds: DepthStencilRenderable,
    {
        let RenderBundleEncoderDescriptor { mut inner, .. } = self;

        inner.depth_stencil_format(Ds::FORMAT_ID.to_web_sys());

        RenderBundleEncoderDescriptor {
            inner,
            _marker: Default::default(),
        }
    }

    pub fn depth_stencil_format_read_only<Ds>(
        self,
    ) -> RenderBundleEncoderDescriptor<RenderLayout<C, ReadOnly<Ds>>>
    where
        Ds: DepthStencilRenderable,
    {
        let RenderBundleEncoderDescriptor { mut inner, .. } = self;

        inner.depth_stencil_format(Ds::FORMAT_ID.to_web_sys());

        RenderBundleEncoderDescriptor {
            inner,
            _marker: Default::default(),
        }
    }
}

impl<C, const SAMPLES: u8> RenderBundleEncoderDescriptor<MultisampleRenderLayout<C, (), SAMPLES>> {
    pub fn depth_stencil_format<Ds>(
        self,
    ) -> RenderBundleEncoderDescriptor<MultisampleRenderLayout<C, Ds, SAMPLES>>
    where
        Ds: DepthStencilRenderable,
    {
        let RenderBundleEncoderDescriptor { mut inner, .. } = self;

        inner.depth_stencil_format(Ds::FORMAT_ID.to_web_sys());

        RenderBundleEncoderDescriptor {
            inner,
            _marker: Default::default(),
        }
    }

    pub fn depth_stencil_format_read_only<Ds>(
        self,
    ) -> RenderBundleEncoderDescriptor<MultisampleRenderLayout<C, ReadOnly<Ds>, SAMPLES>>
    where
        Ds: DepthStencilRenderable,
    {
        let RenderBundleEncoderDescriptor { mut inner, .. } = self;

        inner.depth_stencil_format(Ds::FORMAT_ID.to_web_sys());

        RenderBundleEncoderDescriptor {
            inner,
            _marker: Default::default(),
        }
    }
}

pub struct RenderBundleEncoder<Target, Pipeline, Vertex, Index, Resources> {
    inner: GpuRenderBundleEncoder,
    current_pipeline_id: Option<usize>,
    current_vertex_buffers: [Option<CurrentBufferRange>; 8],
    current_index_buffer: Option<CurrentBufferRange>,
    current_bind_group_ids: [Option<usize>; 4],
    _resource_handles: Vec<ResourceHandle>,
    _marker: marker::PhantomData<(
        *const Target,
        *const Pipeline,
        *const Vertex,
        *const Index,
        *const Resources,
    )>,
}

impl<T, P, V, I, R> RenderBundleEncoder<T, P, V, I, R> {
    pub fn new(device: &Device, descriptor: &RenderBundleEncoderDescriptor<T>) -> Self {
        let inner = device.inner.create_render_bundle_encoder(&descriptor.inner);

        RenderBundleEncoder {
            inner,
            current_pipeline_id: None,
            current_vertex_buffers: [None; 8],
            current_index_buffer: None,
            current_bind_group_ids: [None; 4],
            _resource_handles: Vec::new(),
            _marker: Default::default(),
        }
    }
}

impl<T, P, V, I, R> RenderBundleEncoder<T, P, V, I, R> {
    pub fn finish(self) -> RenderBundle<T> {
        RenderBundle {
            inner: self.inner.finish(),
            _resource_handles: Arc::new(self._resource_handles),
            _marker: Default::default(),
        }
    }
}

impl<T, P, V, I, R> resource_binding_command_encoder_seal::Seal
    for RenderBundleEncoder<T, P, V, I, R>
{
}
impl<T, P, V, I, R> ResourceBindingCommandEncoder for RenderBundleEncoder<T, P, V, I, R> {
    type WithResources<RNew> = RenderBundleEncoder<T, P, V, I, RNew>;

    fn set_bind_groups<RNew>(self, bind_groups: RNew) -> Self::WithResources<RNew>
    where
        RNew: BindGroups,
    {
        let RenderBundleEncoder {
            inner,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            mut current_bind_group_ids,
            _resource_handles: mut _resource_handles,
            ..
        } = self;

        for (i, encoding) in bind_groups.encodings().enumerate() {
            let BindGroupEncoding {
                bind_group,
                id,
                _resource_handles: bind_group_resource_handles,
            } = encoding;

            if current_bind_group_ids[i] != Some(id) {
                inner.set_bind_group(i as u32, Some(&bind_group));
                _resource_handles.push(bind_group_resource_handles.into());

                current_bind_group_ids[i] = Some(id);
            }
        }

        RenderBundleEncoder {
            inner,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            _resource_handles,
            _marker: Default::default(),
        }
    }
}

impl<T, P, V, I, R> render_state_encoder_seal::Seal for RenderBundleEncoder<T, P, V, I, R> {}
impl<T, P, V, I, R> RenderStateEncoder<T> for RenderBundleEncoder<T, P, V, I, R> {
    type WithPipeline<PNew> = RenderBundleEncoder<T, PNew, V, I, R>;
    type WithVertexBuffers<VNew> = RenderBundleEncoder<T, P, VNew, I, R>;
    type WithIndexBuffer<INew> = RenderBundleEncoder<T, P, V, INew, R>;

    fn set_pipeline<PT, PV, PI, PR>(
        self,
        pipeline: &RenderPipeline<PT, PV, PI, PR>,
    ) -> Self::WithPipeline<RenderPipeline<PT, PV, PI, PR>>
    where
        PT: RenderLayoutCompatible<T>,
    {
        let RenderBundleEncoder {
            inner,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            _resource_handles,
            ..
        } = self;

        if Some(pipeline.id()) != current_pipeline_id {
            inner.set_pipeline(pipeline.as_web_sys());
        }

        RenderBundleEncoder {
            inner,
            current_pipeline_id: Some(pipeline.id()),
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            _resource_handles,
            _marker: Default::default(),
        }
    }

    fn set_vertex_buffers<VNew>(self, vertex_buffers: VNew) -> Self::WithVertexBuffers<VNew>
    where
        VNew: VertexBuffers,
    {
        let RenderBundleEncoder {
            inner,
            current_pipeline_id,
            mut current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            mut _resource_handles,
            ..
        } = self;

        for (i, encoding) in vertex_buffers.encodings().enumerate() {
            let VertexBufferEncoding {
                buffer,
                id,
                offset,
                size,
            } = encoding;

            let range = CurrentBufferRange { id, offset, size };

            if current_vertex_buffers[i] != Some(range) {
                inner.set_vertex_buffer_with_u32_and_u32(i as u32, Some(&buffer.buffer), offset, size);
                _resource_handles.push(buffer.into());

                current_vertex_buffers[i] = Some(range);
            }
        }

        RenderBundleEncoder {
            inner,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            _resource_handles,
            _marker: Default::default(),
        }
    }

    fn set_index_buffer<INew>(self, index_buffer: INew) -> Self::WithIndexBuffer<INew>
    where
        INew: IndexBuffer,
    {
        let RenderBundleEncoder {
            inner,
            current_pipeline_id,
            current_vertex_buffers,
            mut current_index_buffer,
            current_bind_group_ids,
            mut _resource_handles,
            ..
        } = self;

        let IndexBufferEncoding {
            buffer,
            id,
            format,
            offset,
            size,
        } = index_buffer.to_encoding();

        let range = CurrentBufferRange { id, offset, size };

        if current_index_buffer != Some(range) {
            inner.set_index_buffer_with_u32_and_u32(&buffer.buffer, format, offset, size);
            _resource_handles.push(buffer.into());

            current_index_buffer = Some(range);
        }

        RenderBundleEncoder {
            inner,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            _resource_handles,
            _marker: Default::default(),
        }
    }
}

impl<T, P, V, I, R> draw_command_encoder_seal::Seal for RenderBundleEncoder<T, P, V, I, R> {}
impl<T, PT, PV, PI, PR, V, I, R> DrawCommandEncoder
    for RenderBundleEncoder<T, RenderPipeline<PT, PV, PI, PR>, V, I, R>
where
    V: VertexBuffers<Layout = PV>,
    R: BindGroups<Layout = PR>,
{
    fn draw(self, draw: Draw) -> Self {
        let Draw {
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        } = draw;

        self.inner
            .draw_with_instance_count_and_first_vertex_and_first_instance(
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            );

        self
    }

    fn draw_indirect<U>(self, view: buffer::View<Draw, U>) -> Self
    where
        U: buffer::Indirect,
    {
        self.inner
            .draw_indirect_with_u32(view.as_web_sys(), view.offset_in_bytes() as u32);

        self
    }
}

impl<T, P, V, I, R> draw_indexed_command_encoder_seal::Seal for RenderBundleEncoder<T, P, V, I, R> {}
impl<T, PT, PV, PI, PR, V, I, R> DrawIndexedCommandEncoder
    for RenderBundleEncoder<T, RenderPipeline<PT, PV, PI, PR>, V, I, R>
where
    PI: PipelineIndexFormat,
    V: VertexBuffers<Layout = PV>,
    I: IndexBuffer,
    I::IndexData: PipelineIndexFormatCompatible<PI>,
    R: BindGroups<Layout = PR>,
{
    fn draw_indexed(self, draw_indexed: DrawIndexed) -> Self {
        let DrawIndexed {
            index_count,
            instance_count,
            first_index,
            base_vertex,
            first_instance,
        } = draw_indexed;

        // TODO: base_vertex in specced to be a signed integer here, but specced to be an unsigned
        // integer in the indirect version. Going with unsigned for both for now (what's the
        // use-case for a negative base vertex number?), but should investigate.
        self.inner
            .draw_indexed_with_instance_count_and_first_index_and_base_vertex_and_first_instance(
                index_count,
                instance_count,
                first_index,
                base_vertex as i32,
                first_instance,
            );

        self
    }

    fn draw_indexed_indirect<U>(self, view: buffer::View<DrawIndexed, U>) -> Self
    where
        U: buffer::Indirect,
    {
        self.inner
            .draw_indexed_indirect_with_u32(view.as_web_sys(), view.offset_in_bytes() as u32);

        self
    }
}
