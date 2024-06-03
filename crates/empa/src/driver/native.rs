use core::marker;
use std::borrow::{Borrow, Cow};
use std::convert::{AsMut, AsRef};
use std::error::Error;
use std::future::{ready, Future};
use std::num::NonZeroU64;
use std::ops::Range;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::{future, mem, ptr, slice};

use arrayvec::ArrayVec;
use flagset::FlagSet;
use wgc::command::{bundle_ffi, compute_commands, render_commands};
use wgc::gfx_select;
use wgc::global::Global;
use wgc::id::{
    AdapterId, BindGroupId, BindGroupLayoutId, BufferId, CommandBufferId, CommandEncoderId,
    ComputePipelineId, DeviceId, PipelineLayoutId, QuerySetId, QueueId,
    RenderBundleEncoderId, RenderBundleId, RenderPassEncoderId, RenderPipelineId, SamplerId,
    ShaderModuleId, TextureId, TextureViewId,
};

use crate::adapter::{Feature, Limits};
use crate::buffer::MapError;
use crate::command::{BlendConstant, Draw, DrawIndexed, ScissorRect, Viewport};
use crate::device::DeviceDescriptor;
use crate::driver::{Adapter, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsage, ClearBuffer, ColorTargetState, CommandEncoder, ComputePassEncoder, ComputePipelineDescriptor, CopyBufferToBuffer, CopyBufferToTexture, CopyTextureToBuffer, CopyTextureToTexture, DepthStencilOperations, DepthStencilState, Device, ExecuteRenderBundlesEncoder, ImageCopyBuffer, ImageCopyTexture, ImageDataLayout, MapMode, MultisampleState, PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, ProgrammablePassEncoder, QuerySetDescriptor, QueryType, Queue, RenderBundleEncoder, RenderBundleEncoderDescriptor, RenderEncoder, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPassEncoder, RenderPipelineDescriptor, ResolveQuerySet, SamplerBindingType, SamplerDescriptor, SetIndexBuffer, SetVertexBuffer, ShaderStage, StencilFaceState, StencilOperation, StorageTextureAccess, Texture, TextureAspect, TextureDescriptor, TextureDimensions, TextureSampleType, TextureUsage, TextureViewDescriptor, TextureViewDimension, WriteBufferOperation, WriteTextureOperation};
use crate::render_pipeline::{
    BlendComponent, BlendFactor, BlendState, ColorWrite, CullMode, FrontFace, IndexFormat,
    VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode,
};
use crate::sampler::{AddressMode, FilterMode};
use crate::texture::format::TextureFormatId;
use crate::{driver, CompareFunction};
use crate::render_target::{LoadOp, StoreOp};

pub struct Driver;

impl driver::Driver for Driver {
    type AdapterHandle = AdapterHandle;
    type BindGroupHandle = BindGroupHandle;
    type DeviceHandle = DeviceHandle;
    type BufferHandle = BufferHandle;
    type BufferBinding = wgc::binding_model::BufferBinding;
    type TextureHandle = TextureHandle;
    type TextureView = TextureView;
    type CommandEncoderHandle = CommandEncoderHandle;
    type ComputePassEncoderHandle = ComputePassEncoderHandle;
    type RenderPassEncoderHandle = RenderPassEncoderHandle;
    type ExecuteRenderBundlesEncoder<'a> = ExecuteRenderBundlesEncoderHandle<'a>;
    type RenderBundleEncoderHandle = RenderBundleEncoderHandle;
    type CommandBufferHandle = CommandBufferHandle;
    type RenderBundleHandle = RenderBundleHandle;
    type QueueHandle = QueueHandle;
    type SamplerHandle = SamplerHandle;
    type BindGroupLayoutHandle = BindGroupLayoutHandle;
    type PipelineLayoutHandle = PipelineLayoutHandle;
    type ComputePipelineHandle = ComputePipelineHandle;
    type RenderPipelineHandle = RenderPipelineHandle;
    type QuerySetHandle = QuerySetHandle;
    type ShaderModuleHandle = ShaderModuleHandle;
}

#[derive(Clone)]
pub struct AdapterHandle {
    global: Arc<Global>,
    id: AdapterId,
}

impl Adapter<Driver> for AdapterHandle {
    type RequestDevice = future::Ready<Result<DeviceHandle, Box<dyn Error>>>;

    fn supported_features(&self) -> FlagSet<Feature> {
        let features: Result<_, wgc::instance::InvalidAdapter> =
            gfx_select!(self.id => self.global.adapter_features(self.id));

        match features {
            Ok(features) => features_from_wgc(features),
            Err(err) => panic!("{}", err),
        }
    }

    fn supported_limits(&self) -> Limits {
        let limits: Result<_, wgc::instance::InvalidAdapter> =
            gfx_select!(self.id => self.global.adapter_limits(self.id));

        match limits {
            Ok(features) => limits_from_wgc(features),
            Err(err) => panic!("{}", err),
        }
    }

    fn request_device<Flags>(&self, descriptor: &DeviceDescriptor<Flags>) -> Self::RequestDevice
    where
        Flags: Into<FlagSet<Feature>> + Copy,
    {
        let (device_id, queue_id, error): (_, _, Option<wgc::instance::RequestDeviceError>) = gfx_select!(self.id => self.global.adapter_request_device(
            self.id,
            &wgc::device::DeviceDescriptor {
                label: None,
                required_features: features_to_wgc(&descriptor.features.into()),
                required_limits: limits_to_wgc(&descriptor.limits.into()),
            },
            None,
            None,
            None
        ));

        if let Some(err) = error {
            return ready(Err(err.into()));
        }

        let device_handle = DeviceHandle {
            global: self.global.clone(),
            device_id,
            queue_id,
        };

        ready(Ok(device_handle))
    }
}

impl Drop for AdapterHandle {
    fn drop(&mut self) {
        gfx_select!(self.id => self.global.adapter_drop(self.id));
    }
}

#[derive(Clone)]
pub struct BindGroupHandle {
    global: Arc<Global>,
    id: BindGroupId,
}

impl Drop for BindGroupHandle {
    fn drop(&mut self) {
        gfx_select!(self.id => self.global.bind_group_drop(self.id));
    }
}

#[derive(Clone)]
pub struct DeviceHandle {
    global: Arc<Global>,
    device_id: DeviceId,
    queue_id: QueueId,
}

impl Device<Driver> for DeviceHandle {
    type CreateComputePipelineAsync = future::Ready<ComputePipelineHandle>;
    type CreateRenderPipelineAsync = future::Ready<RenderPipelineHandle>;

    fn create_buffer(&self, descriptor: &BufferDescriptor) -> BufferHandle {
        let descriptor = wgc::resource::BufferDescriptor {
            label: None,
            size: descriptor.size as u64,
            usage: buffer_usage_to_wgc(&descriptor.usage_flags),
            mapped_at_creation: descriptor.mapped_at_creation,
        };

        let (id, err): (_, Option<wgc::resource::CreateBufferError>) = gfx_select!(self.device_id => self.0.device_create_buffer(
            self.device_id,
            &descriptor,
            None
        ));

        if let Some(err) = err {
            panic!("{}", err)
        }

        BufferHandle {
            global: self.global.clone(),
            id,
        }
    }

    fn create_texture(&self, descriptor: &TextureDescriptor) -> TextureHandle {
        let view_formats = descriptor
            .view_formats
            .iter()
            .map(|f| texture_format_to_wgc(f))
            .collect::<Vec<_>>();
        let descriptor = wgc::resource::TextureDescriptor {
            label: None,
            size: size_3d_to_wgc(&descriptor.size),
            mip_level_count: descriptor.mipmap_levels,
            sample_count: descriptor.sample_count,
            dimension: texture_dimension_to_wgc(&descriptor.dimensions),
            format: texture_format_to_wgc(&descriptor.format),
            usage: texture_usage_to_wgc(&descriptor.usage_flags),
            view_formats,
        };

        let (id, err): (_, Option<wgc::resource::CreateTextureError>) = gfx_select!(self.device_id => self.0.device_create_texture(
            self.device_id,
            &descriptor,
            None
        ));

        if let Some(err) = err {
            panic!("{}", err)
        }

        TextureHandle {
            global: self.global.clone(),
            id,
        }
    }

    fn create_sampler(&self, descriptor: &SamplerDescriptor) -> SamplerHandle {
        let descriptor = wgc::resource::SamplerDescriptor {
            label: None,
            address_modes: [
                address_mode_to_wgc(&descriptor.address_mode_u),
                address_mode_to_wgc(&descriptor.address_mode_v),
                address_mode_to_wgc(&descriptor.address_mode_w),
            ],
            mag_filter: filter_mode_to_wgc(&descriptor.magnification_filter),
            min_filter: filter_mode_to_wgc(&descriptor.minification_filter),
            mipmap_filter: filter_mode_to_wgc(&descriptor.mipmap_filter),
            lod_min_clamp: *descriptor.lod_clamp.start(),
            lod_max_clamp: *descriptor.lod_clamp.end(),
            compare: descriptor.compare.as_ref().map(compare_function_to_wgc),
            anisotropy_clamp: descriptor.max_anisotropy,
            border_color: None,
        };

        let (id, err): (_, Option<wgc::resource::CreateSamplerError>) = gfx_select!(self.device_id => self.0.device_create_sampler(
            self.device_id,
            &descriptor,
            None
        ));

        if let Some(err) = err {
            panic!("{}", err)
        }

        SamplerHandle { global: self.global.clone(), id }
    }

    fn create_bind_group_layout<I>(
        &self,
        descriptor: BindGroupLayoutDescriptor<I>,
    ) -> BindGroupLayoutHandle
    where
        I: IntoIterator<Item = BindGroupLayoutEntry>,
    {
        let entries = descriptor
            .entries
            .into_iter()
            .map(bind_group_layout_entry_to_wgc)
            .collect::<Vec<_>>();
        let descriptor = wgc::binding_model::BindGroupLayoutDescriptor {
            label: None,
            entries: entries.into(),
        };

        let (id, err): (_, Option<wgc::binding_model::CreateBindGroupLayoutError>) = gfx_select!(
            self.device_id => self.0.device_create_bind_group_layout(self.device_id, &descriptor, None)
        );

        if let Some(err) = err {
            panic!("{}", err)
        }

        BindGroupLayoutHandle { global: self.global.clone(), id }
    }

    fn create_pipeline_layout<I>(
        &self,
        descriptor: PipelineLayoutDescriptor<I>,
    ) -> PipelineLayoutHandle
    where
        I: IntoIterator,
        I::Item: Borrow<BindGroupLayoutHandle>,
    {
        let ids: ArrayVec<_, { wgc::MAX_BIND_GROUPS }> = descriptor
            .bind_group_layouts
            .into_iter()
            .map(|h| h.borrow().id)
            .collect();

        let descriptor = wgc::binding_model::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: ids.as_slice().into(),
            push_constant_ranges: (&[]).into(),
        };

