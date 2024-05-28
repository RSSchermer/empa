use std::borrow::Cow;
use std::ops::{Range, Rem};
use std::{marker, mem};

use crate::abi::{MemoryUnit, MemoryUnitLayout};
use crate::buffer::image_copy_buffer_validate;
use crate::command::{
    BindGroupEncoding, BindGroups, IndexBuffer, IndexBufferEncoding, VertexBufferEncoding,
    VertexBuffers,
};
use crate::compute_pipeline::ComputePipeline;
use crate::device::Device;
use crate::driver::{
    ClearBuffer, CommandEncoder as _, ComputePassEncoder as _, CopyBufferToBuffer,
    CopyBufferToTexture, CopyTextureToBuffer, CopyTextureToTexture, Device as _, Driver, Dvr,
    ExecuteRenderBundlesEncoder, ImageCopyBuffer, ProgrammablePassEncoder,
    RenderBundleEncoder as _, RenderEncoder, RenderPassEncoder as _, ResolveQuerySet,
    SetIndexBuffer, SetVertexBuffer,
};
use crate::query::{OcclusionQuerySet, TimestampQuerySet};
use crate::render_pipeline::{PipelineIndexFormat, PipelineIndexFormatCompatible, RenderPipeline};
use crate::render_target::{
    MultisampleRenderLayout, ReadOnly, RenderLayout, RenderLayoutCompatible, TypedColorLayout,
    TypedMultisampleColorLayout, ValidRenderTarget,
};
use crate::texture::format::{DepthStencilRenderable, ImageData, TextureFormat, TextureFormatId};
use crate::texture::ImageCopySize3D;
use crate::type_flag::{TypeFlag, O, X};
use crate::{abi, buffer, driver, texture};

pub struct CommandBuffer {
    pub(crate) handle: <Dvr as Driver>::CommandBufferHandle,
}

pub struct CommandEncoder {
    handle: <Dvr as Driver>::CommandEncoderHandle,
}

impl CommandEncoder {
    pub(crate) fn new(device: &Device) -> Self {
        CommandEncoder {
            handle: device.handle.create_command_encoder(),
        }
    }

    pub fn clear_buffer<T, U>(mut self, buffer: buffer::View<T, U>) -> CommandEncoder
    where
        U: buffer::CopyDst + 'static,
    {
        let size = buffer.size_in_bytes();
        let offset = buffer.offset_in_bytes();

        assert!(
            size.rem(4) == 0,
            "cleared region's size in bytes must be a multiple of `4`"
        );
        assert!(
            offset.rem(4) == 0,
            "cleared region's offset in bytes must be a multiple of `8`"
        );

        let start = offset;
        let end = offset + size;

        self.handle.clear_buffer(ClearBuffer {
            buffer: &buffer.buffer.handle,
            range: start..end,
        });

        self
    }

    pub fn clear_buffer_slice<T, U>(mut self, buffer: buffer::View<[T], U>) -> CommandEncoder
    where
        U: buffer::CopyDst + 'static,
    {
        let size = buffer.size_in_bytes();
        let offset = buffer.offset_in_bytes();

        assert!(
            size.rem(4) == 0,
            "cleared region's size in bytes must be a multiple of `4`"
        );
        assert!(
            offset.rem(4) == 0,
            "cleared region's offset in bytes must be a multiple of `8`"
        );

        let start = offset;
        let end = offset + size;

        self.handle.clear_buffer(ClearBuffer {
            buffer: &buffer.buffer.handle,
            range: start..end,
        });

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
        let source_offset = src.offset_in_bytes();
        let destination_offset = dst.offset_in_bytes();
        let size = mem::size_of::<T>();

        assert!(
            size.rem(4) == 0,
            "copied size in bytes must be a multiple of `4`"
        );

        // This may be redundant, because the offset must be sized aligned anyway?
        assert!(
            source_offset.rem(4) == 0,
            "`src` view's offset in bytes must be a multiple of `8`"
        );
        assert!(
            destination_offset.rem(4) == 0,
            "`dst` view's offset in bytes must be a multiple of `8`"
        );

        self.handle.copy_buffer_to_buffer(CopyBufferToBuffer {
            source: &src.buffer.handle,
            source_offset,
            destination: &dst.buffer.handle,
            destination_offset,
            size,
        });

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

        let source_offset = src.offset_in_bytes();
        let destination_offset = dst.offset_in_bytes();

        debug_assert!(src.size_in_bytes() == dst.size_in_bytes());

        let size = src.size_in_bytes();

        assert!(
            size.rem(4) == 0,
            "copied size in bytes must be a multiple of `4`"
        );
        assert!(
            source_offset.rem(4) == 0,
            "`src` view's offset in bytes must be a multiple of `8`"
        );
        assert!(
            destination_offset.rem(4) == 0,
            "`dst` view's offset in bytes must be a multiple of `8`"
        );

        self.handle.copy_buffer_to_buffer(CopyBufferToBuffer {
            source: &src.buffer.handle,
            source_offset,
            destination: &dst.buffer.handle,
            destination_offset,
            size,
        });

        self
    }

