use crate::buffer::BufferDestroyer;
use crate::command::{
    BindGroupEncoding, BindGroups, IndexBuffer, IndexBufferEncoding, VertexBufferEncoding,
    VertexBuffers,
};
use crate::compute_pipeline::ComputePipeline;
use crate::query::OcclusionQuerySet;
use crate::render_pipeline::{
    IndexData, PipelineIndexFormat, PipelineIndexFormatCompatible, RenderPipeline,
};
use crate::render_target::{RenderTargetEncoding, ValidRenderTarget};
use crate::resource_binding::{BindGroupEntry, EntryDestroyer};
use crate::texture::format::{ImageData, TextureFormat};
use crate::texture::TextureDestroyer;
use crate::type_flag::{TypeFlag, O, X};
use crate::{buffer, texture};
use std::any::Any;
use std::ops::{Range, Rem};
use std::sync::Arc;
use std::{marker, mem};
use web_sys::{
    GpuBuffer, GpuColorDict, GpuCommandBuffer, GpuCommandEncoder, GpuComputePassEncoder,
    GpuExtent3dDict, GpuRenderPassDescriptor, GpuRenderPassEncoder,
};
use crate::device::Device;

enum ResourceDestroyer {
    Buffer(Arc<BufferDestroyer>),
    Texture(Arc<TextureDestroyer>),
    BindGroup(Arc<Vec<EntryDestroyer>>),
}

impl From<Arc<BufferDestroyer>> for ResourceDestroyer {
    fn from(destroyer: Arc<BufferDestroyer>) -> Self {
        ResourceDestroyer::Buffer(destroyer)
    }
}

impl From<Arc<TextureDestroyer>> for ResourceDestroyer {
    fn from(destroyer: Arc<TextureDestroyer>) -> Self {
        ResourceDestroyer::Texture(destroyer)
    }
}

impl From<Arc<Vec<EntryDestroyer>>> for ResourceDestroyer {
    fn from(destroyer: Arc<Vec<EntryDestroyer>>) -> Self {
        ResourceDestroyer::BindGroup(destroyer)
    }
}

pub struct CommandBuffer {
    inner: GpuCommandBuffer,
    _resource_destroyers: Vec<ResourceDestroyer>,
}

impl CommandBuffer {
    pub(crate) fn as_web_sys(&self) -> &GpuCommandBuffer {
        &self.inner
    }
}

pub struct CommandEncoder {
    inner: GpuCommandEncoder,
    _resource_destroyers: Vec<ResourceDestroyer>,
}

impl CommandEncoder {
    pub(crate) fn new(device: &Device) -> Self {
        CommandEncoder {
            inner: device.inner.create_command_encoder(),
            _resource_destroyers: Vec::new()
        }
    }

    pub fn copy_buffer_to_buffer<T, U0, U1>(
        self,
        src: buffer::View<T, U0>,
        dst: buffer::View<T, U1>,
    ) -> CommandEncoder
    where
        U0: buffer::CopySrc + 'static,
        U1: buffer::CopyDst + 'static,
        T: 'static,
    {
        todo!();

        self
    }

    pub fn copy_buffer_to_buffer_slice<T, U0, U1>(
        self,
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

        todo!();

        self
    }