        let (id, err): (_, Option<wgc::binding_model::CreatePipelineLayoutError>) = gfx_select!(self.device_id => self.global.device_create_pipeline_layout(
            self.device_id,
            &descriptor,
            None
        ));

        if let Some(err) = err {
            panic!("{}", err)
        }

        PipelineLayoutHandle { global: self.global.clone(), id }
    }

    fn create_bind_group<'a, E>(
        &self,
        descriptor: BindGroupDescriptor<Driver, E>,
    ) -> BindGroupHandle
    where
        E: IntoIterator<Item = BindGroupEntry<'a, Driver>>,
    {
        let entries: Vec<_> = descriptor
            .entries
            .into_iter()
            .map(bind_group_entry_to_wgc)
            .collect();

        let descriptor = wgc::binding_model::BindGroupDescriptor {
            label: None,
            layout: descriptor.layout.id,
            entries: entries.into(),
        };

        let (id, err): (_, Option<wgc::binding_model::CreatePipelineLayoutError>) = gfx_select!(self.device_id => self.global.device_create_bind_group(
            self.device_id,
            &descriptor,
            None
        ));

        if let Some(err) = err {
            panic!("{}", err)
        }

        BindGroupHandle { global: self.global.clone(), id }
    }

    fn create_query_set(&self, descriptor: &QuerySetDescriptor) -> QuerySetHandle {
        let descriptor = wgc::resource::QuerySetDescriptor {
            label: None,
            ty: query_type_to_wgc(&descriptor.query_type),
            count: descriptor.len as u32,
        };

        let (id, err): (_, Option<wgc::resource::CreateQuerySetError>) = gfx_select!(self.device_id => self.global.device_create_bind_group(
            self.device_id,
            &descriptor,
            None
        ));

        if let Some(err) = err {
            panic!("{}", err)
        }

        QuerySetHandle { global: self.global.clone(), id }
    }

    fn create_shader_module(&self, source: &str) -> ShaderModuleHandle {
        let descriptor = wgc::pipeline::ShaderModuleDescriptor {
            label: None,
            shader_bound_checks: Default::default(),
        };

        let (id, err): (_, Option<wgc::resource::CreateQuerySetError>) = gfx_select!(self.device_id => self.global.device_create_shader_module(
            self.device_id,
            &descriptor,
            wgc::pipeline::ShaderModuleSource::Wgsl(source.into()),
            None
        ));

        if let Some(err) = err {
            panic!("{}", err)
        }

        ShaderModuleHandle { global: self.global.clone(), id }
    }

    fn create_compute_pipeline(
        &self,
        descriptor: &ComputePipelineDescriptor<Driver>,
    ) -> ComputePipelineHandle {
        let descriptor = wgc::pipeline::ComputePipelineDescriptor {
            label: None,
            layout: Some(descriptor.layout.id),
            stage: wgc::pipeline::ProgrammableStageDescriptor {
                module: descriptor.shader_module.id,
                entry_point: Some(descriptor.entry_point.into()),
                constants: Cow::Borrowed(descriptor.constants),
                zero_initialize_workgroup_memory: true,
            },
        };

        let (id, err): (_, Option<wgc::pipeline::CreateComputePipelineError>) = gfx_select!(self.device_id => self.global.device_create_compute_pipeline(
            self.device_id,
            &descriptor,
            None,
            None
        ));

        if let Some(err) = err {
            panic!("{}", err)
        }

        ComputePipelineHandle { global: self.global.clone(),id }
    }

    fn create_compute_pipeline_async(
        &self,
        descriptor: &ComputePipelineDescriptor<Driver>,
    ) -> Self::CreateComputePipelineAsync {
        future::ready(self.create_compute_pipeline(descriptor))
    }

    fn create_render_pipeline(
        &self,
        descriptor: &RenderPipelineDescriptor<Driver>,
    ) -> RenderPipelineHandle {
        let vertex_buffers: ArrayVec<_, { wgc::MAX_VERTEX_BUFFERS }> = descriptor
            .vertex_state
            .vertex_buffer_layouts
            .iter()
            .map(vertex_buffer_layout_to_wgc)
            .collect();
        let mut targets: ArrayVec<_, { wgc::MAX_COLOR_ATTACHMENTS }> = ArrayVec::new();

        let descriptor = wgc::pipeline::RenderPipelineDescriptor {
            label: None,
            layout: Some(descriptor.layout.id),
            vertex: wgc::pipeline::VertexState {
                stage: wgc::pipeline::ProgrammableStageDescriptor {
                    module: descriptor.vertex_state.shader_module.id,
                    entry_point: Some(descriptor.vertex_state.entry_point.into()),
                    constants: Cow::Borrowed(descriptor.vertex_state.constants),
                    zero_initialize_workgroup_memory: true,
                },
                buffers: vertex_buffers.as_slice().into(),
            },
            primitive: primitive_state_to_wgc(descriptor.primitive_state),
            depth_stencil: descriptor
                .depth_stencil_state
                .map(depth_stencil_state_to_wgc),
            multisample: descriptor
                .multisample_state
                .map(multisample_state_to_wgc)
                .unwrap_or_default(),
            fragment: descriptor.fragment_state.as_ref().map(|s| {
                for target in s.targets {
                    targets.push(Some(color_target_state_to_wgc(target)));
                }

                wgc::pipeline::FragmentState {
                    stage: wgc::pipeline::ProgrammableStageDescriptor {
                        module: s.shader_module.id,
                        entry_point: Some(s.entry_point.into()),
                        constants: Cow::Borrowed(s.constants),
                        zero_initialize_workgroup_memory: true,
                    },
                    targets: targets.as_slice().into(),
                }
            }),
            multiview: None,
        };

        let (id, err): (_, Option<wgc::pipeline::CreateRenderPipelineError>) = gfx_select!(self.device_id => self.global.device_create_render_pipeline(
            self.device_id,
            &descriptor,
            None,
            None
        ));

        if let Some(err) = err {
            panic!("{}", err)
        }

        RenderPipelineHandle { global: self.global.clone(),id }
    }

    fn create_render_pipeline_async(
        &self,
        descriptor: &RenderPipelineDescriptor<Driver>,
    ) -> Self::CreateRenderPipelineAsync {
        future::ready(self.create_render_pipeline(descriptor))
    }

    fn create_command_encoder(&self) -> CommandEncoderHandle {
        let (id, err): (_, Option<wgc::device::DeviceError>) = gfx_select!(self.device_id => self.0.device_create_command_encoder(
            self.device_id,
            None,
            None
        ));

        if let Some(err) = err {
            panic!("{}", err)
        }

        CommandEncoderHandle {
            global: self.global.clone(),
            id,
        }
    }

    fn create_render_bundle_encoder(
        &self,
        descriptor: &RenderBundleEncoderDescriptor,
    ) -> RenderBundleEncoderHandle {
        let color_formats: ArrayVec<_, { wgc::MAX_COLOR_ATTACHMENTS }> = descriptor
            .color_formats
            .iter()
            .map(texture_format_to_wgc)
            .map(Some)
            .collect();

        let descriptor = wgc::command::RenderBundleEncoderDescriptor {
            label: None,
            color_formats: color_formats.as_slice().into(),
            depth_stencil: descriptor
                .depth_stencil_format
                .map(|f| wgt::RenderBundleDepthStencil {
                    format: texture_format_to_wgc(&f),
                    depth_read_only: descriptor.depth_read_only,
                    stencil_read_only: descriptor.stencil_read_only,
                }),
            sample_count: descriptor.sample_count,
            multiview: None,
        };

        let encoder = match wgc::command::RenderBundleEncoder::new(&descriptor, self.device_id, None) {
            Ok(encoder) => encoder,
            Err(err) => panic!("{}", err),
        };

        RenderBundleEncoderHandle {
            global: self.global.clone(),
            bundle: encoder,
        }
    }

    fn queue_handle(&self) -> QueueHandle {
        QueueHandle {
            global: self.global.clone(),
            id: self.queue_id,
        }
    }
}

impl Drop for DeviceHandle {
    fn drop(&mut self) {
        let _ = wgc::gfx_select!(self.device_id => self.global.device_poll(self.device_id, wgt::Maintain::wait()));

        gfx_select!(self.device_id => self.global.device_drop(self.device_id));
    }
}

#[derive(Clone)]
pub struct BufferHandle {
    global: Arc<Global>,
    id: BufferId,
}

impl Buffer<Driver> for BufferHandle {
    type Map = Map;
    type Mapped<'a, E: 'a> = &'a [E];
    type MappedMut<'a, E: 'a> = &'a mut [E];

    fn map(&self, mode: MapMode, range: Range<usize>) -> Map {
        Map {
            global: self.global.clone(),
            buffer_id: self.id,
            host: map_mode_to_wgc(&mode),
            range: Some(range),
            status: None,
        }
    }

    // Note on the safety of `mapped` and `mapped_mut`. Returning slices here is convenient, although this code in
    // itself does not guarantee that the slice pointer won't be dangling after the buffer is unmapped. However,
    // the rest of empa should never (successfully) unmap the buffer while there is still a live mapped range.

    fn mapped<'a, E>(&'a self, offset_in_bytes: usize, len_in_elements: usize) -> &[E] {
        let size = len_in_elements * mem::size_of::<E>();

        let res: Result<(*mut u8, u64), wgc::resource::BufferAccessError> = gfx_select!(self.id => self.global.buffer_get_mapped_range(
            self.id,
            offset_in_bytes as u64,
            NoneZeroU64::new(size),
        ));

        match res {
            Ok((ptr, mapped_size)) => {
                assert_eq!(mapped_size, size as u64);

                let ptr = ptr as *const E;

                unsafe { slice::from_raw_parts(ptr, len_in_elements) }
            }
            Err(err) => {
                panic!("{}", err)
            }
        }
    }

    fn mapped_mut<'a, E>(&'a self, offset_in_bytes: usize, len_in_elements: usize) -> &mut [E] {
        let size = len_in_elements * mem::size_of::<E>();

        let res: Result<(*mut u8, u64), wgc::resource::BufferAccessError> = gfx_select!(self.id => self.global.buffer_get_mapped_range(
            self.id,
            offset_in_bytes as u64,
            NoneZeroU64::new(size),
        ));

        match res {
            Ok((ptr, mapped_size)) => {
                assert_eq!(mapped_size, size as u64);

                let ptr = ptr as *mut E;

                unsafe { slice::from_raw_parts_mut(ptr, len_in_elements) }
            }
            Err(err) => {
                panic!("{}", err)
            }
        }
    }

    fn unmap(&self) {
        let res: wgc::resource::BufferAccessResult =
            gfx_select!(self.id => self.global.buffer_unmap(self.id));

        if let Err(err) = res {
            panic!("{}", err);
        }
    }

    fn binding(&self, offset: usize, size: usize) -> wgc::binding_model::BufferBinding {
        wgc::binding_model::BufferBinding {
            buffer_id: self.id,
            offset: offset as u64,
            size: NonZeroU64::new(size as u64),
        }
    }
}