    fn image_copy_buffer_to_texture_internal<F>(
        mut self,
        src: ImageCopyBuffer<Dvr>,
        dst: texture::ImageCopyTexture<F>,
    ) -> Self {
        let width = dst.width;
        let height = dst.height;
        let depth_or_layers = dst.depth_or_layers;

        image_copy_buffer_validate(&src, (width, height, depth_or_layers), dst.block_size);

        self.handle.copy_buffer_to_texture(CopyBufferToTexture {
            source: src,
            destination: dst.inner,
            copy_size: (width, height, depth_or_layers),
        });

        self
    }

    pub fn image_copy_buffer_to_texture<T, F>(
        self,
        src: buffer::ImageCopySrc<T>,
        dst: texture::ImageCopyDst<F>,
    ) -> Self
    where
        T: ImageData<F>,
        F: TextureFormat,
    {
        self.image_copy_buffer_to_texture_internal(src.inner, dst.inner)
    }

    pub fn image_copy_buffer_to_texture_raw<F>(
        self,
        src: buffer::ImageCopySrcRaw,
        dst: texture::ImageCopyDst<F>,
    ) -> Self {
        assert!(
            src.inner.bytes_per_block == dst.inner.bytes_per_block,
            "`src` bytes per block does not match `dst` bytes per block"
        );

        self.image_copy_buffer_to_texture_internal(src.inner, dst.inner)
    }

    fn sub_image_copy_buffer_to_texture_internal<F>(
        mut self,
        src: ImageCopyBuffer<Dvr>,
        dst: texture::ImageCopyTexture<F>,
        size: ImageCopySize3D,
    ) -> Self {
        let ImageCopySize3D {
            width,
            height,
            depth_or_layers,
        } = size;

        size.validate_with_block_size(dst.block_size);
        image_copy_buffer_validate(&src, (width, height, depth_or_layers), dst.block_size);
        dst.validate_dst_with_size(size);

        self.handle.copy_buffer_to_texture(CopyBufferToTexture {
            source: src,
            destination: dst.inner,
            copy_size: (width, height, depth_or_layers),
        });

        self
    }

    pub fn sub_image_copy_buffer_to_texture<T, F>(
        self,
        src: buffer::ImageCopySrc<T>,
        dst: texture::SubImageCopyDst<F>,
        size: ImageCopySize3D,
    ) -> Self
    where
        T: ImageData<F>,
        F: TextureFormat,
    {
        self.sub_image_copy_buffer_to_texture_internal(src.inner, dst.inner, size)
    }

    pub fn sub_image_copy_buffer_to_texture_raw<F>(
        self,
        src: buffer::ImageCopySrcRaw,
        dst: texture::SubImageCopyDst<F>,
        size: ImageCopySize3D,
    ) -> Self {
        assert!(
            src.inner.bytes_per_block == dst.inner.bytes_per_block,
            "`src` bytes per block does not match `dst` bytes per block"
        );

        self.sub_image_copy_buffer_to_texture_internal(src.inner, dst.inner, size)
    }

    fn image_copy_texture_to_buffer_internal<F>(
        mut self,
        src: texture::ImageCopyTexture<F>,
        dst: ImageCopyBuffer<Dvr>,
    ) -> Self {
        let width = src.width;
        let height = src.height;
        let depth_or_layers = src.depth_or_layers;

        image_copy_buffer_validate(&dst, (width, height, depth_or_layers), src.block_size);

        self.handle.copy_texture_to_buffer(CopyTextureToBuffer {
            source: src.inner,
            destination: dst,
            copy_size: (width, height, depth_or_layers),
        });

        self
    }