    fn image_copy_buffer_to_texture_internal<F>(
        mut self,
        src: &buffer::ImageCopyBuffer,
        dst: &texture::ImageCopyFromBufferDst<F>,
    ) -> Self {
        let width = dst.inner.width;
        let height = dst.inner.height;
        let depth_or_layers = dst.inner.depth_or_layers;

        src.validate_with_size_and_block_size(
            ImageCopySize {
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

        self._resource_destroyers.push(src.buffer.clone().into());
        self._resource_destroyers
            .push(dst.inner.texture.clone().into());

        self
    }

    pub fn image_copy_buffer_to_texture<T, F>(
        self,
        src: &buffer::ImageCopySrc<T>,
        dst: &texture::ImageCopyFromBufferDst<F>,
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
        dst: &texture::ImageCopyFromBufferDst<F>,
    ) -> Self {
        self.image_copy_buffer_to_texture_internal(&src.inner, dst)
    }

    fn sub_image_copy_buffer_to_texture_internal<F>(
        mut self,
        src: &buffer::ImageCopyBuffer,
        dst: &texture::SubImageCopyFromBufferDst<F>,
        size: ImageCopySize,
    ) -> Self {
        size.validate_with_block_size(dst.inner.block_size);
        src.validate_with_size_and_block_size(size, dst.inner.block_size);
        dst.inner.validate_dst_with_size(size);

        self.inner.copy_buffer_to_texture_with_gpu_extent_3d_dict(
            &src.to_web_sys(),
            &dst.inner.to_web_sys(),
            &size.to_web_sys(),
        );

        self._resource_destroyers.push(src.buffer.clone().into());
        self._resource_destroyers
            .push(dst.inner.texture.clone().into());

        self
    }

    pub fn sub_image_copy_buffer_to_texture<T, F>(
        self,
        src: &buffer::ImageCopySrc<T>,
        dst: &texture::SubImageCopyFromBufferDst<F>,
        size: ImageCopySize,
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
        dst: &texture::SubImageCopyFromBufferDst<F>,
        size: ImageCopySize,
    ) -> Self {
        self.sub_image_copy_buffer_to_texture_internal(&src.inner, dst, size)
    }

    fn image_copy_texture_to_buffer_internal<F>(
        mut self,
        src: &texture::ImageCopyToBufferSrc<F>,
        dst: &buffer::ImageCopyBuffer,
    ) -> Self {
        let width = src.inner.width;
        let height = src.inner.height;
        let depth_or_layers = src.inner.depth_or_layers;

        dst.validate_with_size_and_block_size(
            ImageCopySize {
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

        self._resource_destroyers
            .push(src.inner.texture.clone().into());
        self._resource_destroyers.push(dst.buffer.clone().into());

        self
    }

    pub fn image_copy_texture_to_buffer<F, T>(
        self,
        src: &texture::ImageCopyToBufferSrc<F>,
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
        src: &texture::ImageCopyToBufferSrc<F>,
        dst: &buffer::ImageCopyDstRaw,
    ) -> Self {
        self.image_copy_texture_to_buffer_internal(src, &dst.inner)
    }

    fn sub_image_copy_texture_to_buffer_internal<F>(
        mut self,
        src: &texture::SubImageCopyToBufferSrc<F>,
        dst: &buffer::ImageCopyBuffer,
        size: ImageCopySize,
    ) -> Self {
        size.validate_with_block_size(src.inner.block_size);
        src.inner.validate_src_with_size(size);
        dst.validate_with_size_and_block_size(size, src.inner.block_size);

        self.inner.copy_texture_to_buffer_with_gpu_extent_3d_dict(
            &src.inner.to_web_sys(),
            &dst.to_web_sys(),
            &size.to_web_sys(),
        );

        self._resource_destroyers
            .push(src.inner.texture.clone().into());
        self._resource_destroyers.push(dst.buffer.clone().into());

        self
    }

    pub fn sub_image_copy_texture_to_buffer<F, T>(
        self,
        src: &texture::SubImageCopyToBufferSrc<F>,
        dst: &buffer::ImageCopyDst<T>,
        size: ImageCopySize,
    ) -> Self
    where
        F: TextureFormat,
        T: ImageData<F>,
    {
        self.sub_image_copy_texture_to_buffer_internal(src, &dst.inner, size)
    }

    pub fn sub_image_copy_texture_to_buffer_raw<F>(
        self,
        src: &texture::SubImageCopyToBufferSrc<F>,
        dst: &buffer::ImageCopyDstRaw,
        size: ImageCopySize,
    ) -> Self {
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

        self._resource_destroyers
            .push(src.inner.texture.clone().into());
        self._resource_destroyers
            .push(dst.inner.texture.clone().into());

        self
    }

    pub fn sub_image_copy_texture_to_texture<F>(
        self,
        src: &texture::SubImageCopyToTextureSrc<F>,
        dst: &texture::SubImageCopyFromTextureDst<F>,
        size: ImageCopySize,
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

        self._resource_destroyers
            .push(src.inner.texture.clone().into());
        self._resource_destroyers
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
            _marker: Default::default()
        }
    }

    pub fn begin_render_pass<T, Q>(
        self,
        descriptor: &RenderPassDescriptor<T, Q>,
    ) -> RenderPassEncoder<T, (), (), (), (), Q> {
        let inner = self.inner.begin_render_pass(&descriptor.inner);

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

    pub fn resolve_occlusion_query_set<U>(
        self,
        query: &OcclusionQuerySet,
        offset: u32,
        view: buffer::View<[u32], U>,
    ) -> Self
    where
        U: buffer::QueryResolve,
    {
        assert!(
            offset + view.len() as u32 <= query.len(),
            "resolve range out of bounds"
        );

        self.inner.resolve_query_set_with_u32(
            query.as_web_sys(),
            offset,
            view.len() as u32,
            view.as_web_sys(),
            view.offset_in_bytes() as u32,
        );

        self
    }

    pub fn finish(self) -> CommandBuffer {
        let CommandEncoder {
            inner,
            _resource_destroyers,
        } = self;

        CommandBuffer {
            inner: inner.finish(),
            _resource_destroyers,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ImageCopySize {
    pub width: u32,
    pub height: u32,
    pub depth_or_layers: u32,
}

impl ImageCopySize {
    pub(crate) fn validate_with_block_size(&self, block_size: [u32; 2]) {
        let ImageCopySize {
            width,
            height,
            depth_or_layers,
        } = *self;

        assert!(width != 0, "copy width cannot be `0`");
        assert!(height != 0, "copy height cannot be `0`");
        assert!(
            depth_or_layers != 0,
            "copy depth or layer count cannot be `0`"
        );

        let [block_width, block_height] = block_size;

        assert!(
            width.rem(block_width) == 0,
            "copy width must be a multiple of the block width (`{}`)",
            block_width
        );
        assert!(
            height.rem(block_height) == 0,
            "copy height must be a multiple of the block height (`{}`)",
            block_height
        );
    }

    pub(crate) fn to_web_sys(&self) -> GpuExtent3dDict {
        let ImageCopySize {
            width,
            height,
            depth_or_layers,
        } = *self;

        let mut extent = GpuExtent3dDict::new(width);

        extent.height(height);
        extent.depth_or_array_layers(depth_or_layers);

        extent
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct DispatchWorkgroups {
    pub count_x: u32,
    pub count_y: u32,
    pub count_z: u32,
}

pub struct ComputePassEncoder<Pipeline, Resources> {
    inner: GpuComputePassEncoder,
    command_encoder: CommandEncoder,
    current_pipeline_id: Option<usize>,
    current_bind_group_ids: [Option<usize>; 4],
    _marker: marker::PhantomData<(*const Pipeline, *const Resources)>,
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

    pub fn set_bind_groups<RNew>(self, bind_groups: RNew) -> ComputePassEncoder<P, RNew>
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
                _resource_destroyers,
            } = encoding;

            if current_bind_group_ids[i] != Some(id) {
                inner.set_bind_group(i as u32, &bind_group);
                command_encoder
                    ._resource_destroyers
                    .push(_resource_destroyers.into());

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

    pub fn end(self) -> CommandEncoder {
        self.inner.end_pass();

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

        self.inner.dispatch_with_y_and_z(count_x, count_y, count_z);

        self
    }

    pub fn dispatch_workgroups_indirect<U>(self, view: buffer::View<DispatchWorkgroups, U>) -> Self
    where
        U: buffer::Indirect,
    {
        self.inner
            .dispatch_indirect_with_u32(view.as_web_sys(), view.size_in_bytes() as u32);

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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct DrawIndexed {
    pub index_count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub base_vertex: u32,
    pub first_instance: u32,
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
    _marker: marker::PhantomData<(*const RenderTarget, OcclusionQueryState)>,
}

impl<T> RenderPassDescriptor<T, ()>
where
    T: ValidRenderTarget,
{
    pub fn new(render_target: T) -> Self {
        let RenderTargetEncoding {
            color_attachments,
            depth_stencil_attachment,
        } = render_target.encoding();

        let mut inner = GpuRenderPassDescriptor::new(&color_attachments);

        if let Some(depth_stencil_attachment) = depth_stencil_attachment {
            inner.depth_stencil_attachment(&depth_stencil_attachment);
        }

        RenderPassDescriptor {
            inner,
            _marker: Default::default(),
        }
    }

    pub fn occlusion_query_set(
        mut self,
        occlusion_query_set: &OcclusionQuerySet,
    ) -> RenderPassDescriptor<T, OcclusionQueryState<O>> {
        self.inner
            .occlusion_query_set(occlusion_query_set.as_web_sys());

        RenderPassDescriptor {
            inner: self.inner,
            _marker: Default::default(),
        }
    }
}

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

    pub fn set_pipeline<PV, PI, PR>(
        self,
        pipeline: &RenderPipeline<T, PV, PI, PR>,
    ) -> RenderPassEncoder<T, RenderPipeline<T, PV, PI, PR>, V, I, R, Q> {
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

    pub fn set_vertex_buffers<VNew>(
        self,
        vertex_buffers: VNew,
    ) -> RenderPassEncoder<T, P, VNew, I, R, Q>
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
                inner.set_vertex_buffer_with_u32_and_u32(i as u32, &buffer.buffer, offset, size);
                command_encoder._resource_destroyers.push(buffer.into());

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

    pub fn set_index_buffer<INew>(
        self,
        index_buffer: INew,
    ) -> RenderPassEncoder<T, P, V, INew, R, Q>
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
            command_encoder._resource_destroyers.push(buffer.into());

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

    pub fn set_bind_groups<RNew>(self, bind_groups: RNew) -> RenderPassEncoder<T, P, V, I, RNew, Q>
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
                _resource_destroyers,
            } = encoding;

            if current_bind_group_ids[i] != Some(id) {
                inner.set_bind_group(i as u32, &bind_group);
                command_encoder
                    ._resource_destroyers
                    .push(_resource_destroyers.into());

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

impl<T, VLayout, IFormat, RLayout, V, I, R, Q>
    RenderPassEncoder<T, RenderPipeline<T, VLayout, IFormat, RLayout>, V, I, R, Q>
where
    V: VertexBuffers<Layout = VLayout>,
    R: BindGroups<Layout = RLayout>,
{
    pub fn draw(self, draw: Draw) -> Self {
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

    pub fn draw_indirect<U>(self, view: buffer::View<Draw, U>) -> Self
    where
        U: buffer::Indirect,
    {
        self.inner
            .draw_indirect_with_u32(view.as_web_sys(), view.size_in_bytes() as u32);

        self
    }
}

impl<T, VLayout, IFormat, RLayout, V, I, R, Q>
    RenderPassEncoder<T, RenderPipeline<T, VLayout, IFormat, RLayout>, V, I, R, Q>
where
    IFormat: PipelineIndexFormat,
    V: VertexBuffers<Layout = VLayout>,
    I: IndexBuffer,
    I::IndexData: PipelineIndexFormatCompatible<IFormat>,
    R: BindGroups<Layout = RLayout>,
{
    pub fn draw_indexed(self, draw_indexed: DrawIndexed) -> Self {
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

    pub fn draw_indexed_indirect<U>(self, view: buffer::View<DrawIndexed, U>) -> Self
    where
        U: buffer::Indirect,
    {
        self.inner
            .draw_indexed_indirect_with_u32(view.as_web_sys(), view.size_in_bytes() as u32);

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
        self.inner.end_pass();

        self.command_encoder
    }
}