impl Drop for BufferHandle {
    fn drop(&mut self) {
        gfx_select!(self.id => self.global.buffer_drop(self.id));
    }
}

/// Helper type to facilitate sending a pointer across threads.
struct StatusPtr(*mut Option<wgc::resource::BufferAccessResult>);

// SAFETY: private type only used in the `Future` implementation for `Map` (see below).
unsafe impl Sync for StatusPtr {}
unsafe impl Send for StatusPtr {}

#[must_use = "futures do nothing if they are not polled"]
pub struct Map {
    global: Arc<Global>,
    buffer_id: BufferId,
    host: wgc::device::HostMap,
    range: Option<Range<usize>>,
    status: Option<wgc::resource::BufferAccessResult>,
}

impl Future for Map {
    type Output = Result<(), MapError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        if let Some(range) = this.range.take() {
            // First poll, initialize the task and return `Pending`.

            let offset = range.start as u64;
            let size = range.len() as u64;
            let status_ptr = StatusPtr(&mut this.status as *mut _);

            let mut waker = Some(cx.waker().clone());

            let callback = wgc::resource::BufferMapCallback::from_rust(Box::new(move |status| {
                if let Some(waker) = waker.take() {
                    // Move the entire wrapper into into the closure, otherwise a partial move happens of only the
                    // pointer (and the compiler will complain about the pointer not being `Send` and `Sync`).
                    let status_ptr = status_ptr;

                    unsafe {
                        // SAFETY: safe because `self` is pinned so the pointer will be valid, and the callback only
                        // runs once, so we will not alias the address.
                        *status_ptr.0 = Some(status);
                    }

                    waker.wake();
                }
            }));

            gfx_select!(this.buffer_id =>
                this.global.buffer_map_async(
                    this.buffer_id,
                    offset,
                    Some(size),
                    wgc::resource::BufferMapOperation {
                        host: self.host,
                        callback: Some(callback),
                    },
                )
            );

            return Poll::Pending;
        }

        if let Some(status) = this.status.as_ref() {
            if status.is_err() {
                Poll::Ready(Err(MapError))
            } else {
                Poll::Ready(Ok(()))
            }
        } else {
            Poll::Pending
        }
    }
}

#[derive(Clone)]
pub struct BufferBinding {
    binding: wgc::binding_model::BufferBinding,
}

#[derive(Clone)]
pub struct TextureHandle {
    global: Arc<Global>,
    id: TextureId,
}

impl Texture<Driver> for TextureHandle {
    fn texture_view(&self, descriptor: &TextureViewDescriptor) -> TextureView {
        let descriptor = wgc::resource::TextureViewDescriptor {
            label: None,
            format: Some(texture_format_to_wgc(&descriptor.format)),
            dimension: Some(texture_view_dimension_to_wgc(&descriptor.dimensions)),
            range: wgt::ImageSubresourceRange {
                aspect: texture_aspect_to_wgc(&descriptor.aspect),
                base_mip_level: descriptor.mip_levels.start,
                mip_level_count: Some(descriptor.mip_levels.len() as u32),
                base_array_layer: descriptor.layers.start,
                array_layer_count: Some(descriptor.layers.len() as u32),
            },
        };
        let (id, err): (_, Option<wgc::resource::CreateTextureViewError>) = gfx_select!(
            self.id => self.global.texture_create_view(self.id, &descriptor, None)
        );

        if let Some(err) = err {
            panic!("{}", err)
        }

        TextureView {
            id,
        }
    }
}

impl Drop for TextureHandle {
    fn drop(&mut self) {
        gfx_select!(self.id => self.global.texture_drop(self.id));
    }
}

#[derive(Clone)]
pub struct TextureView {
    id: TextureViewId,
}

impl Drop for TextureView {
    fn drop(&mut self) {
        gfx_select!(self.id => self.global.texture_view_drop(self.id));
    }
}

#[derive(Clone)]
pub struct CommandEncoderHandle {
    global: Arc<Global>,
    id: CommandEncoderId,
}

impl CommandEncoder<Driver> for CommandEncoderHandle {
    fn copy_buffer_to_buffer(&mut self, op: CopyBufferToBuffer<Driver>) {
         let res: Result<(), wgc::command::CopyError>= gfx_select!(self.id => self.global.command_encoder_copy_buffer_to_buffer(
            self.id,
            op.source.id,
            op.source_offset as u64,
            op.destination.id,
            op.destination_offset as u64,
            op.size as u64
        ));

        if let Err(err) = res {
            panic!("{}", err)
        }
    }

    fn copy_buffer_to_texture(&mut self, op: CopyBufferToTexture<Driver>) {
        let res: Result<(), wgc::command::CopyError>= gfx_select!(self.id => self.global.command_encoder_copy_buffer_to_texture(
            self.id,
            &image_copy_buffer_to_wgc(&op.source),
            &image_copy_texture_to_wgc(&op.destination),
            &size_3d_to_wgc(&op.size)
        ));

        if let Err(err) = res {
            panic!("{}", err)
        }
    }

    fn copy_texture_to_buffer(&mut self, op: CopyTextureToBuffer<Driver>) {
        let res: Result<(), wgc::command::CopyError>= gfx_select!(self.id => self.global.command_encoder_copy_texture_to_buffer(
            self.id,
            &image_copy_texture_to_wgc(&op.source),
            &image_copy_buffer_to_wgc(&op.destination),
            &size_3d_to_wgc(&op.size)
        ));

        if let Err(err) = res {
            panic!("{}", err)
        }
    }

    fn copy_texture_to_texture(&mut self, op: CopyTextureToTexture<Driver>) {
        let res: Result<(), wgc::command::CopyError>= gfx_select!(self.id => self.global.command_encoder_copy_texture_to_texture(
            self.id,
            &image_copy_texture_to_wgc(&op.source),
            &image_copy_texture_to_wgc(&op.destination),
            &size_3d_to_wgc(&op.size)
        ));

        if let Err(err) = res {
            panic!("{}", err)
        }
    }

    fn clear_buffer(&mut self, op: ClearBuffer<Driver>) {
        let res: Result<(), wgc::command::ClearError>= gfx_select!(self.id => self.global.command_encoder_clear_buffer(
            self.id,
            op.buffer.id,
            op.range.start as u64,
            Some(op.range.len() as u64)
        ));

        if let Err(err) = res {
            panic!("{}", err)
        }
    }

    fn begin_compute_pass(&mut self) -> ComputePassEncoderHandle {
        ComputePassEncoderHandle {
            global: self.global.clone(),
            compute_pass: wgc::command::ComputePass::new(self.id, &wgc::command::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            }),
        }
    }

    fn begin_render_pass<I>(
        &mut self,
        descriptor: RenderPassDescriptor<Driver, I>,
    ) -> RenderPassEncoderHandle
    where
        I: IntoIterator<Item = Option<RenderPassColorAttachment<Driver>>>,
    {
        let color_attachments: ArrayVec<_, {wgc::MAX_COLOR_ATTACHMENTS}> = descriptor.color_attachments.into_iter().map(|a| a.map(render_pass_color_attachment_to_wgc)).collect();

        RenderPassEncoderHandle {
            global: self.global.clone(),
            render_pass: wgc::command::RenderPass::new(self.id, &wgc::command::RenderPassDescriptor {
                label: None,
                color_attachments: color_attachments.as_slice().into(),
                depth_stencil_attachment: descriptor.depth_stencil_attachment.as_ref().map(render_pass_depth_stencil_attachment_to_wgc).as_ref(),
                timestamp_writes: None,
                occlusion_query_set: descriptor.occlusion_query_set.map(|s| s.id),
            }),
        }
    }

    fn write_timestamp(&mut self, query_set: &QuerySetHandle, index: usize) {
        let res: Result<(), wgc::command::QueryError>= gfx_select!(self.id => self.global.command_encoder_write_timestamp(
            self.id,
            query_set.id,
            index as u32,
        ));

        if let Err(err) = res {
            panic!("{}", err)
        }
    }

    fn resolve_query_set(&mut self, op: ResolveQuerySet<Driver>) {
        let res: Result<(), wgc::command::QueryError>= gfx_select!(self.id => self.global.command_encoder_resolve_query_set(
            self.id,
            op.query_set.id,
            op.query_range.start as u32,
            op.query_range.len() as u32,
            op.destination.id,
            op.destination_offset as u64
        ));

        if let Err(err) = res {
            panic!("{}", err)
        }
    }

    fn finish(self) -> CommandBufferHandle {
        let res: Result<CommandBufferId, wgc::command::CommandEncoderError>= gfx_select!(self.id => self.global.command_encoder_finish(
            self.id,
            &wgt::CommandBufferDescriptor {
                label: None
            }
        ));

        match res {
            Ok(id) => CommandBufferHandle {
                global: self.global.clone(),
                id,
            },
            Err(err) => {
                panic!("{}", err)
            }
        }
    }
}

impl Drop for CommandEncoderHandle {
    fn drop(&mut self) {
        gfx_select!(self.id => self.global.command_encoder_drop(self.id));
    }
}

pub struct ComputePassEncoderHandle {
    global: Arc<Global>,
    compute_pass: wgc::command::ComputePass,
}

impl ProgrammablePassEncoder<Driver> for ComputePassEncoderHandle {
    fn set_bind_group(&mut self, index: u32, handle: &BindGroupHandle) {
        compute_commands::wgpu_compute_pass_set_bind_group(&mut self.compute_pass, index, handle.id, &[]);
    }
}

impl ComputePassEncoder<Driver> for ComputePassEncoderHandle {
    fn set_pipeline(&mut self, handle: &ComputePipelineHandle) {
        compute_commands::wgpu_compute_pass_set_pipeline(&mut self.compute_pass, handle.id);
    }

    fn dispatch_workgroups(&mut self, x: u32, y: u32, z: u32) {
        compute_commands::wgpu_compute_pass_dispatch_workgroups(
            &mut self.compute_pass,
            x,
            y,
            z
        );
    }

    fn dispatch_workgroups_indirect(&mut self, buffer_handle: &BufferHandle, offset: usize) {
        compute_commands::wgpu_compute_pass_dispatch_workgroups_indirect(&mut self.compute_pass, buffer_handle.id, offset as u64);
    }