    pub fn image_copy_texture_to_buffer<F, T>(
        self,
        src: texture::ImageCopySrc<F>,
        dst: buffer::ImageCopyDst<T>,
    ) -> Self
    where
        F: TextureFormat,
        T: ImageData<F>,
    {
        self.image_copy_texture_to_buffer_internal(src.inner, dst.inner)
    }

    pub fn image_copy_texture_to_buffer_raw<F>(
        self,
        src: texture::ImageCopySrc<F>,
        dst: buffer::ImageCopyDstRaw,
    ) -> Self {
        assert!(
            src.inner.bytes_per_block == dst.inner.bytes_per_block,
            "`src` bytes per block does not match `dst` bytes per block"
        );

        self.image_copy_texture_to_buffer_internal(src.inner, dst.inner)
    }

    fn sub_image_copy_texture_to_buffer_internal<F>(
        mut self,
        src: texture::ImageCopyTexture<F>,
        dst: ImageCopyBuffer<Dvr>,
        size: ImageCopySize3D,
    ) -> Self {
        let ImageCopySize3D {
            width,
            height,
            depth_or_layers,
        } = size;

        size.validate_with_block_size(src.block_size);
        src.validate_src_with_size(size);
        image_copy_buffer_validate(&dst, (width, height, depth_or_layers), src.block_size);

        self.handle.copy_texture_to_buffer(CopyTextureToBuffer {
            source: src.inner,
            destination: dst,
            copy_size: (width, height, depth_or_layers),
        });

        self
    }

    pub fn sub_image_copy_texture_to_buffer<F, T>(
        self,
        src: texture::SubImageCopySrc<F>,
        dst: buffer::ImageCopyDst<T>,
        size: ImageCopySize3D,
    ) -> Self
    where
        F: TextureFormat,
        T: ImageData<F>,
    {
        self.sub_image_copy_texture_to_buffer_internal(src.inner, dst.inner, size)
    }

    pub fn sub_image_copy_texture_to_buffer_raw<F>(
        self,
        src: texture::SubImageCopySrc<F>,
        dst: buffer::ImageCopyDstRaw,
        size: ImageCopySize3D,
    ) -> Self {
        assert!(
            src.inner.bytes_per_block == dst.inner.bytes_per_block,
            "`src` bytes per block does not match `dst` bytes per block"
        );

        self.sub_image_copy_texture_to_buffer_internal(src.inner, dst.inner, size)
    }

    pub fn image_copy_texture_to_texture<F>(
        mut self,
        src: texture::ImageCopyToTextureSrc<F>,
        dst: texture::ImageCopyFromTextureDst<F>,
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

        self.handle.copy_texture_to_texture(CopyTextureToTexture {
            source: src.inner.inner,
            destination: dst.inner.inner,
            copy_size: (src.inner.width, src.inner.height, src.inner.depth_or_layers),
        });

        self
    }

    pub fn sub_image_copy_texture_to_texture<F>(
        mut self,
        src: texture::SubImageCopyToTextureSrc<F>,
        dst: texture::SubImageCopyFromTextureDst<F>,
        size: ImageCopySize3D,
    ) -> Self {
        size.validate_with_block_size(src.inner.block_size);
        src.inner.validate_src_with_size(size);
        dst.inner.validate_dst_with_size(size);

        self.handle.copy_texture_to_texture(CopyTextureToTexture {
            source: src.inner.inner,
            destination: dst.inner.inner,
            copy_size: (size.width, size.height, size.depth_or_layers),
        });

        self
    }

    pub fn image_copy_texture_to_texture_multisample<F, const SAMPLES: u8>(
        mut self,
        src: texture::ImageCopyToTextureSrcMultisample<F, SAMPLES>,
        dst: texture::ImageCopyToTextureDstMultisample<F, SAMPLES>,
    ) -> Self {
        assert!(
            src.inner.width == dst.inner.width,
            "`src` and `dst` widths must match"
        );
        assert!(
            src.inner.height == dst.inner.height,
            "`src` and `dst` heights must match"
        );

        self.handle.copy_texture_to_texture(CopyTextureToTexture {
            source: src.inner.inner,
            destination: dst.inner.inner,
            copy_size: (src.inner.width, src.inner.height, src.inner.depth_or_layers),
        });

        self
    }