    fn end(self) {
        let encoder_id = self.compute_pass.parent_id();

        let res: Result<(), wgc::command::ComputePassError>= gfx_select!(encoder_id => self.global.command_encoder_run_compute_pass(
            encoder_id,
            &self.compute_pass,
        ));

        if let Err(err) = res {
            panic!("{}", err)
        }
    }
}

pub struct RenderPassEncoderHandle {
    global: Arc<Global>,
    render_pass: wgc::command::RenderPass,
}

impl ProgrammablePassEncoder<Driver> for RenderPassEncoderHandle {
    fn set_bind_group(&mut self, index: u32, handle: &BindGroupHandle) {
        render_commands::wgpu_render_pass_set_bind_group(&mut self.render_pass, index, handle.id, &[]);
    }
}

impl RenderEncoder<Driver> for RenderPassEncoderHandle {
    fn set_pipeline(&mut self, handle: &RenderPipelineHandle) {
        render_commands::wgpu_render_pass_set_pipeline(&mut self.render_pass, handle.id);
    }

    fn set_index_buffer(&mut self, op: SetIndexBuffer<Driver>) {
        render_commands::wgpu_render_pass_set_index_buffer(
            &mut self.render_pass,
            op.buffer_handle.id,
            index_format_to_wgc(&op.index_format),
            op.range.as_ref().map(|r| r.start).unwrap_or(0) as u64,
            op.range.as_ref().and_then(|r| NonZeroU64::new(r.len() as u64))
        );
    }

    fn set_vertex_buffer(&mut self, op: SetVertexBuffer<Driver>) {
        render_commands::wgpu_render_pass_set_vertex_buffer(
            &mut self.render_pass,
            op.slot,
            op.buffer_handle.id,
            op.range.as_ref().map(|r| r.start).unwrap_or(0) as u64,
            op.range.as_ref().and_then(|r| NonZeroU64::new(r.len() as u64))
        );
    }

    fn draw(&mut self, op: Draw) {
        render_commands::wgpu_render_pass_draw(&mut self.render_pass, op.vertex_count, op.instance_count, op.first_vertex, op.first_instance);
    }

    fn draw_indexed(&mut self, op: DrawIndexed) {
        render_commands::wgpu_render_pass_draw_indexed(&mut self.render_pass, op.index_count, op.instance_count, op.first_index, op.base_vertex as i32, op.first_instance);
    }

    fn draw_indirect(&mut self, buffer_handle: &BufferHandle, offset: usize) {
        render_commands::wgpu_render_pass_draw_indirect(&mut self.render_pass, buffer_handle.id, offset as u64);
    }

    fn draw_indexed_indirect(&mut self, buffer_handle: &BufferHandle, offset: usize) {
        render_commands::wgpu_render_pass_draw_indexed_indirect(&mut self.render_pass, buffer_handle.id, offset as u64);
    }
}

impl RenderPassEncoder<Driver> for RenderPassEncoderHandle {
    fn set_viewport(&mut self, viewport: &Viewport) {
        render_commands::wgpu_render_pass_set_viewport(&mut self.render_pass, viewport.x, viewport.y, viewport.width, viewport.height, viewport.min_depth, viewport.max_depth);
    }

    fn set_scissor_rect(&mut self, scissor_rect: &ScissorRect) {
        render_commands::wgpu_render_pass_set_scissor_rect(&mut self.render_pass, scissor_rect.x, scissor_rect.y, scissor_rect.width, scissor_rect.height);
    }

    fn set_blend_constant(&mut self, blend_constant: &BlendConstant) {
        render_commands::wgpu_render_pass_set_blend_constant(&mut self.render_pass, &wgt::Color {
            r: blend_constant.r as f64,
            g: blend_constant.g as f64,
            b: blend_constant.b as f64,
            a: blend_constant.a as f64,
        });
    }

    fn set_stencil_reference(&mut self, stencil_reference: u32) {
        render_commands::wgpu_render_pass_set_stencil_reference(&mut self.render_pass, stencil_reference);
    }

    fn begin_occlusion_query(&mut self, query_index: u32) {
        render_commands::wgpu_render_pass_begin_occlusion_query(&mut self.render_pass, query_index);
    }

    fn end_occlusion_query(&mut self) {
        render_commands::wgpu_render_pass_end_occlusion_query(&mut self.render_pass);
    }

    fn execute_bundles(&mut self) -> ExecuteRenderBundlesEncoderHandle {
        ExecuteRenderBundlesEncoderHandle {
            render_pass: &mut self.render_pass,
            bundle_ids: vec![],
        }
    }

    fn end(self) {
        let encoder_id = self.render_pass.parent_id();

        let res: Result<(), wgc::command::RenderPassError>= gfx_select!(encoder_id => self.global.command_encoder_run_render_pass(
            encoder_id,
            &self.render_pass,
        ));

        if let Err(err) = res {
            panic!("{}", err)
        }
    }
}

pub struct ExecuteRenderBundlesEncoderHandle<'a> {
    render_pass: &'a mut wgc::command::RenderPass,
    bundle_ids: Vec<RenderBundleId>,
}

impl ExecuteRenderBundlesEncoder<Driver> for ExecuteRenderBundlesEncoderHandle<'_> {
    fn push_bundle(&mut self, bundle: &RenderBundleHandle) {
        self.bundle_ids.push(bundle.id);
    }

    fn finish(self) {
        render_commands::wgpu_render_pass_execute_bundles(self.render_pass, &self.bundle_ids);
    }
}

pub struct RenderBundleEncoderHandle {
    global: Arc<Global>,
    bundle: wgc::command::RenderBundleEncoder,
}

impl ProgrammablePassEncoder<Driver> for RenderBundleEncoderHandle {
    fn set_bind_group(&mut self, index: u32, handle: &BindGroupHandle) {
        unsafe {
            bundle_ffi::wgpu_render_bundle_set_bind_group(&mut self.bundle, index, handle.id, ptr::null(), 0);
        }
    }
}

impl RenderEncoder<Driver> for RenderBundleEncoderHandle {
    fn set_pipeline(&mut self, handle: &RenderPipelineHandle) {
        bundle_ffi::wgpu_render_bundle_set_pipeline(&mut self.bundle, handle.id);
    }

    fn set_index_buffer(&mut self, op: SetIndexBuffer<Driver>) {
        bundle_ffi::wgpu_render_bundle_set_index_buffer(
            &mut self.bundle,
            op.buffer_handle.id,
            index_format_to_wgc(&op.index_format),
            op.range.as_ref().map(|r| r.start).unwrap_or(0) as u64,
            op.range.as_ref().and_then(|r| NonZeroU64::new(r.len() as u64))
        );
    }

    fn set_vertex_buffer(&mut self, op: SetVertexBuffer<Driver>) {
        bundle_ffi::wgpu_render_bundle_set_vertex_buffer(
            &mut self.bundle,
            op.slot,
            op.buffer_handle.id,
            op.range.as_ref().map(|r| r.start).unwrap_or(0) as u64,
            op.range.as_ref().and_then(|r| NonZeroU64::new(r.len() as u64))
        );
    }

    fn draw(&mut self, op: Draw) {
        bundle_ffi::wgpu_render_bundle_draw(&mut self.bundle, op.vertex_count, op.instance_count, op.first_vertex, op.first_instance);
    }

    fn draw_indexed(&mut self, op: DrawIndexed) {
        bundle_ffi::wgpu_render_bundle_draw_indexed(&mut self.bundle, op.index_count, op.instance_count, op.first_index, op.base_vertex as i32, op.first_instance);
    }

    fn draw_indirect(&mut self, buffer_handle: &BufferHandle, offset: usize) {
        bundle_ffi::wgpu_render_bundle_draw_indirect(&mut self.bundle, buffer_handle.id, offset as u64);
    }

    fn draw_indexed_indirect(&mut self, buffer_handle: &BufferHandle, offset: usize) {
        bundle_ffi::wgpu_render_bundle_draw_indexed_indirect(&mut self.bundle, buffer_handle.id, offset as u64);
    }
}

impl RenderBundleEncoder<Driver> for RenderBundleEncoderHandle {
    fn finish(self) -> RenderBundleHandle {
        let res: Result<_, wgc::command::RenderBundleError> = gfx_select!(self.bundle.parent() => self.global.render_bundle_encoder_finish(
            self.bundle,
            &wgc::command::RenderBundleDescriptor {
                label: None
            },
            None,
        ));

        match res {
            Ok(id) => RenderBundleHandle {
                global: self.global.clone(),
                id
            },
            Err(err)  => panic!("{}", err)
        }
    }
}

#[derive(Clone)]
pub struct CommandBufferHandle {
    global: Arc<Global>,
    id: CommandBufferId,
}

impl Drop for CommandBufferHandle {
    fn drop(&mut self) {
        gfx_select!(self.id => self.global.command_buffer_drop(self.id));
    }
}

#[derive(Clone)]
pub struct RenderBundleHandle {
    global: Arc<Global>,
    id: RenderBundleId,
}

impl Drop for RenderBundleHandle {
    fn drop(&mut self) {
        gfx_select!(self.id => self.global.render_bundle_drop(self.id));
    }
}

#[derive(Clone)]
pub struct QueueHandle {
    global: Arc<Global>,
    id: QueueId,
}

impl Queue<Driver> for QueueHandle {
    fn submit(&self, command_buffer: &CommandBufferHandle) {
        let res: Result<wgc::device::queue::WrappedSubmissionIndex, wgc::device::queue::QueueSubmitError> = gfx_select!(self.id => self.global.queue_submit(
            self.id,
            &[command_buffer.id],
        ));

        if let Err(err) = res {
            panic!("{}", err)
        }
    }

    fn write_buffer(&self, operation: WriteBufferOperation<Driver>) {
        let res: Result<(), wgc::device::queue::QueueWriteError> = gfx_select!(self.id => self.global.queue_write_buffer(
            self.id,
            operation.buffer_handle.id,
            operation.offset as u64,
            operation.data,
        ));

        if let Err(err) = res {
            panic!("{}", err)
        }
    }

    fn write_texture(&self, operation: WriteTextureOperation<Driver>) {
        let res: Result<(), wgc::device::queue::QueueWriteError> = gfx_select!(self.id => self.global.queue_write_texture(
            self.id,
            &image_copy_texture_to_wgc(&operation.image_copy_texture),
            operation.data,
            &image_data_layout_to_wgc(&operation.image_data_layout),
            &size_3d_to_wgc(&operation.extent),
        ));

        if let Err(err) = res {
            panic!("{}", err)
        }
    }
}

impl Drop for QueueHandle {
    fn drop(&mut self) {
        gfx_select!(self.id => self.global.queue_drop(self.id));
    }
}

#[derive(Clone)]
pub struct SamplerHandle {
    global: Arc<Global>,
    id: SamplerId,
}

impl Drop for SamplerHandle {
    fn drop(&mut self) {
        gfx_select!(self.id => self.global.sampler_drop(self.id));
    }
}

#[derive(Clone)]
pub struct BindGroupLayoutHandle {
    global: Arc<Global>,
    id: BindGroupLayoutId,
}

impl Drop for BindGroupLayoutHandle {
    fn drop(&mut self) {
        gfx_select!(self.id => self.global.bind_group_layout_drop(self.id));
    }
}

#[derive(Clone)]
pub struct PipelineLayoutHandle {
    global: Arc<Global>,
    id: PipelineLayoutId,
}

impl Drop for PipelineLayoutHandle {
    fn drop(&mut self) {
        gfx_select!(self.id => self.global.pipeline_layout_drop(self.id));
    }
}

#[derive(Clone)]
pub struct ComputePipelineHandle {
    global: Arc<Global>,
    id: ComputePipelineId,
}

impl Drop for ComputePipelineHandle {
    fn drop(&mut self) {
        gfx_select!(self.id => self.global.compute_pipeline_drop(self.id));
    }
}

#[derive(Clone)]
pub struct RenderPipelineHandle {
    global: Arc<Global>,
    id: RenderPipelineId,
}

impl Drop for RenderPipelineHandle {
    fn drop(&mut self) {
        gfx_select!(self.id => self.global.render_pipeline_drop(self.id));
    }
}

#[derive(Clone)]
pub struct QuerySetHandle {
    global: Arc<Global>,
    id: QuerySetId,
}

impl Drop for QuerySetHandle {
    fn drop(&mut self) {
        gfx_select!(self.id => self.global.query_set_drop(self.id));
    }
}

#[derive(Clone)]
pub struct ShaderModuleHandle {
    global: Arc<Global>,
    id: ShaderModuleId,
}

impl Drop for ShaderModuleHandle {
    fn drop(&mut self) {
        gfx_select!(self.id => self.global.shader_module_drop(self.id));
    }
}

fn features_from_wgc(raw: wgt::Features) -> FlagSet<Feature> {
    let mut features = FlagSet::from(Feature::Depth24UNormStencil8);

    if raw.contains(wgt::Features::DEPTH_CLIP_CONTROL) {
        features |= Feature::DepthClipControl;
    }

    if raw.contains(wgt::Features::DEPTH32FLOAT_STENCIL8) {
        features |= Feature::Depth32FloatStencil8;
    }

    if raw.contains(wgt::Features::TEXTURE_COMPRESSION_BC) {
        features |= Feature::TextureCompressionBc;
    }

    if raw.contains(wgt::Features::TEXTURE_COMPRESSION_ETC2) {
        features |= Feature::TextureComporessionEtc2;
    }

    if raw.contains(wgt::Features::TEXTURE_COMPRESSION_ASTC) {
        features |= Feature::TextureCompressionAstc;
    }

    if raw.contains(wgt::Features::TIMESTAMP_QUERY) {
        features |= Feature::TimestampQuery;
    }

    if raw.contains(wgt::Features::INDIRECT_FIRST_INSTANCE) {
        features |= Feature::IndirectFirstInstance;
    }

    if raw.contains(wgt::Features::SHADER_F16) {
        features |= Feature::ShaderF16;
    }

    if raw.contains(wgt::Features::BGRA8UNORM_STORAGE) {
        features |= Feature::Bgra8UNormStorage;
    }

    features
}

pub fn limits_from_wgc(limits: &wgt::Limits) -> Limits {
    Limits {
        max_texture_dimension_1d: limits.max_texture_dimension_1d,
        max_texture_dimension_2d: limits.max_texture_dimension_2d,
        max_texture_dimension_3d: limits.max_texture_dimension_3d,
        max_texture_array_layers: limits.max_texture_array_layers,
        max_bind_groups: limits.max_bind_groups,
        max_dynamic_uniform_buffers_per_pipeline_layout: limits
            .max_dynamic_uniform_buffers_per_pipeline_layout,
        max_dynamic_storage_buffers_per_pipeline_layout: limits
            .max_dynamic_storage_buffers_per_pipeline_layout,
        max_sampled_textures_per_shader_stage: limits.max_sampled_textures_per_shader_stage,
        max_samplers_per_shader_stage: limits.max_samplers_per_shader_stage,
        max_storage_buffers_per_shader_stage: limits.max_storage_buffers_per_shader_stage,
        max_storage_textures_per_shader_stage: limits.max_storage_textures_per_shader_stage,
        max_uniform_buffers_per_shader_stage: limits.max_uniform_buffers_per_shader_stage,
        max_uniform_buffer_binding_size: limits.max_uniform_buffer_binding_size as u64,
        max_storage_buffer_binding_size: limits.max_storage_buffer_binding_size as u64,
        min_uniform_buffer_offset_alignment: limits.min_uniform_buffer_offset_alignment,
        min_storage_buffer_offset_alignment: limits.min_storage_buffer_offset_alignment,
        max_vertex_buffers: limits.max_vertex_buffers,
        max_vertex_attributes: limits.max_vertex_attributes,
        max_vertex_buffer_array_stride: limits.max_vertex_buffer_array_stride,
        max_inter_stage_shader_components: limits.max_inter_stage_shader_components,
        max_compute_workgroup_storage_size: limits.max_compute_workgroup_storage_size,
        max_compute_invocations_per_workgroup: limits.max_compute_invocations_per_workgroup,
        max_compute_workgroup_size_x: limits.max_compute_workgroup_size_x,
        max_compute_workgroup_size_y: limits.max_compute_workgroup_size_y,
        max_compute_workgroup_size_z: limits.max_compute_workgroup_size_z,
        max_compute_workgroups_per_dimension: limits.max_compute_workgroups_per_dimension,
    }
}

pub fn buffer_usage_to_wgc(usage: &FlagSet<BufferUsage>) -> wgt::BufferUsages {
    wgt::BufferUsages::from_bits_truncate(usage.bits())
}