    pub fn begin_compute_pass(mut self) -> ComputePassEncoder<(), ()> {
        let handle = self.handle.begin_compute_pass();

        ComputePassEncoder {
            handle,
            command_encoder: self,
            current_pipeline_id: None,
            current_bind_group_ids: [None; 4],
            _marker: Default::default(),
        }
    }

    pub fn begin_render_pass<T, Q>(
        mut self,
        descriptor: RenderPassDescriptor<T, Q>,
    ) -> ClearRenderPassEncoder<T::RenderLayout, Q>
    where
        T: ValidRenderTarget,
    {
        let handle = self.handle.begin_render_pass(driver::RenderPassDescriptor {
            color_attachments: descriptor
                .render_target
                .color_target_encodings()
                .into_iter()
                .map(|a| a.inner),
            depth_stencil_attachment: descriptor
                .render_target
                .depth_stencil_target_encoding()
                .inner,
            occlusion_query_set: descriptor.occlusion_query_set,
        });

        RenderPassEncoder {
            handle,
            command_encoder: self,
            current_pipeline_id: None,
            current_vertex_buffers: [None, None, None, None, None, None, None, None],
            current_index_buffer: None,
            current_bind_group_ids: [None; 4],
            _marker: Default::default(),
        }
    }

    pub fn write_timestamp(mut self, query_set: &TimestampQuerySet, index: usize) -> Self {
        assert!(index < query_set.len(), "index out of bounds");

        self.handle.write_timestamp(&query_set.handle, index);

        self
    }

    pub fn resolve_occlusion_query_set<U>(
        mut self,
        query_set: &OcclusionQuerySet,
        offset: usize,
        view: buffer::View<[u64], U>,
    ) -> Self
    where
        U: buffer::QueryResolve,
    {
        let start = offset;
        let end = start + view.len();

        assert!(end <= query_set.len(), "resolve range out of bounds");

        self.handle.resolve_query_set(ResolveQuerySet {
            query_set: &query_set.handle,
            query_range: start..end,
            destination: &view.buffer.handle,
            destination_offset: view.offset_in_bytes(),
        });

        self
    }

    pub fn resolve_timestamp_query_set<U>(
        mut self,
        query_set: &TimestampQuerySet,
        offset: usize,
        view: buffer::View<[u64], U>,
    ) -> Self
    where
        U: buffer::QueryResolve,
    {
        let start = offset;
        let end = start + view.len();

        assert!(end <= query_set.len(), "resolve range out of bounds");

        self.handle.resolve_query_set(ResolveQuerySet {
            query_set: &query_set.handle,
            query_range: start..end,
            destination: &view.buffer.handle,
            destination_offset: view.offset_in_bytes(),
        });

        self
    }

    pub fn finish(self) -> CommandBuffer {
        CommandBuffer {
            handle: self.handle.finish(),
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
    handle: <Dvr as Driver>::ComputePassEncoderHandle,
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
            mut handle,
            command_encoder,
            current_pipeline_id,
            mut current_bind_group_ids,
            ..
        } = self;

        for (i, encoding) in bind_groups.encodings().enumerate() {
            let BindGroupEncoding {
                bind_group_handle,
                id,
            } = encoding;

            if current_bind_group_ids[i] != Some(id) {
                handle.set_bind_group(i as u32, &bind_group_handle);

                current_bind_group_ids[i] = Some(id);
            }
        }

        ComputePassEncoder {
            handle,
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
            mut handle,
            command_encoder,
            current_pipeline_id,
            current_bind_group_ids,
            ..
        } = self;

        if Some(pipeline.id()) != current_pipeline_id {
            handle.set_pipeline(&pipeline.handle);
        }

        ComputePassEncoder {
            handle,
            command_encoder,
            current_pipeline_id: Some(pipeline.id()),
            current_bind_group_ids,
            _marker: Default::default(),
        }
    }

    pub fn end(self) -> CommandEncoder {
        self.handle.end();

        self.command_encoder
    }
}