pub fn texture_format_to_wgc(texture_format: &TextureFormatId) -> wgt::TextureFormat {
    match texture_format {
        TextureFormatId::r8unorm => wgt::TextureFormat::R8Unorm,
        TextureFormatId::r8snorm => wgt::TextureFormat::R8Snorm,
        TextureFormatId::r8uint => wgt::TextureFormat::R8Uint,
        TextureFormatId::r8sint => wgt::TextureFormat::R8Sint,
        TextureFormatId::r16uint => wgt::TextureFormat::R16Uint,
        TextureFormatId::r16sint => wgt::TextureFormat::R16Sint,
        TextureFormatId::r16float => wgt::TextureFormat::R16Float,
        TextureFormatId::rg8unorm => wgt::TextureFormat::Rg8Unorm,
        TextureFormatId::rg8snorm => wgt::TextureFormat::R8Snorm,
        TextureFormatId::rg8uint => wgt::TextureFormat::Rg8Uint,
        TextureFormatId::rg8sint => wgt::TextureFormat::Rg8Sint,
        TextureFormatId::r32uint => wgt::TextureFormat::R32Uint,
        TextureFormatId::r32sint => wgt::TextureFormat::R32Sint,
        TextureFormatId::r32float => wgt::TextureFormat::R32Float,
        TextureFormatId::rg16uint => wgt::TextureFormat::Rg16Uint,
        TextureFormatId::rg16sint => wgt::TextureFormat::Rg16Sint,
        TextureFormatId::rg16float => wgt::TextureFormat::Rg16Float,
        TextureFormatId::rgba8unorm => wgt::TextureFormat::Rgba8Unorm,
        TextureFormatId::rgba8unorm_srgb => wgt::TextureFormat::Rgba8UnormSrgb,
        TextureFormatId::rgba8snorm => wgt::TextureFormat::Rgba8Snorm,
        TextureFormatId::rgba8uint => wgt::TextureFormat::Rgba8Uint,
        TextureFormatId::rgba8sint => wgt::TextureFormat::Rgba8Sint,
        TextureFormatId::bgra8unorm => wgt::TextureFormat::Bgra8Unorm,
        TextureFormatId::bgra8unorm_srgb => wgt::TextureFormat::Bgra8UnormSrgb,
        TextureFormatId::rgb9e5ufloat => wgt::TextureFormat::Rgb9e5Ufloat,
        TextureFormatId::rgb10a2unorm => wgt::TextureFormat::Rgb10a2Unorm,
        TextureFormatId::rg11b10ufloat => wgt::TextureFormat::Rg11b10Float,
        TextureFormatId::rg32uint => wgt::TextureFormat::Rg32Uint,
        TextureFormatId::rg32sint => wgt::TextureFormat::Rg32Sint,
        TextureFormatId::rg32float => wgt::TextureFormat::Rg32Float,
        TextureFormatId::rgba16uint => wgt::TextureFormat::Rgba16Uint,
        TextureFormatId::rgba16sint => wgt::TextureFormat::Rgba16Sint,
        TextureFormatId::rgba16float => wgt::TextureFormat::Rgba16Float,
        TextureFormatId::rgba32uint => wgt::TextureFormat::Rgba32Uint,
        TextureFormatId::rgba32sint => wgt::TextureFormat::Rgba32Sint,
        TextureFormatId::rgba32float => wgt::TextureFormat::Rgba32Float,
        TextureFormatId::stencil8 => wgt::TextureFormat::Stencil8,
        TextureFormatId::depth16unorm => wgt::TextureFormat::Depth16Unorm,
        TextureFormatId::depth24plus => wgt::TextureFormat::Depth24Plus,
        TextureFormatId::depth24plus_stencil8 => wgt::TextureFormat::Depth24PlusStencil8,
        TextureFormatId::depth32float => wgt::TextureFormat::Depth32Float,
        TextureFormatId::depth32float_stencil8 => wgt::TextureFormat::Depth32FloatStencil8,
        TextureFormatId::bc1_rgba_unorm => wgt::TextureFormat::Bc1RgbaUnorm,
        TextureFormatId::bc1_rgba_unorm_srgb => wgt::TextureFormat::Bc1RgbaUnormSrgb,
        TextureFormatId::bc2_rgba_unorm => wgt::TextureFormat::Bc2RgbaUnorm,
        TextureFormatId::bc2_rgba_unorm_srgb => wgt::TextureFormat::Bc2RgbaUnormSrgb,
        TextureFormatId::bc3_rgba_unorm => wgt::TextureFormat::Bc3RgbaUnorm,
        TextureFormatId::bc3_rgba_unorm_srgb => wgt::TextureFormat::Bc3RgbaUnormSrgb,
        TextureFormatId::bc4_r_unorm => wgt::TextureFormat::Bc4RUnorm,
        TextureFormatId::bc4_r_snorm => wgt::TextureFormat::Bc4RSnorm,
        TextureFormatId::bc5_rg_unorm => wgt::TextureFormat::Bc5RgUnorm,
        TextureFormatId::bc5_rg_snorm => wgt::TextureFormat::Bc5RgSnorm,
        TextureFormatId::bc6h_rgb_ufloat => wgt::TextureFormat::Bc6hRgbUfloat,
        TextureFormatId::bc6h_rgb_float => wgt::TextureFormat::Bc6hRgbFloat,
        TextureFormatId::bc7_rgba_unorm => wgt::TextureFormat::Bc7RgbaUnorm,
        TextureFormatId::bc7_rgba_unorm_srgb => wgt::TextureFormat::Bc7RgbaUnormSrgb,
        TextureFormatId::etc2_rgb8unorm => wgt::TextureFormat::Etc2Rgb8Unorm,
        TextureFormatId::etc2_rgb8unorm_srgb => wgt::TextureFormat::Etc2Rgb8UnormSrgb,
        TextureFormatId::etc2_rgb8a1unorm => wgt::TextureFormat::Etc2Rgb8A1Unorm,
        TextureFormatId::etc2_rgb8a1unorm_srgb => wgt::TextureFormat::Etc2Rgb8A1UnormSrgb,
        TextureFormatId::etc2_rgba8unorm => wgt::TextureFormat::Etc2Rgba8Unorm,
        TextureFormatId::etc2_rgba8unorm_srgb => wgt::TextureFormat::Etc2Rgba8UnormSrgb,
        TextureFormatId::eac_r11unorm => wgt::TextureFormat::EacR11Unorm,
        TextureFormatId::eac_r11snorm => wgt::TextureFormat::EacR11Snorm,
        TextureFormatId::eac_rg11unorm => wgt::TextureFormat::EacRg11Unorm,
        TextureFormatId::eac_rg11snorm => wgt::TextureFormat::EacRg11Snorm,
        TextureFormatId::astc_4x4_unorm => wgt::TextureFormat::Astc {
            block: wgt::AstcBlock::B4x4,
            channel: wgt::AstcChannel::Unorm,
        },
        TextureFormatId::astc_4x4_unorm_srgb => wgt::TextureFormat::Astc {
            block: wgt::AstcBlock::B4x4,
            channel: wgt::AstcChannel::UnormSrgb,
        },
        TextureFormatId::astc_5x4_unorm => wgt::TextureFormat::Astc {
            block: wgt::AstcBlock::B5x4,
            channel: wgt::AstcChannel::Unorm,
        },
        TextureFormatId::astc_5x4_unorm_srgb => wgt::TextureFormat::Astc {
            block: wgt::AstcBlock::B5x4,
            channel: wgt::AstcChannel::UnormSrgb,
        },
        TextureFormatId::astc_5x5_unorm => wgt::TextureFormat::Astc {
            block: wgt::AstcBlock::B5x5,
            channel: wgt::AstcChannel::Unorm,
        },
        TextureFormatId::astc_5x5_unorm_srgb => wgt::TextureFormat::Astc {
            block: wgt::AstcBlock::B5x5,
            channel: wgt::AstcChannel::UnormSrgb,
        },
        TextureFormatId::astc_6x5_unorm => wgt::TextureFormat::Astc {
            block: wgt::AstcBlock::B6x5,
            channel: wgt::AstcChannel::Unorm,
        },
        TextureFormatId::astc_6x5_unorm_srgb => wgt::TextureFormat::Astc {
            block: wgt::AstcBlock::B6x5,
            channel: wgt::AstcChannel::UnormSrgb,
        },
        TextureFormatId::astc_6x6_unorm => wgt::TextureFormat::Astc {
            block: wgt::AstcBlock::B6x6,
            channel: wgt::AstcChannel::Unorm,
        },
        TextureFormatId::astc_6x6_unorm_srgb => wgt::TextureFormat::Astc {
            block: wgt::AstcBlock::B6x6,
            channel: wgt::AstcChannel::UnormSrgb,
        },
        TextureFormatId::astc_8x5_unorm => wgt::TextureFormat::Astc {
            block: wgt::AstcBlock::B8x5,
            channel: wgt::AstcChannel::Unorm,
        },
        TextureFormatId::astc_8x5_unorm_srgb => wgt::TextureFormat::Astc {
            block: wgt::AstcBlock::B8x5,
            channel: wgt::AstcChannel::UnormSrgb,
        },
        TextureFormatId::astc_8x6_unorm => wgt::TextureFormat::Astc {
            block: wgt::AstcBlock::B8x6,
            channel: wgt::AstcChannel::Unorm,
        },
        TextureFormatId::astc_8x6_unorm_srgb => wgt::TextureFormat::Astc {
            block: wgt::AstcBlock::B8x6,
            channel: wgt::AstcChannel::UnormSrgb,
        },
        TextureFormatId::astc_8x8_unorm => wgt::TextureFormat::Astc {
            block: wgt::AstcBlock::B8x8,
            channel: wgt::AstcChannel::Unorm,
        },
        TextureFormatId::astc_8x8_unorm_srgb => wgt::TextureFormat::Astc {
            block: wgt::AstcBlock::B8x8,
            channel: wgt::AstcChannel::UnormSrgb,
        },
        TextureFormatId::astc_10x5_unorm => wgt::TextureFormat::Astc {
            block: wgt::AstcBlock::B10x5,
            channel: wgt::AstcChannel::Unorm,
        },
        TextureFormatId::astc_10x5_unorm_srgb => wgt::TextureFormat::Astc {
            block: wgt::AstcBlock::B10x5,
            channel: wgt::AstcChannel::UnormSrgb,
        },
        TextureFormatId::astc_10x6_unorm => wgt::TextureFormat::Astc {
            block: wgt::AstcBlock::B10x6,
            channel: wgt::AstcChannel::Unorm,
        },
        TextureFormatId::astc_10x6_unorm_srgb => wgt::TextureFormat::Astc {
            block: wgt::AstcBlock::B10x6,
            channel: wgt::AstcChannel::UnormSrgb,
        },
        TextureFormatId::astc_10x8_unorm => wgt::TextureFormat::Astc {
            block: wgt::AstcBlock::B10x8,
            channel: wgt::AstcChannel::Unorm,
        },
        TextureFormatId::astc_10x8_unorm_srgb => wgt::TextureFormat::Astc {
            block: wgt::AstcBlock::B10x8,
            channel: wgt::AstcChannel::UnormSrgb,
        },
        TextureFormatId::astc_10x10_unorm => wgt::TextureFormat::Astc {
            block: wgt::AstcBlock::B10x10,
            channel: wgt::AstcChannel::Unorm,
        },
        TextureFormatId::astc_10x10_unorm_srgb => wgt::TextureFormat::Astc {
            block: wgt::AstcBlock::B10x10,
            channel: wgt::AstcChannel::UnormSrgb,
        },
        TextureFormatId::astc_12x10_unorm => wgt::TextureFormat::Astc {
            block: wgt::AstcBlock::B12x10,
            channel: wgt::AstcChannel::Unorm,
        },
        TextureFormatId::astc_12x10_unorm_srgb => wgt::TextureFormat::Astc {
            block: wgt::AstcBlock::B12x10,
            channel: wgt::AstcChannel::UnormSrgb,
        },
        TextureFormatId::astc_12x12_unorm => wgt::TextureFormat::Astc {
            block: wgt::AstcBlock::B12x12,
            channel: wgt::AstcChannel::Unorm,
        },
        TextureFormatId::astc_12x12_unorm_srgb => wgt::TextureFormat::Astc {
            block: wgt::AstcBlock::B12x12,
            channel: wgt::AstcChannel::UnormSrgb,
        },
    }
}

pub fn size_3d_to_wgc(size: &(u32, u32, u32)) -> wgt::Extent3d {
    let (width, height, depth_or_array_layers) = *size;

    wgt::Extent3d {
        width,
        height,
        depth_or_array_layers,
    }
}

pub fn texture_dimension_to_wgc(texture_dimension: &TextureDimensions) -> wgt::TextureDimension {
    match texture_dimension {
        TextureDimensions::One => wgt::TextureDimension::D1,
        TextureDimensions::Two => wgt::TextureDimension::D2,
        TextureDimensions::Three => wgt::TextureDimension::D3,
    }
}

pub fn texture_usage_to_wgc(texture_usage: &FlagSet<TextureUsage>) -> wgt::TextureUsages {
    wgt::TextureUsages::from_bits_retain(texture_usage.bits())
}

pub fn address_mode_to_wgc(address_mode: &AddressMode) -> wgt::AddressMode {
    match address_mode {
        AddressMode::ClampToEdge => wgt::AddressMode::ClampToEdge,
        AddressMode::Repeat => wgt::AddressMode::Repeat,
        AddressMode::MirrorRepeat => wgt::AddressMode::MirrorRepeat,
    }
}

pub fn filter_mode_to_wgc(filter_mode: &FilterMode) -> wgt::FilterMode {
    match filter_mode {
        FilterMode::Nearest => wgt::FilterMode::Nearest,
        FilterMode::Linear => wgt::FilterMode::Linear,
    }
}

pub fn compare_function_to_wgc(compare_function: &CompareFunction) -> wgt::CompareFunction {
    match compare_function {
        CompareFunction::Never => wgt::CompareFunction::Never,
        CompareFunction::Less => wgt::CompareFunction::Less,
        CompareFunction::Equal => wgt::CompareFunction::Equal,
        CompareFunction::LessEqual => wgt::CompareFunction::LessEqual,
        CompareFunction::Greater => wgt::CompareFunction::Greater,
        CompareFunction::NotEqual => wgt::CompareFunction::NotEqual,
        CompareFunction::GreaterEqual => wgt::CompareFunction::GreaterEqual,
        CompareFunction::Always => wgt::CompareFunction::Always,
    }
}

pub fn visibility_to_wgc(visibility: &FlagSet<ShaderStage>) -> wgt::ShaderStages {
    wgt::ShaderStages::from_bits_retain(visibility.bits())
}

pub fn buffer_binding_type_to_wgc(
    buffer_binding_type: &BufferBindingType,
) -> wgt::BufferBindingType {
    match buffer_binding_type {
        BufferBindingType::Uniform => wgt::BufferBindingType::Uniform,
        BufferBindingType::Storage => wgt::BufferBindingType::Storage { read_only: false },
        BufferBindingType::ReadonlyStorage => wgt::BufferBindingType::Storage { read_only: true },
    }
}

pub fn sampler_binding_type_to_wgc(
    sampler_binding_type: &SamplerBindingType,
) -> wgt::SamplerBindingType {
    match sampler_binding_type {
        SamplerBindingType::Filtering => wgt::SamplerBindingType::Filtering,
        SamplerBindingType::NonFiltering => wgt::SamplerBindingType::NonFiltering,
        SamplerBindingType::Comparison => wgt::SamplerBindingType::Comparison,
    }
}

pub fn texture_sample_type_to_wgc(
    texture_sampler_type: &TextureSampleType,
) -> wgt::TextureSampleType {
    match texture_sampler_type {
        TextureSampleType::Float => wgt::TextureSampleType::Float { filterable: true },
        TextureSampleType::UnfilterableFloat => wgt::TextureSampleType::Float { filterable: false },
        TextureSampleType::SignedInteger => wgt::TextureSampleType::Sint,
        TextureSampleType::UnsignedInteger => wgt::TextureSampleType::Uint,
        TextureSampleType::Depth => wgt::TextureSampleType::Depth,
    }
}

pub fn texture_view_dimension_to_wgc(
    texture_view_dimension: &TextureViewDimension,
) -> wgt::TextureViewDimension {
    match texture_view_dimension {
        TextureViewDimension::One => wgt::TextureViewDimension::D1,
        TextureViewDimension::Two => wgt::TextureViewDimension::D2,
        TextureViewDimension::Three => wgt::TextureViewDimension::D3,
        TextureViewDimension::TwoArray => wgt::TextureViewDimension::D2Array,
        TextureViewDimension::Cube => wgt::TextureViewDimension::Cube,
        TextureViewDimension::CubeArray => wgt::TextureViewDimension::CubeArray,
    }
}

pub fn storage_texture_access_to_wgc(
    storage_texture_access: &StorageTextureAccess,
) -> wgt::StorageTextureAccess {
    match storage_texture_access {
        StorageTextureAccess::ReadOnly => wgt::StorageTextureAccess::ReadOnly,
        StorageTextureAccess::WriteOnly => wgt::StorageTextureAccess::WriteOnly,
        StorageTextureAccess::ReadWrite => wgt::StorageTextureAccess::ReadWrite,
    }
}

pub fn binding_type_to_wgc(binding_type: &BindingType) -> wgt::BindingType {
    match binding_type {
        BindingType::Buffer(binding_type) => wgt::BindingType::Buffer {
            ty: Default::default(),
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        BindingType::Sampler(binding_type) => {
            wgt::BindingType::Sampler(sampler_binding_type_to_wgc(binding_type))
        }
        BindingType::Texture {
            sample_type,
            dimension,
            multisampled,
        } => wgt::BindingType::Texture {
            sample_type: texture_sample_type_to_wgc(sample_type),
            view_dimension: texture_view_dimension_to_wgc(dimension),
            multisampled: *multisampled,
        },
        BindingType::StorageTexture {
            access,
            format,
            dimension,
        } => wgt::BindingType::StorageTexture {
            access: storage_texture_access_to_wgc(access),
            format: texture_format_to_wgc(format),
            view_dimension: texture_view_dimension_to_wgc(dimension),
        },
    }
}

pub fn bind_group_layout_entry_to_wgc(
    bind_group_layout_entry: BindGroupLayoutEntry,
) -> wgt::BindGroupLayoutEntry {
    wgt::BindGroupLayoutEntry {
        binding: bind_group_layout_entry.binding,
        visibility: visibility_to_wgc(&bind_group_layout_entry.visibility),
        ty: binding_type_to_wgc(&bind_group_layout_entry.binding_type),
        count: None,
    }
}

pub fn binding_resource_to_wgc<'a>(
    binding_resource: BindingResource<'a, Driver>,
) -> wgc::binding_model::BindingResource<'a> {
    match binding_resource {
        BindingResource::BufferBinding(b) => wgc::binding_model::BindingResource::Buffer(b),
        BindingResource::TextureView(b) => wgc::binding_model::BindingResource::TextureView(b.id),
        BindingResource::Sampler(b) => wgc::binding_model::BindingResource::Sampler(b.id),
    }
}

pub fn bind_group_entry_to_wgc(
    bind_group_entry: BindGroupEntry<Driver>,
) -> wgc::binding_model::BindGroupEntry {
    wgc::binding_model::BindGroupEntry {
        binding: bind_group_entry.binding,
        resource: binding_resource_to_wgc(bind_group_entry.resource),
    }
}

pub fn query_type_to_wgc(query_type: &QueryType) -> wgt::QueryType {
    match query_type {
        QueryType::Occlusion => wgt::QueryType::Occlusion,
        QueryType::Timestamp => wgt::QueryType::Timestamp,
    }
}

pub fn vertex_step_mode_to_wgc(vertex_step_mode: &VertexStepMode) -> wgt::VertexStepMode {
    match vertex_step_mode {
        VertexStepMode::Vertex => wgt::VertexStepMode::Vertex,
        VertexStepMode::Instance => wgt::VertexStepMode::Instance,
    }
}

pub fn vertex_format_to_wgc(vertex_format: &VertexFormat) -> wgt::VertexFormat {
    match vertex_format {
        VertexFormat::uint8x2 => wgt::VertexFormat::Uint8x2,
        VertexFormat::uint8x4 => wgt::VertexFormat::Uint8x4,
        VertexFormat::sint8x2 => wgt::VertexFormat::Sint8x2,
        VertexFormat::sint8x4 => wgt::VertexFormat::Sint8x4,
        VertexFormat::unorm8x2 => wgt::VertexFormat::Unorm8x2,
        VertexFormat::unorm8x4 => wgt::VertexFormat::Unorm8x4,
        VertexFormat::snorm8x2 => wgt::VertexFormat::Snorm8x2,
        VertexFormat::snorm8x4 => wgt::VertexFormat::Snorm8x4,
        VertexFormat::uint16x2 => wgt::VertexFormat::Uint16x2,
        VertexFormat::uint16x4 => wgt::VertexFormat::Uint16x4,
        VertexFormat::sint16x2 => wgt::VertexFormat::Sint16x2,
        VertexFormat::sint16x4 => wgt::VertexFormat::Sint16x4,
        VertexFormat::unorm16x2 => wgt::VertexFormat::Unorm16x2,
        VertexFormat::unorm16x4 => wgt::VertexFormat::Unorm16x4,
        VertexFormat::snorm16x2 => wgt::VertexFormat::Snorm16x2,
        VertexFormat::snorm16x4 => wgt::VertexFormat::Snorm16x4,
        VertexFormat::float16x2 => wgt::VertexFormat::Float16x2,
        VertexFormat::float16x4 => wgt::VertexFormat::Float16x4,
        VertexFormat::float32 => wgt::VertexFormat::Float32,
        VertexFormat::float32x2 => wgt::VertexFormat::Float32x2,
        VertexFormat::float32x3 => wgt::VertexFormat::Float32x3,
        VertexFormat::float32x4 => wgt::VertexFormat::Float32x4,
        VertexFormat::uint32 => wgt::VertexFormat::Uint32,
        VertexFormat::uint32x2 => wgt::VertexFormat::Uint32x2,
        VertexFormat::uint32x3 => wgt::VertexFormat::Uint32x3,
        VertexFormat::uint32x4 => wgt::VertexFormat::Uint32x4,
        VertexFormat::sint32 => wgt::VertexFormat::Sint32,
        VertexFormat::sint32x2 => wgt::VertexFormat::Sint32x2,
        VertexFormat::sint32x3 => wgt::VertexFormat::Sint32x3,
        VertexFormat::sint32x4 => wgt::VertexFormat::Sint32x4,
    }
}

pub fn vertex_attribute_to_wgc(vertex_attribute: &VertexAttribute) -> wgt::VertexAttribute {
    wgt::VertexAttribute {
        format: vertex_format_to_wgc(&vertex_attribute.format),
        offset: vertex_attribute.offset as u64,
        shader_location: vertex_attribute.shader_location,
    }
}

pub fn vertex_buffer_layout_to_wgc(
    vertex_buffer_layout: &VertexBufferLayout,
) -> wgc::pipeline::VertexBufferLayout<'static> {
    let attributes: Vec<_> = vertex_buffer_layout
        .attributes
        .iter()
        .map(vertex_attribute_to_wgc)
        .collect();

    wgc::pipeline::VertexBufferLayout {
        array_stride: vertex_buffer_layout.array_stride as u64,
        step_mode: vertex_step_mode_to_wgc(&vertex_buffer_layout.step_mode),
        attributes: attributes.into(),
    }
}

pub fn primitive_topology_to_wgc(primitive_topology: &PrimitiveTopology) -> wgt::PrimitiveTopology {
    match primitive_topology {
        PrimitiveTopology::PointList => wgt::PrimitiveTopology::PointList,
        PrimitiveTopology::LineList => wgt::PrimitiveTopology::LineList,
        PrimitiveTopology::LineStrip => wgt::PrimitiveTopology::LineStrip,
        PrimitiveTopology::TriangleList => wgt::PrimitiveTopology::TriangleList,
        PrimitiveTopology::TriangleStrip => wgt::PrimitiveTopology::TriangleStrip,
    }
}

pub fn index_format_to_wgc(index_format: &IndexFormat) -> wgt::IndexFormat {
    match index_format {
        IndexFormat::U16 => wgt::IndexFormat::Uint16,
        IndexFormat::U32 => wgt::IndexFormat::Uint32,
    }
}

pub fn front_face_to_wgc(front_face: &FrontFace) -> wgt::FrontFace {
    match front_face {
        FrontFace::Clockwise => wgt::FrontFace::Cw,
        FrontFace::CounterClockwise => wgt::FrontFace::Ccw,
    }
}

pub fn cull_mode_to_wgc(cull_mode: CullMode) -> wgt::Face {
    match cull_mode {
        CullMode::Front => wgt::Face::Front,
        CullMode::Back => wgt::Face::Back,
    }
}

pub fn primitive_state_to_wgc(primitive_state: &PrimitiveState) -> wgt::PrimitiveState {
    wgt::PrimitiveState {
        topology: primitive_topology_to_wgc(&primitive_state.topology),
        strip_index_format: primitive_state
            .strip_index_format
            .as_ref()
            .map(index_format_to_wgc),
        front_face: front_face_to_wgc(&primitive_state.front_face),
        cull_mode: primitive_state.cull_mode.map(cull_mode_to_wgc),
        unclipped_depth: false,
        polygon_mode: wgt::PolygonMode::Fill,
        conservative: false,
    }
}

pub fn stencil_operation_to_wgc(stencil_operation: &StencilOperation) -> wgt::StencilOperation {
    match stencil_operation {
        StencilOperation::Keep => wgt::StencilOperation::Keep,
        StencilOperation::Zero => wgt::StencilOperation::Zero,
        StencilOperation::Replace => wgt::StencilOperation::Replace,
        StencilOperation::Invert => wgt::StencilOperation::Invert,
        StencilOperation::IncrementClamp => wgt::StencilOperation::IncrementClamp,
        StencilOperation::DecrementClamp => wgt::StencilOperation::DecrementClamp,
        StencilOperation::IncrementWrap => wgt::StencilOperation::IncrementWrap,
        StencilOperation::DecrementWrap => wgt::StencilOperation::DecrementWrap,
    }
}