impl<RLayout, R> ComputePassEncoder<ComputePipeline<RLayout>, R>
where
    R: BindGroups<Layout = RLayout>,
{
    pub fn dispatch_workgroups(mut self, dispatch_workgroups: DispatchWorkgroups) -> Self {
        let DispatchWorkgroups {
            count_x,
            count_y,
            count_z,
        } = dispatch_workgroups;

        self.handle.dispatch_workgroups(count_x, count_y, count_z);

        self
    }

    pub fn dispatch_workgroups_indirect<U>(
        mut self,
        view: buffer::View<DispatchWorkgroups, U>,
    ) -> Self
    where
        U: buffer::Indirect,
    {
        self.handle
            .dispatch_workgroups_indirect(&view.buffer.handle, view.offset_in_bytes());

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

    // TODO: base_vertex is specced to be a signed integer, but specced to be an unsigned
    // integer in the indirect version. Going with unsigned for both for now (what's the
    // use-case for a negative base vertex number?), but should investigate.
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

#[derive(Clone, PartialEq, Eq, Debug)]
struct CurrentBufferRange {
    id: usize,
    range: Range<usize>,
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

pub struct RenderPassDescriptor<'a, RenderTarget, OcclusionQueryState> {
    render_target: &'a RenderTarget,
    occlusion_query_set: Option<&'a <Dvr as Driver>::QuerySetHandle>,
    _marker: marker::PhantomData<OcclusionQueryState>,
}

impl<'a> RenderPassDescriptor<'a, (), ()> {
    pub fn new<T: ValidRenderTarget>(render_target: &'a T) -> RenderPassDescriptor<'a, T, ()> {
        let mut dimensions = None;

        for attachment in render_target.color_target_encodings() {
            if attachment.inner.is_some() {
                if let Some((width, height)) = dimensions {
                    if attachment.width != width || attachment.height != height {
                        panic!("all attachment dimensions must match")
                    }
                } else {
                    dimensions = Some((attachment.width, attachment.height));
                }
            }
        }

        let depth_stencil_attachment = render_target.depth_stencil_target_encoding();

        if depth_stencil_attachment.inner.is_some() {
            if let Some((width, height)) = dimensions {
                if depth_stencil_attachment.width != width
                    || depth_stencil_attachment.height != height
                {
                    panic!("all attachment dimensions must match")
                }
            } else {
                dimensions = Some((
                    depth_stencil_attachment.width,
                    depth_stencil_attachment.height,
                ));
            }
        }

        if dimensions.is_none() {
            panic!(
                "target must specify either at least 1 color attachment or a depth-stencil \
                attachment"
            );
        }

        RenderPassDescriptor {
            render_target,
            occlusion_query_set: None,
            _marker: Default::default(),
        }
    }
}

impl<'a, T> RenderPassDescriptor<'a, T, ()> {
    pub fn occlusion_query_set(
        self,
        occlusion_query_set: &'a OcclusionQuerySet,
    ) -> RenderPassDescriptor<'a, T, OcclusionQueryState<O>> {
        RenderPassDescriptor {
            render_target: self.render_target,
            occlusion_query_set: Some(&occlusion_query_set.handle),
            _marker: Default::default(),
        }
    }
}

pub type ClearRenderPassEncoder<Target, Q> = RenderPassEncoder<Target, (), (), (), (), Q>;

pub struct RenderPassEncoder<Target, Pipeline, Vertex, Index, Resources, OcclusionQueryState> {
    handle: <Dvr as Driver>::RenderPassEncoderHandle,
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
            mut handle,
            command_encoder,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            mut current_bind_group_ids,
            ..
        } = self;

        for (i, encoding) in bind_groups.encodings().enumerate() {
            let BindGroupEncoding {
                bind_group_handle,
                id,
            } = encoding;

            if current_bind_group_ids[i] != Some(id) {
                handle.set_bind_group(i as u32, &bind_group_handle);

                current_bind_group_ids[i] = Some(id);
            }
        }

        RenderPassEncoder {
            handle,
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
            mut handle,
            command_encoder,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            ..
        } = self;

        if Some(pipeline.id()) != current_pipeline_id {
            handle.set_pipeline(&pipeline.handle);
        }

        RenderPassEncoder {
            handle,
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
            mut handle,
            command_encoder,
            current_pipeline_id,
            mut current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            ..
        } = self;

        for (i, encoding) in vertex_buffers.encodings().as_ref().iter().enumerate() {
            let VertexBufferEncoding { buffer, id, range } = encoding;

            let range_id = CurrentBufferRange {
                id: *id,
                range: range.clone(),
            };

            if current_vertex_buffers[i] != Some(range_id.clone()) {
                handle.set_vertex_buffer(SetVertexBuffer {
                    slot: i as u32,
                    buffer_handle: Some(buffer),
                    range: Some(range.clone()),
                });

                current_vertex_buffers[i] = Some(range_id);
            }
        }

        RenderPassEncoder {
            handle,
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
            mut handle,
            command_encoder,
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
            range,
        } = index_buffer.to_encoding();

        let range_id = CurrentBufferRange {
            id,
            range: range.clone(),
        };

        if current_index_buffer != Some(range_id.clone()) {
            handle.set_index_buffer(SetIndexBuffer {
                buffer_handle: &buffer,
                index_format: format,
                range: Some(range),
            });

            current_index_buffer = Some(range_id);
        }

        RenderPassEncoder {
            handle,
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
    pub fn set_viewport(mut self, viewport: &Viewport) -> Self {
        self.handle.set_viewport(viewport);

        self
    }

    pub fn set_scissor_rect(mut self, scissor_rect: &ScissorRect) -> Self {
        self.handle.set_scissor_rect(scissor_rect);

        self
    }

    pub fn set_blend_constant(mut self, blend_constant: &BlendConstant) -> Self {
        self.handle.set_blend_constant(blend_constant);

        self
    }

    pub fn set_stencil_reference(mut self, stencil_reference: u32) -> Self {
        self.handle.set_stencil_reference(stencil_reference);

        self
    }

    pub fn clear_state(self) -> ClearRenderPassEncoder<T, Q> {
        let RenderPassEncoder {
            handle,
            command_encoder,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            ..
        } = self;

        RenderPassEncoder {
            handle,
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
            mut handle,
            command_encoder,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            ..
        } = self;

        let mut encoder = handle.execute_bundles();

        encoder.push_bundle(&render_bundle.handle);
        encoder.finish();

        RenderPassEncoder {
            handle,
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
        B::Item: AsRef<RenderBundle<T>>,
    {
        let RenderPassEncoder {
            mut handle,
            command_encoder,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            ..
        } = self;

        let mut encoder = handle.execute_bundles();

        for bundle in render_bundles.into_iter() {
            encoder.push_bundle(&bundle.as_ref().handle);
        }

        encoder.finish();

        RenderPassEncoder {
            handle,
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
    fn draw(mut self, draw: Draw) -> Self {
        self.handle.draw(draw);

        self
    }

    fn draw_indirect<U>(mut self, view: buffer::View<Draw, U>) -> Self
    where
        U: buffer::Indirect,
    {
        self.handle
            .draw_indirect(&view.buffer.handle, view.offset_in_bytes());

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
    fn draw_indexed(mut self, draw_indexed: DrawIndexed) -> Self {
        self.handle.draw_indexed(draw_indexed);

        self
    }

    fn draw_indexed_indirect<U>(mut self, view: buffer::View<DrawIndexed, U>) -> Self
    where
        U: buffer::Indirect,
    {
        self.handle
            .draw_indexed_indirect(&view.buffer.handle, view.offset_in_bytes());

        self
    }
}

impl<T, P, V, I, R, Q> RenderPassEncoder<T, P, V, I, R, Q>
where
    Q: BeginOcclusionQuery,
{
    pub fn begin_occlusion_query(self, query_index: u32) -> RenderPassEncoder<T, P, V, I, R, X> {
        let RenderPassEncoder {
            mut handle,
            command_encoder,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            ..
        } = self;

        handle.begin_occlusion_query(query_index);

        RenderPassEncoder {
            handle,
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
            mut handle,
            command_encoder,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            ..
        } = self;

        handle.end_occlusion_query();

        RenderPassEncoder {
            handle,
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
        self.handle.end();

        self.command_encoder
    }
}

pub struct RenderBundle<Target> {
    handle: <Dvr as Driver>::RenderBundleHandle,
    _marker: marker::PhantomData<Target>,
}

impl<T> AsRef<RenderBundle<T>> for RenderBundle<T> {
    fn as_ref(&self) -> &RenderBundle<T> {
        self
    }
}

impl<T> AsRef<<Dvr as Driver>::RenderBundleHandle> for RenderBundle<T> {
    fn as_ref(&self) -> &<Dvr as Driver>::RenderBundleHandle {
        &self.handle
    }
}

pub struct RenderBundleEncoderDescriptor<Target> {
    color_formats: Cow<'static, [TextureFormatId]>,
    depth_stencil_format: Option<TextureFormatId>,
    sample_count: u32,
    depth_read_only: bool,
    stencil_read_only: bool,
    _marker: marker::PhantomData<Target>,
}

impl RenderBundleEncoderDescriptor<()> {
    pub fn new<C>() -> RenderBundleEncoderDescriptor<RenderLayout<C, ()>>
    where
        C: TypedColorLayout,
    {
        RenderBundleEncoderDescriptor {
            color_formats: Cow::Borrowed(C::COLOR_FORMATS),
            depth_stencil_format: None,
            sample_count: 1,
            depth_read_only: false,
            stencil_read_only: false,
            _marker: Default::default(),
        }
    }

    pub fn multisample<C, const SAMPLES: u8>(
    ) -> RenderBundleEncoderDescriptor<MultisampleRenderLayout<C, (), SAMPLES>>
    where
        C: TypedMultisampleColorLayout,
    {
        RenderBundleEncoderDescriptor {
            color_formats: Cow::Borrowed(C::COLOR_FORMATS),
            depth_stencil_format: None,
            sample_count: SAMPLES as u32,
            depth_read_only: false,
            stencil_read_only: false,
            _marker: Default::default(),
        }
    }
}

impl<C> RenderBundleEncoderDescriptor<RenderLayout<C, ()>> {
    pub fn depth_stencil_format<Ds>(self) -> RenderBundleEncoderDescriptor<RenderLayout<C, Ds>>
    where
        Ds: DepthStencilRenderable,
    {
        let RenderBundleEncoderDescriptor {
            color_formats,
            sample_count,
            ..
        } = self;

        RenderBundleEncoderDescriptor {
            color_formats,
            depth_stencil_format: Some(Ds::FORMAT_ID),
            sample_count,
            depth_read_only: !Ds::HAS_DEPTH_COMPONENT,
            stencil_read_only: !Ds::HAS_STENCIL_COMPONENT,
            _marker: Default::default(),
        }
    }

    pub fn depth_stencil_format_read_only<Ds>(
        self,
    ) -> RenderBundleEncoderDescriptor<RenderLayout<C, ReadOnly<Ds>>>
    where
        Ds: DepthStencilRenderable,
    {
        let RenderBundleEncoderDescriptor {
            color_formats,
            sample_count,
            ..
        } = self;

        RenderBundleEncoderDescriptor {
            color_formats,
            depth_stencil_format: Some(Ds::FORMAT_ID),
            sample_count,
            depth_read_only: true,
            stencil_read_only: true,
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
        let RenderBundleEncoderDescriptor {
            color_formats,
            sample_count,
            ..
        } = self;

        RenderBundleEncoderDescriptor {
            color_formats,
            depth_stencil_format: Some(Ds::FORMAT_ID),
            sample_count,
            depth_read_only: false,
            stencil_read_only: false,
            _marker: Default::default(),
        }
    }

    pub fn depth_stencil_format_read_only<Ds>(
        self,
    ) -> RenderBundleEncoderDescriptor<MultisampleRenderLayout<C, ReadOnly<Ds>, SAMPLES>>
    where
        Ds: DepthStencilRenderable,
    {
        let RenderBundleEncoderDescriptor {
            color_formats,
            sample_count,
            ..
        } = self;

        RenderBundleEncoderDescriptor {
            color_formats,
            depth_stencil_format: Some(Ds::FORMAT_ID),
            sample_count,
            depth_read_only: true,
            stencil_read_only: true,
            _marker: Default::default(),
        }
    }
}

impl<T> RenderBundleEncoderDescriptor<T> {
    pub(crate) fn to_driver(&self) -> driver::RenderBundleEncoderDescriptor {
        driver::RenderBundleEncoderDescriptor {
            color_formats: self.color_formats.as_ref(),
            depth_stencil_format: self.depth_stencil_format,
            sample_count: self.sample_count,
            depth_read_only: self.depth_read_only,
            stencil_read_only: self.stencil_read_only,
        }
    }
}

pub struct RenderBundleEncoder<Target, Pipeline, Vertex, Index, Resources> {
    handle: <Dvr as Driver>::RenderBundleEncoderHandle,
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
    )>,
}

impl<T, P, V, I, R> RenderBundleEncoder<T, P, V, I, R> {
    pub fn new(device: &Device, descriptor: &RenderBundleEncoderDescriptor<T>) -> Self {
        let handle = device
            .handle
            .create_render_bundle_encoder(&descriptor.to_driver());

        RenderBundleEncoder {
            handle,
            current_pipeline_id: None,
            current_vertex_buffers: [None, None, None, None, None, None, None, None],
            current_index_buffer: None,
            current_bind_group_ids: [None; 4],
            _marker: Default::default(),
        }
    }
}

impl<T, P, V, I, R> RenderBundleEncoder<T, P, V, I, R> {
    pub fn finish(self) -> RenderBundle<T> {
        RenderBundle {
            handle: self.handle.finish(),
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
            mut handle,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            mut current_bind_group_ids,
            ..
        } = self;

        for (i, encoding) in bind_groups.encodings().enumerate() {
            let BindGroupEncoding {
                bind_group_handle,
                id,
            } = encoding;

            if current_bind_group_ids[i] != Some(id) {
                handle.set_bind_group(i as u32, &bind_group_handle);

                current_bind_group_ids[i] = Some(id);
            }
        }

        RenderBundleEncoder {
            handle,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
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
            mut handle,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            ..
        } = self;

        if Some(pipeline.id()) != current_pipeline_id {
            handle.set_pipeline(&pipeline.handle);
        }

        RenderBundleEncoder {
            handle,
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
        let RenderBundleEncoder {
            mut handle,
            current_pipeline_id,
            mut current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
            ..
        } = self;

        for (i, encoding) in vertex_buffers.encodings().as_ref().iter().enumerate() {
            let VertexBufferEncoding { buffer, id, range } = encoding;

            let range_id = CurrentBufferRange {
                id: *id,
                range: range.clone(),
            };

            if current_vertex_buffers[i] != Some(range_id.clone()) {
                handle.set_vertex_buffer(SetVertexBuffer {
                    slot: i as u32,
                    buffer_handle: Some(buffer),
                    range: Some(range.clone()),
                });

                current_vertex_buffers[i] = Some(range_id);
            }
        }

        RenderBundleEncoder {
            handle,
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
        let RenderBundleEncoder {
            mut handle,
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
            range,
        } = index_buffer.to_encoding();

        let range_id = CurrentBufferRange {
            id,
            range: range.clone(),
        };

        if current_index_buffer != Some(range_id.clone()) {
            handle.set_index_buffer(SetIndexBuffer {
                buffer_handle: &buffer,
                index_format: format,
                range: Some(range),
            });

            current_index_buffer = Some(range_id);
        }

        RenderBundleEncoder {
            handle,
            current_pipeline_id,
            current_vertex_buffers,
            current_index_buffer,
            current_bind_group_ids,
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
    fn draw(mut self, draw: Draw) -> Self {
        self.handle.draw(draw);

        self
    }

    fn draw_indirect<U>(mut self, view: buffer::View<Draw, U>) -> Self
    where
        U: buffer::Indirect,
    {
        self.handle
            .draw_indirect(&view.buffer.handle, view.offset_in_bytes());

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
    fn draw_indexed(mut self, draw_indexed: DrawIndexed) -> Self {
        self.handle.draw_indexed(draw_indexed);

        self
    }

    fn draw_indexed_indirect<U>(mut self, view: buffer::View<DrawIndexed, U>) -> Self
    where
        U: buffer::Indirect,
    {
        self.handle
            .draw_indexed_indirect(&view.buffer.handle, view.offset_in_bytes());

        self
    }
}