pub fn stencil_face_state_to_wgc(stencil_face_state: &StencilFaceState) -> wgt::StencilFaceState {
    wgt::StencilFaceState {
        compare: compare_function_to_wgc(&stencil_face_state.compare),
        fail_op: stencil_operation_to_wgc(&stencil_face_state.fail_op),
        depth_fail_op: stencil_operation_to_wgc(&stencil_face_state.depth_fail_op),
        pass_op: stencil_operation_to_wgc(&stencil_face_state.pass_op),
    }
}

pub fn depth_stencil_state_to_wgc(
    depth_stencil_state: &DepthStencilState,
) -> wgt::DepthStencilState {
    wgt::DepthStencilState {
        format: texture_format_to_wgc(&depth_stencil_state.format),
        depth_write_enabled: depth_stencil_state.depth_write_enabled,
        depth_compare: compare_function_to_wgc(&depth_stencil_state.depth_compare),
        stencil: wgt::StencilState {
            front: stencil_face_state_to_wgc(&depth_stencil_state.stencil_front),
            back: stencil_face_state_to_wgc(&depth_stencil_state.stencil_back),
            read_mask: depth_stencil_state.stencil_read_mask,
            write_mask: depth_stencil_state.stencil_write_mask,
        },
        bias: wgt::DepthBiasState {
            constant: depth_stencil_state.depth_bias,
            slope_scale: depth_stencil_state.depth_bias_slope_scale,
            clamp: depth_stencil_state.depth_bias_clamp,
        },
    }
}

pub fn multisample_state_to_wgc(multisample_state: &MultisampleState) -> wgt::MultisampleState {
    wgt::MultisampleState {
        count: multisample_state.count,
        mask: multisample_state.mask as u64,
        alpha_to_coverage_enabled: multisample_state.alpha_to_coverage_enabled,
    }
}

pub fn blend_factor_to_wgc(blend_factor: &BlendFactor) -> wgt::BlendFactor {
    match blend_factor {
        BlendFactor::Zero => wgt::BlendFactor::Zero,
        BlendFactor::One => wgt::BlendFactor::One,
        BlendFactor::Src => wgt::BlendFactor::Src,
        BlendFactor::OneMinusSrc => wgt::BlendFactor::OneMinusSrc,
        BlendFactor::SrcAlpha => wgt::BlendFactor::SrcAlpha,
        BlendFactor::OneMinusSrcAlpha => wgt::BlendFactor::OneMinusSrcAlpha,
        BlendFactor::Dst => wgt::BlendFactor::Dst,
        BlendFactor::OneMinusDst => wgt::BlendFactor::OneMinusDst,
        BlendFactor::DstAlpha => wgt::BlendFactor::DstAlpha,
        BlendFactor::OneMinusDstAlpha => wgt::BlendFactor::OneMinusDstAlpha,
        BlendFactor::SrcAlphaSaturated => wgt::BlendFactor::SrcAlphaSaturated,
        BlendFactor::Constant => wgt::BlendFactor::Constant,
        BlendFactor::OneMinusConstant => wgt::BlendFactor::OneMinusConstant,
    }
}

pub fn blend_component_to_wgc(blend_component: &BlendComponent) -> wgt::BlendComponent {
    match blend_component {
        BlendComponent::Add {
            src_factor,
            dst_factor,
        } => wgt::BlendComponent {
            src_factor: blend_factor_to_wgc(src_factor),
            dst_factor: blend_factor_to_wgc(dst_factor),
            operation: wgt::BlendOperation::Add,
        },
        BlendComponent::Subtract {
            src_factor,
            dst_factor,
        } => wgt::BlendComponent {
            src_factor: blend_factor_to_wgc(src_factor),
            dst_factor: blend_factor_to_wgc(dst_factor),
            operation: wgt::BlendOperation::Subtract,
        },
        BlendComponent::ReverseSubtract {
            src_factor,
            dst_factor,
        } => wgt::BlendComponent {
            src_factor: blend_factor_to_wgc(src_factor),
            dst_factor: blend_factor_to_wgc(dst_factor),
            operation: wgt::BlendOperation::ReverseSubtract,
        },
        BlendComponent::Min => wgt::BlendComponent {
            src_factor: wgt::BlendFactor::One,
            dst_factor: wgt::BlendFactor::Zero,
            operation: wgt::BlendOperation::Min,
        },
        BlendComponent::Max => wgt::BlendComponent {
            src_factor: wgt::BlendFactor::One,
            dst_factor: wgt::BlendFactor::Zero,
            operation: wgt::BlendOperation::Max,
        },
    }
}

pub fn blend_state_to_wgc(blend_state: &BlendState) -> wgt::BlendState {
    wgt::BlendState {
        color: blend_component_to_wgc(&blend_state.color),
        alpha: blend_component_to_wgc(&blend_state.alpha),
    }
}

pub fn color_write_wgc(color_write: &FlagSet<ColorWrite>) -> wgt::ColorWrites {
    wgt::ColorWrites::from_bits_retain(color_write.bits())
}

pub fn color_target_state_to_wgc(color_target_state: &ColorTargetState) -> wgt::ColorTargetState {
    wgt::ColorTargetState {
        format: texture_format_to_wgc(&color_target_state.format),
        blend: color_target_state.blend.as_ref().map(blend_state_to_wgc),
        write_mask: color_write_wgc(&color_target_state.write_mask),
    }
}

pub fn map_mode_to_wgc(map_mode: &MapMode) -> wgc::device::HostMap {
    match map_mode {
        MapMode::Read => wgc::device::HostMap::Read,
        MapMode::Write => wgc::device::HostMap::Write,
    }
}

pub fn texture_aspect_to_wgc(texture_aspect: &TextureAspect) -> wgt::TextureAspect {
    match texture_aspect {
        TextureAspect::All => wgt::TextureAspect::All,
        TextureAspect::StencilOnly => wgt::TextureAspect::StencilOnly,
        TextureAspect::DepthOnly => wgt::TextureAspect::DepthOnly,
    }
}

pub fn image_copy_buffer_to_wgc(image_copy_buffer: &ImageCopyBuffer<Driver>) -> wgc::command::ImageCopyBuffer {
    let bytes_per_row = image_copy_buffer.bytes_per_block * image_copy_buffer.blocks_per_row;

    wgc::command::ImageCopyBuffer {
        buffer: image_copy_buffer.buffer_handle.id,
        layout: wgt::ImageDataLayout {
            offset: image_copy_buffer.offset as u64,
            bytes_per_row: Some(bytes_per_row),
            rows_per_image: Some(image_copy_buffer.rows_per_image),
        },
    }
}

pub fn origin_3d_to_wgc(origin: &(u32, u32, u32)) -> wgt::Origin3d {
    wgt::Origin3d {
        x: origin.0,
        y: origin.1,
        z: origin.2,
    }
}

pub fn image_copy_texture_to_wgc(image_copy_texture: &ImageCopyTexture<Driver>) -> wgc::command::ImageCopyTexture {
    wgc::command::ImageCopyTexture {
        texture: image_copy_texture.texture_handle.id,
        mip_level: image_copy_texture.mip_level,
        origin: origin_3d_to_wgc(&image_copy_texture.origin),
        aspect: texture_aspect_to_wgc(&image_copy_texture.aspect),
    }
}

pub fn load_op_to_wgc<T>(load_op: &LoadOp<T>) -> wgc::command::LoadOp {
    match load_op {
        LoadOp::Load => wgc::command::LoadOp::Load,
        LoadOp::Clear(_) => wgc::command::LoadOp::Clear,
    }
}

pub fn load_op_color_to_wgc(load_op: &LoadOp<[f64; 4]>) -> wgt::Color {
    match load_op {
        LoadOp::Load => wgt::Color::default(),
        LoadOp::Clear([r, g, b, a]) => wgt::Color {
            r: *r,
            g: *g,
            b: *b,
            a: *a,
        },
    }
}

pub fn load_op_clear_value<T: Copy + Default>(load_op: &LoadOp<T>) -> T {
    match load_op {
        LoadOp::Load => T::default(),
        LoadOp::Clear(v) => *v
    }
}

pub fn store_op_to_wgc(store_op: &StoreOp) -> wgc::command::StoreOp {
    match store_op {
        StoreOp::Store => wgc::command::StoreOp::Store,
        StoreOp::Discard => wgc::command::StoreOp::Discard,
    }
}

pub fn render_pass_color_attachment_to_wgc(render_pass_color_attachment: RenderPassColorAttachment<Driver>) -> wgc::command::RenderPassColorAttachment {
    wgc::command::RenderPassColorAttachment {
        view: render_pass_color_attachment.view.id,
        resolve_target: render_pass_color_attachment.resolve_target.as_ref().map(|t| t.id),
        channel: wgc::command::PassChannel {
            load_op: load_op_to_wgc(&render_pass_color_attachment.load_op),
            store_op: store_op_to_wgc(&render_pass_color_attachment.store_op),
            clear_value: load_op_color_to_wgc(&render_pass_color_attachment.load_op),
            read_only: false,
        },
    }
}

pub fn depth_stencil_operations_to_wgc<T: Copy + Default>(depth_stencil_operations: &Option<DepthStencilOperations<T>>) -> wgc::command::PassChannel<T> {
    if let Some(depth_stencil_operations) = depth_stencil_operations {
        wgc::command::PassChannel {
            load_op: load_op_to_wgc(&depth_stencil_operations.load_op),
            store_op: store_op_to_wgc(&depth_stencil_operations.store_op),
            clear_value: load_op_clear_value(&depth_stencil_operations.load_op),
            read_only: false,
        }
    } else {
        wgc::command::PassChannel {
            load_op: wgc::command::LoadOp::Load,
            store_op: wgc::command::StoreOp::Discard,
            clear_value: T::default(),
            read_only: true,
        }
    }
}

pub fn render_pass_depth_stencil_attachment_to_wgc(render_pass_depth_stencil_attachment: &RenderPassDepthStencilAttachment<Driver>) -> wgc::command::RenderPassDepthStencilAttachment {
    wgc::command::RenderPassDepthStencilAttachment {
        view: render_pass_depth_stencil_attachment.view.id,
        depth: depth_stencil_operations_to_wgc(&render_pass_depth_stencil_attachment.depth_operations),
        stencil: depth_stencil_operations_to_wgc(&render_pass_depth_stencil_attachment.stencil_operations),
    }
}

pub fn image_data_layout_to_wgc(image_data_layout: &ImageDataLayout) -> wgt::ImageDataLayout {
    wgt::ImageDataLayout {
        offset: image_data_layout.offset as u64,
        bytes_per_row: Some(image_data_layout.bytes_per_row),
        rows_per_image: Some(image_data_layout.rows_per_image),
    }
}