use std::borrow::Borrow;
use std::collections::HashMap;
use std::error::Error;
use std::future::Future;
use std::ops::Range;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{fmt, mem, slice};

use flagset::FlagSet;
use js_sys::Uint8Array;
use pin_project::pin_project;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::GpuSupportedFeatures;

use crate::adapter::{Feature, Limits};
use crate::buffer::MapError;
use crate::command::{BlendConstant, Draw, DrawIndexed, ScissorRect, Viewport};
use crate::device::DeviceDescriptor;
use crate::driver::{
    Adapter, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingResource, BindingType, Buffer, BufferBindingType, BufferDescriptor, ClearBuffer,
    ColorTargetState, CommandEncoder, ComputePassEncoder, ComputePipelineDescriptor,
    CopyBufferToBuffer, CopyBufferToTexture, CopyTextureToBuffer, CopyTextureToTexture,
    DepthStencilOperations, DepthStencilState, Device, ExecuteRenderBundlesEncoder, FragmentState,
    ImageCopyBuffer, ImageCopyTexture, ImageDataLayout, MapMode, MultisampleState,
    PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, ProgrammablePassEncoder,
    QuerySetDescriptor, QueryType, Queue, RenderBundleEncoder, RenderBundleEncoderDescriptor,
    RenderEncoder, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, RenderPassEncoder, RenderPipelineDescriptor, ResolveQuerySet,
    SamplerBindingType, SamplerDescriptor, SetIndexBuffer, SetVertexBuffer, StencilFaceState,
    StencilOperation, StorageTextureAccess, Texture, TextureAspect, TextureDescriptor,
    TextureDimensions, TextureSampleType, TextureViewDescriptor, TextureViewDimension, VertexState,
    WriteBufferOperation, WriteTextureOperation,
};
use crate::render_pipeline::{
    BlendComponent, BlendFactor, BlendState, CullMode, FrontFace, IndexFormat, VertexFormat,
    VertexStepMode,
};
use crate::render_target::{LoadOp, StoreOp};
use crate::sampler::{AddressMode, FilterMode};
use crate::texture::format::TextureFormatId;
use crate::{driver, CompareFunction};

pub struct Driver;

impl driver::Driver for Driver {
    type AdapterHandle = AdapterHandle;
    type BindGroupHandle = BindGroupHandle;
    type DeviceHandle = DeviceHandle;
    type BufferHandle = BufferHandle;
    type BufferBinding = BufferBinding;
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
    inner: web_sys::GpuAdapter,
}

impl From<web_sys::GpuAdapter> for AdapterHandle {
    fn from(inner: web_sys::GpuAdapter) -> Self {
        AdapterHandle { inner }
    }
}

impl Adapter<Driver> for AdapterHandle {
    type RequestDevice = RequestDevice;

    fn supported_features(&self) -> FlagSet<Feature> {
        features_from_web_sys(&self.inner.features())
    }

    fn supported_limits(&self) -> Limits {
        limits_from_web_sys(&self.inner.limits())
    }

    fn request_device<Flags>(&self, descriptor: &DeviceDescriptor<Flags>) -> RequestDevice
    where
        Flags: Into<FlagSet<Feature>> + Copy,
    {
        let DeviceDescriptor {
            required_features,
            required_limits,
        } = descriptor;

        let mut desc = web_sys::GpuDeviceDescriptor::new();

        let required_features = (*required_features).into();

        if required_features != FlagSet::from(Feature::None) {
            desc.required_features(features_to_web_sys(&required_features).as_ref());
        }

        if required_limits != &Limits::default() {
            todo!("not present in web_sys")
        }

        let promise = self.inner.request_device_with_descriptor(&desc);

        RequestDevice {
            inner: JsFuture::from(promise),
        }
    }
}

pub struct RequestDevice {
    inner: JsFuture,
}

impl Future for RequestDevice {
    type Output = Result<DeviceHandle, Box<dyn Error>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.get_mut().inner)
            .poll(cx)
            .map_ok(|device| DeviceHandle {
                inner: device.unchecked_into(),
            })
            .map_err(|err| {
                RequestDeviceError {
                    inner: err.unchecked_into(),
                }
                .into()
            })
    }
}

pub struct RequestDeviceError {
    inner: web_sys::DomException,
}

impl fmt::Display for RequestDeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.message().fmt(f)
    }
}

impl fmt::Debug for RequestDeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Error for RequestDeviceError {}

#[derive(Clone)]
pub struct DeviceHandle {
    pub(crate) inner: web_sys::GpuDevice,
}

impl Device<Driver> for DeviceHandle {
    type CreateComputePipelineAsync = CreateComputePipelineAsync;

    type CreateRenderPipelineAsync = CreateRenderPipelineAsync;

    fn create_buffer(&self, descriptor: &BufferDescriptor) -> BufferHandle {
        let BufferDescriptor {
            size,
            usage_flags,
            mapped_at_creation,
        } = *descriptor;

        let mut desc = web_sys::GpuBufferDescriptor::new(size as f64, usage_flags.bits());

        if mapped_at_creation {
            desc.mapped_at_creation(true);
        }

        let inner = self.inner.create_buffer(&desc);

        BufferHandle { inner }
    }

    fn create_texture(&self, descriptor: &TextureDescriptor) -> TextureHandle {
        let TextureDescriptor {
            size,
            mipmap_levels,
            sample_count,
            dimensions,
            format,
            usage_flags,
            view_formats,
        } = *descriptor;

        let size = size_3d_to_web_sys(size);
        let format = texture_format_to_web_sys(&format);
        let dimensions = texture_dimension_to_web_sys(&dimensions);

        let view_formats = view_formats
            .iter()
            .map(|f| JsValue::from_str(texture_format_to_str(f)))
            .collect::<js_sys::Array>();

        let mut desc =
            web_sys::GpuTextureDescriptor::new(format, size.as_ref(), usage_flags.bits());

        desc.dimension(dimensions);
        desc.mip_level_count(mipmap_levels);
        desc.sample_count(sample_count);
        desc.view_formats(view_formats.as_ref());

        let inner = self.inner.create_texture(&desc);

        TextureHandle { inner }
    }

    fn create_sampler(&self, descriptor: &SamplerDescriptor) -> SamplerHandle {
        let SamplerDescriptor {
            address_mode_u,
            address_mode_v,
            address_mode_w,
            magnification_filter,
            minification_filter,
            mipmap_filter,
            lod_clamp,
            max_anisotropy,
            compare,
        } = descriptor;

        let mut desc = web_sys::GpuSamplerDescriptor::new();

        desc.address_mode_u(address_mode_to_web_sys(address_mode_u));
        desc.address_mode_v(address_mode_to_web_sys(address_mode_v));
        desc.address_mode_w(address_mode_to_web_sys(address_mode_w));
        desc.lod_min_clamp(*lod_clamp.start());
        desc.lod_max_clamp(*lod_clamp.end());
        desc.min_filter(filter_mode_to_web_sys(minification_filter));
        desc.mag_filter(filter_mode_to_web_sys(magnification_filter));
        desc.mipmap_filter(filter_mode_to_web_sys_mipmap(mipmap_filter));
        desc.max_anisotropy(*max_anisotropy);

        if let Some(compare) = compare {
            desc.compare(compare_function_to_web_sys(compare));
        }

        let inner = self.inner.create_sampler_with_descriptor(&desc);

        SamplerHandle { inner }
    }

    fn create_bind_group_layout<E>(
        &self,
        descriptor: BindGroupLayoutDescriptor<E>,
    ) -> BindGroupLayoutHandle
    where
        E: IntoIterator<Item = BindGroupLayoutEntry>,
    {
        let entries = descriptor
            .entries
            .into_iter()
            .map(|e| bind_group_layout_entry_to_web_sys(&e))
            .collect::<js_sys::Array>();

        let desc = web_sys::GpuBindGroupLayoutDescriptor::new(entries.as_ref());

        let inner = self.inner.create_bind_group_layout(&desc);

        BindGroupLayoutHandle { inner }
    }

    fn create_pipeline_layout<I>(
        &self,
        descriptor: PipelineLayoutDescriptor<I>,
    ) -> PipelineLayoutHandle
    where
        I: IntoIterator,
        I::Item: Borrow<BindGroupLayoutHandle>,
    {
        let bind_group_layouts = js_sys::Array::new();

        for layout in descriptor.bind_group_layouts {
            bind_group_layouts.push(layout.borrow().inner.as_ref());
        }

        let desc = web_sys::GpuPipelineLayoutDescriptor::new(bind_group_layouts.as_ref());
        let inner = self.inner.create_pipeline_layout(&desc);

        PipelineLayoutHandle { inner }
    }

    fn create_bind_group<'a, E>(
        &self,
        descriptor: BindGroupDescriptor<Driver, E>,
    ) -> BindGroupHandle
    where
        E: IntoIterator<Item = BindGroupEntry<'a, Driver>>,
    {
        let entries = js_sys::Array::new();

        for entry in descriptor.entries.into_iter() {
            match &entry.resource {
                BindingResource::BufferBinding(buffer_binding) => {
                    entries.push(
                        web_sys::GpuBindGroupEntry::new(
                            entry.binding,
                            buffer_binding.inner.as_ref(),
                        )
                        .as_ref(),
                    );
                }
                BindingResource::TextureView(texture_view) => {
                    entries.push(
                        web_sys::GpuBindGroupEntry::new(entry.binding, texture_view.inner.as_ref())
                            .as_ref(),
                    );
                }
                BindingResource::Sampler(sampler_handle) => {
                    entries.push(
                        web_sys::GpuBindGroupEntry::new(
                            entry.binding,
                            sampler_handle.inner.as_ref(),
                        )
                        .as_ref(),
                    );
                }
            }
        }

        let desc = web_sys::GpuBindGroupDescriptor::new(entries.as_ref(), &descriptor.layout.inner);
        let inner = self.inner.create_bind_group(&desc);

        BindGroupHandle { inner }
    }

    fn create_query_set(&self, descriptor: &QuerySetDescriptor) -> QuerySetHandle {
        let QuerySetDescriptor { query_type, len } = descriptor;

        let desc =
            web_sys::GpuQuerySetDescriptor::new(*len as u32, query_type_to_web_sys(query_type));
        let inner = self.inner.create_query_set(&desc);

        QuerySetHandle { inner }
    }

    fn create_shader_module(&self, source: &str) -> ShaderModuleHandle {
        let desc = web_sys::GpuShaderModuleDescriptor::new(source);
        let inner = self.inner.create_shader_module(&desc);

        ShaderModuleHandle { inner }
    }

    fn create_compute_pipeline(
        &self,
        descriptor: &ComputePipelineDescriptor<Driver>,
    ) -> ComputePipelineHandle {
        let desc = compute_pipeline_descriptor_to_web_sys(descriptor);
        let inner = self.inner.create_compute_pipeline(&desc);

        ComputePipelineHandle { inner }
    }

    fn create_compute_pipeline_async(
        &self,
        descriptor: &ComputePipelineDescriptor<Driver>,
    ) -> CreateComputePipelineAsync {
        let desc = compute_pipeline_descriptor_to_web_sys(descriptor);
        let promise = self.inner.create_compute_pipeline_async(&desc);

        CreateComputePipelineAsync {
            inner: promise.into(),
        }
    }

    fn create_render_pipeline(
        &self,
        descriptor: &RenderPipelineDescriptor<Driver>,
    ) -> RenderPipelineHandle {
        let desc = render_pipeline_descriptor_to_web_sys(descriptor);
        let inner = self.inner.create_render_pipeline(&desc);

        RenderPipelineHandle { inner }
    }

    fn create_render_pipeline_async(
        &self,
        descriptor: &RenderPipelineDescriptor<Driver>,
    ) -> CreateRenderPipelineAsync {
        let desc = render_pipeline_descriptor_to_web_sys(descriptor);
        let promise = self.inner.create_render_pipeline_async(&desc);

        CreateRenderPipelineAsync {
            inner: promise.into(),
        }
    }

    fn create_command_encoder(&self) -> CommandEncoderHandle {
        let inner = self.inner.create_command_encoder();

        CommandEncoderHandle { inner }
    }

    fn create_render_bundle_encoder(
        &self,
        descriptor: &RenderBundleEncoderDescriptor,
    ) -> RenderBundleEncoderHandle {
        let RenderBundleEncoderDescriptor {
            color_formats,
            depth_stencil_format,
            sample_count,
            depth_read_only,
            stencil_read_only,
        } = descriptor;

        let color_formats: js_sys::Array = color_formats
            .into_iter()
            .map(|f| JsValue::from_str(texture_format_to_str(f)))
            .collect();

        let mut desc = web_sys::GpuRenderBundleEncoderDescriptor::new(&color_formats);

        if let Some(depth_stencil_format) = depth_stencil_format {
            desc.depth_stencil_format(texture_format_to_web_sys(depth_stencil_format));
        }

        desc.depth_read_only(*depth_read_only);
        desc.stencil_read_only(*stencil_read_only);
        desc.sample_count(*sample_count);

        let inner = self.inner.create_render_bundle_encoder(&desc);

        RenderBundleEncoderHandle { inner }
    }

    fn queue_handle(&self) -> QueueHandle {
        let inner = self.inner.queue();

        QueueHandle { inner }
    }
}

#[pin_project]
pub struct Map {
    #[pin]
    inner: JsFuture,
}

impl Future for Map {
    type Output = Result<(), MapError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project()
            .inner
            .poll(cx)
            .map_ok(|_| ())
            .map_err(|_| MapError)
    }
}

pub struct Mapped<T> {
    buffered: Box<[T]>,
}

impl<T> AsRef<[T]> for Mapped<T> {
    fn as_ref(&self) -> &[T] {
        &self.buffered
    }
}

pub struct MappedMut<T> {
    buffered: Box<[T]>,
    mapped_bytes: Uint8Array,
}

impl<T> AsRef<[T]> for MappedMut<T> {
    fn as_ref(&self) -> &[T] {
        &self.buffered
    }
}

impl<T> AsMut<[T]> for MappedMut<T> {
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.buffered
    }
}

impl<T> Drop for MappedMut<T> {
    fn drop(&mut self) {
        let size_in_bytes = self.buffered.len() * mem::size_of::<T>();
        let ptr = self.buffered.as_ptr() as *const u8;

        let bytes = unsafe { slice::from_raw_parts(ptr, size_in_bytes) };

        self.mapped_bytes.copy_from(bytes);
    }
}

#[derive(Clone)]
pub struct BufferBinding {
    inner: web_sys::GpuBufferBinding,
}

#[derive(Clone)]
pub struct BufferHandle {
    inner: web_sys::GpuBuffer,
}

impl Buffer<Driver> for BufferHandle {
    type Map = Map;
    type Mapped<'a, E: 'a> = Mapped<E>;
    type MappedMut<'a, E: 'a> = MappedMut<E>;

    fn map(&self, mode: MapMode, range: Range<usize>) -> Map {
        let size = range.len() as u32;

        let promise = self.inner.map_async_with_u32_and_u32(
            map_mode_to_web_sys(&mode),
            range.start as u32,
            size,
        );

        Map {
            inner: promise.into(),
        }
    }

    fn mapped<'a, E>(&'a self, offset_in_bytes: usize, size_in_elements: usize) -> Mapped<E> {
        let size_in_bytes = (size_in_elements * mem::size_of::<E>()) as u32;

        let mapped_bytes = Uint8Array::new(
            &self
                .inner
                .get_mapped_range_with_u32_and_u32(offset_in_bytes as u32, size_in_bytes),
        );
        let mut buffered = Box::<[E]>::new_uninit_slice(size_in_elements);
        let ptr = buffered.as_mut_ptr() as *mut ();

        copy_buffer_to_memory(
            &mapped_bytes,
            0,
            size_in_bytes,
            &wasm_bindgen::memory(),
            ptr,
        );

        let buffered = unsafe { buffered.assume_init() };

        Mapped { buffered }
    }

    fn mapped_mut<'a, E>(
        &'a self,
        offset_in_bytes: usize,
        size_in_elements: usize,
    ) -> MappedMut<E> {
        let size_in_bytes = (size_in_elements * mem::size_of::<E>()) as u32;

        let mapped_bytes = Uint8Array::new(
            &self
                .inner
                .get_mapped_range_with_u32_and_u32(offset_in_bytes as u32, size_in_bytes),
        );
        let mut buffered = Box::<[E]>::new_uninit_slice(size_in_elements);
        let ptr = buffered.as_mut_ptr() as *mut ();

        copy_buffer_to_memory(
            &mapped_bytes,
            0,
            size_in_bytes,
            &wasm_bindgen::memory(),
            ptr,
        );

        let buffered = unsafe { buffered.assume_init() };

        MappedMut {
            buffered,
            mapped_bytes,
        }
    }

    fn unmap(&self) {
        self.inner.unmap();
    }

    fn binding(&self, offset: usize, size: usize) -> BufferBinding {
        let mut inner = web_sys::GpuBufferBinding::new(&self.inner);

        inner.offset(offset as f64);
        inner.size(size as f64);

        BufferBinding { inner }
    }
}

#[derive(Clone)]
pub struct TextureHandle {
    pub inner: web_sys::GpuTexture,
}

impl From<web_sys::GpuTexture> for TextureHandle {
    fn from(inner: web_sys::GpuTexture) -> Self {
        TextureHandle { inner }
    }
}

impl Texture<Driver> for TextureHandle {
    fn texture_view(&self, descriptor: &TextureViewDescriptor) -> TextureView {
        let TextureViewDescriptor {
            format,
            dimensions: dimension,
            aspect,
            mip_levels,
            layers,
        } = descriptor;

        let mut desc = web_sys::GpuTextureViewDescriptor::new();

        desc.format(texture_format_to_web_sys(format));
        desc.dimension(texture_view_dimension_to_web_sys(dimension));
        desc.aspect(texture_aspect_to_web_sys(aspect));
        desc.base_mip_level(mip_levels.start);
        desc.mip_level_count(mip_levels.len() as u32);
        desc.base_array_layer(layers.start);
        desc.array_layer_count(layers.len() as u32);

        let inner = self.inner.create_view_with_descriptor(&desc);

        TextureView { inner }
    }
}

#[derive(Clone)]
pub struct TextureView {
    inner: web_sys::GpuTextureView,
}

#[derive(Clone)]
pub struct SamplerHandle {
    inner: web_sys::GpuSampler,
}

#[derive(Clone)]
pub struct BindGroupLayoutHandle {
    inner: web_sys::GpuBindGroupLayout,
}

#[derive(Clone)]
pub struct PipelineLayoutHandle {
    inner: web_sys::GpuPipelineLayout,
}

#[derive(Clone)]
pub struct BindGroupHandle {
    inner: web_sys::GpuBindGroup,
}

#[derive(Clone)]
pub struct ShaderModuleHandle {
    inner: web_sys::GpuShaderModule,
}

#[derive(Clone)]
pub struct QuerySetHandle {
    inner: web_sys::GpuQuerySet,
}

#[derive(Clone)]
pub struct CommandBufferHandle {
    inner: web_sys::GpuCommandBuffer,
}

#[derive(Clone)]
pub struct QueueHandle {
    pub inner: web_sys::GpuQueue,
}

impl Queue<Driver> for QueueHandle {
    fn submit(&self, command_buffer: &CommandBufferHandle) {
        let array = js_sys::Array::new();

        array.push(command_buffer.inner.as_ref());

        self.inner.submit(array.as_ref());
    }

    fn write_buffer(&self, operation: WriteBufferOperation<Driver>) {
        let WriteBufferOperation {
            buffer_handle,
            offset,
            data,
        } = operation;

        self.inner
            .write_buffer_with_u32_and_u8_array(&buffer_handle.inner, offset as u32, data);
    }

    fn write_texture(&self, operation: WriteTextureOperation<Driver>) {
        let WriteTextureOperation {
            image_copy_texture,
            image_data_layout,
            extent,
            data,
        } = operation;

        self.inner
            .write_texture_with_u8_array_and_gpu_extent_3d_dict(
                &image_copy_texture_to_web_sys(image_copy_texture),
                data,
                &image_data_layout_to_web_sys(&image_data_layout),
                &size_3d_to_web_sys(extent),
            );
    }
}

#[derive(Clone)]
pub struct ComputePipelineHandle {
    inner: web_sys::GpuComputePipeline,
}

pub struct CreateComputePipelineAsync {
    inner: JsFuture,
}

impl Future for CreateComputePipelineAsync {
    type Output = ComputePipelineHandle;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.get_mut().inner).poll(cx).map(|result| {
            let inner = result.expect("pipeline creation should not fail");

            ComputePipelineHandle {
                inner: inner.unchecked_into(),
            }
        })
    }
}

#[derive(Clone)]
pub struct RenderPipelineHandle {
    inner: web_sys::GpuRenderPipeline,
}

pub struct CreateRenderPipelineAsync {
    inner: JsFuture,
}

impl Future for CreateRenderPipelineAsync {
    type Output = RenderPipelineHandle;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.get_mut().inner).poll(cx).map(|result| {
            let inner = result.expect("pipeline creation should not fail");

            RenderPipelineHandle {
                inner: inner.unchecked_into(),
            }
        })
    }
}

#[derive(Clone)]
pub struct CommandEncoderHandle {
    inner: web_sys::GpuCommandEncoder,
}

impl CommandEncoder<Driver> for CommandEncoderHandle {
    fn copy_buffer_to_buffer(&mut self, op: CopyBufferToBuffer<Driver>) {
        let CopyBufferToBuffer {
            source,
            source_offset,
            destination,
            destination_offset,
            size,
        } = op;

        self.inner.copy_buffer_to_buffer_with_u32_and_u32_and_u32(
            &source.inner,
            source_offset as u32,
            &destination.inner,
            destination_offset as u32,
            size as u32,
        );
    }

    fn copy_buffer_to_texture(&mut self, op: CopyBufferToTexture<Driver>) {
        let CopyBufferToTexture {
            source,
            destination,
            copy_size,
        } = op;

        self.inner.copy_buffer_to_texture_with_gpu_extent_3d_dict(
            &image_copy_buffer_to_web_sys(source),
            &image_copy_texture_to_web_sys(destination),
            &size_3d_to_web_sys(copy_size),
        );
    }

    fn copy_texture_to_buffer(&mut self, op: CopyTextureToBuffer<Driver>) {
        let CopyTextureToBuffer {
            source,
            destination,
            copy_size,
        } = op;

        self.inner.copy_texture_to_buffer_with_gpu_extent_3d_dict(
            &image_copy_texture_to_web_sys(source),
            &image_copy_buffer_to_web_sys(destination),
            &size_3d_to_web_sys(copy_size),
        );
    }

    fn copy_texture_to_texture(&mut self, op: CopyTextureToTexture<Driver>) {
        let CopyTextureToTexture {
            source,
            destination,
            copy_size,
        } = op;

        self.inner.copy_texture_to_texture_with_gpu_extent_3d_dict(
            &image_copy_texture_to_web_sys(source),
            &image_copy_texture_to_web_sys(destination),
            &size_3d_to_web_sys(copy_size),
        );
    }

    fn clear_buffer(&mut self, op: ClearBuffer<Driver>) {
        let ClearBuffer { buffer, range } = op;

        let offset = range.start as u32;
        let size = range.len() as u32;

        self.inner
            .clear_buffer_with_u32_and_u32(&buffer.inner, offset, size);
    }

    fn begin_compute_pass(&mut self) -> ComputePassEncoderHandle {
        let inner = self.inner.begin_compute_pass();

        ComputePassEncoderHandle { inner }
    }

    fn begin_render_pass<I>(
        &mut self,
        descriptor: RenderPassDescriptor<Driver, I>,
    ) -> RenderPassEncoderHandle
    where
        I: IntoIterator<Item = Option<RenderPassColorAttachment<Driver>>>,
    {
        let color_attachments = js_sys::Array::new();

        for attachment in descriptor.color_attachments {
            if let Some(attachment) = attachment {
                color_attachments
                    .push(render_pass_color_attachment_to_web_sys(&attachment).as_ref());
            } else {
                color_attachments.push(&JsValue::null());
            }
        }

        let mut desc = web_sys::GpuRenderPassDescriptor::new(color_attachments.as_ref());

        if let Some(depth_stencil_attachment) = &descriptor.depth_stencil_attachment {
            desc.depth_stencil_attachment(&render_pass_depth_stencil_attachment_to_web_sys(
                depth_stencil_attachment,
            ));
        }

        if let Some(query_set) = descriptor.occlusion_query_set {
            desc.occlusion_query_set(&query_set.inner);
        }

        let inner = self.inner.begin_render_pass(&desc);

        RenderPassEncoderHandle { inner }
    }

    fn write_timestamp(&mut self, query_set: &QuerySetHandle, index: usize) {
        write_timestamp(&self.inner, &query_set.inner, index as u32);
    }

    fn resolve_query_set(&mut self, op: ResolveQuerySet<Driver>) {
        let ResolveQuerySet {
            query_set,
            query_range,
            destination,
            destination_offset,
        } = op;

        let offset = query_range.start as u32;
        let count = query_range.len() as u32;

        self.inner.resolve_query_set_with_u32(
            &query_set.inner,
            offset,
            count,
            &destination.inner,
            destination_offset as u32,
        );
    }

    fn finish(self) -> CommandBufferHandle {
        let inner = self.inner.finish();

        CommandBufferHandle { inner }
    }
}

#[derive(Clone)]
pub struct ComputePassEncoderHandle {
    inner: web_sys::GpuComputePassEncoder,
}

impl ProgrammablePassEncoder<Driver> for ComputePassEncoderHandle {
    fn set_bind_group(&mut self, index: u32, handle: &BindGroupHandle) {
        self.inner.set_bind_group(index, Some(&handle.inner));
    }
}

impl ComputePassEncoder<Driver> for ComputePassEncoderHandle {
    fn set_pipeline(&mut self, handle: &ComputePipelineHandle) {
        self.inner.set_pipeline(&handle.inner);
    }

    fn dispatch_workgroups(&mut self, x: u32, y: u32, z: u32) {
        self.inner
            .dispatch_workgroups_with_workgroup_count_y_and_workgroup_count_z(x, y, z);
    }

    fn dispatch_workgroups_indirect(&mut self, buffer_handle: &BufferHandle, offset: usize) {
        self.inner
            .dispatch_workgroups_indirect_with_u32(&buffer_handle.inner, offset as u32);
    }

    fn end(self) {
        self.inner.end();
    }
}

#[derive(Clone)]
pub struct RenderPassEncoderHandle {
    inner: web_sys::GpuRenderPassEncoder,
}

impl ProgrammablePassEncoder<Driver> for RenderPassEncoderHandle {
    fn set_bind_group(&mut self, index: u32, handle: &BindGroupHandle) {
        self.inner.set_bind_group(index, Some(&handle.inner));
    }
}

impl RenderEncoder<Driver> for RenderPassEncoderHandle {
    fn set_pipeline(&mut self, handle: &RenderPipelineHandle) {
        self.inner.set_pipeline(&handle.inner);
    }

    fn set_index_buffer(&mut self, op: SetIndexBuffer<Driver>) {
        let SetIndexBuffer {
            buffer_handle,
            index_format,
            range,
        } = op;

        let index_format = index_format_to_web_sys(&index_format);

        if let Some(range) = range {
            let offset = range.start as u32;
            let size = range.len() as u32;

            self.inner.set_index_buffer_with_u32_and_u32(
                &buffer_handle.inner,
                index_format,
                offset,
                size,
            );
        } else {
            self.inner
                .set_index_buffer(&buffer_handle.inner, index_format);
        }
    }

    fn set_vertex_buffer(&mut self, op: SetVertexBuffer<Driver>) {
        let SetVertexBuffer {
            slot,
            buffer_handle,
            range,
        } = op;

        if let Some(range) = range {
            let offset = range.start as u32;
            let size = range.len() as u32;

            self.inner.set_vertex_buffer_with_u32_and_u32(
                slot,
                Some(&buffer_handle.inner),
                offset,
                size,
            );
        } else {
            self.inner
                .set_vertex_buffer(slot, Some(&buffer_handle.inner));
        }
    }

    fn draw(&mut self, op: Draw) {
        let Draw {
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        } = op;

        self.inner
            .draw_with_instance_count_and_first_vertex_and_first_instance(
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            );
    }

    fn draw_indexed(&mut self, op: DrawIndexed) {
        let DrawIndexed {
            index_count,
            instance_count,
            first_index,
            base_vertex,
            first_instance,
        } = op;

        self.inner
            .draw_indexed_with_instance_count_and_first_index_and_base_vertex_and_first_instance(
                index_count,
                instance_count,
                first_index,
                base_vertex as i32,
                first_instance,
            );
    }

    fn draw_indirect(&mut self, buffer_handle: &BufferHandle, offset: usize) {
        self.inner
            .draw_indirect_with_u32(&buffer_handle.inner, offset as u32);
    }

    fn draw_indexed_indirect(&mut self, buffer_handle: &BufferHandle, offset: usize) {
        self.inner
            .draw_indexed_indirect_with_u32(&buffer_handle.inner, offset as u32);
    }
}

impl RenderPassEncoder<Driver> for RenderPassEncoderHandle {
    fn set_viewport(&mut self, viewport: &Viewport) {
        let Viewport {
            x,
            y,
            width,
            height,
            min_depth,
            max_depth,
        } = *viewport;

        self.inner
            .set_viewport(x, y, width, height, min_depth, max_depth);
    }

    fn set_scissor_rect(&mut self, scissor_rect: &ScissorRect) {
        let ScissorRect {
            x,
            y,
            width,
            height,
        } = *scissor_rect;

        self.inner.set_scissor_rect(x, y, width, height);
    }

    fn set_blend_constant(&mut self, blend_constant: &BlendConstant) {
        let BlendConstant { r, g, b, a } = *blend_constant;

        let color = web_sys::GpuColorDict::new(r as f64, g as f64, b as f64, a as f64);

        self.inner.set_blend_constant_with_gpu_color_dict(&color);
    }

    fn set_stencil_reference(&mut self, stencil_reference: u32) {
        self.inner.set_stencil_reference(stencil_reference);
    }

    fn begin_occlusion_query(&mut self, query_index: u32) {
        self.inner.begin_occlusion_query(query_index);
    }

    fn end_occlusion_query(&mut self) {
        self.inner.end_occlusion_query();
    }

    fn execute_bundles(&mut self) -> ExecuteRenderBundlesEncoderHandle {
        ExecuteRenderBundlesEncoderHandle {
            inner: &self.inner,
            bundles: js_sys::Array::new(),
        }
    }

    fn end(self) {
        self.inner.end();
    }
}

pub struct ExecuteRenderBundlesEncoderHandle<'a> {
    inner: &'a web_sys::GpuRenderPassEncoder,
    bundles: js_sys::Array,
}

impl<'a> ExecuteRenderBundlesEncoder<Driver> for ExecuteRenderBundlesEncoderHandle<'a> {
    fn push_bundle(&mut self, bundle: &RenderBundleHandle) {
        self.bundles.push(bundle.inner.as_ref());
    }

    fn finish(self) {
        self.inner.execute_bundles(self.bundles.as_ref());
    }
}

#[derive(Clone)]
pub struct RenderBundleHandle {
    inner: web_sys::GpuRenderBundle,
}

impl AsRef<RenderBundleHandle> for RenderBundleHandle {
    fn as_ref(&self) -> &RenderBundleHandle {
        self
    }
}

#[derive(Clone)]
pub struct RenderBundleEncoderHandle {
    inner: web_sys::GpuRenderBundleEncoder,
}

impl ProgrammablePassEncoder<Driver> for RenderBundleEncoderHandle {
    fn set_bind_group(&mut self, index: u32, handle: &BindGroupHandle) {
        self.inner.set_bind_group(index, Some(&handle.inner));
    }
}

impl RenderEncoder<Driver> for RenderBundleEncoderHandle {
    fn set_pipeline(&mut self, handle: &RenderPipelineHandle) {
        self.inner.set_pipeline(&handle.inner);
    }

    fn set_index_buffer(&mut self, op: SetIndexBuffer<Driver>) {
        let SetIndexBuffer {
            buffer_handle,
            index_format,
            range,
        } = op;

        let index_format = index_format_to_web_sys(&index_format);

        if let Some(range) = range {
            let offset = range.start as u32;
            let size = range.len() as u32;

            self.inner.set_index_buffer_with_u32_and_u32(
                &buffer_handle.inner,
                index_format,
                offset,
                size,
            );
        } else {
            self.inner
                .set_index_buffer(&buffer_handle.inner, index_format);
        }
    }

    fn set_vertex_buffer(&mut self, op: SetVertexBuffer<Driver>) {
        let SetVertexBuffer {
            slot,
            buffer_handle,
            range,
        } = op;

        if let Some(range) = range {
            let offset = range.start as u32;
            let size = range.len() as u32;

            self.inner.set_vertex_buffer_with_u32_and_u32(
                slot,
                Some(&buffer_handle.inner),
                offset,
                size,
            );
        } else {
            self.inner
                .set_vertex_buffer(slot, Some(&buffer_handle.inner));
        }
    }

    fn draw(&mut self, op: Draw) {
        let Draw {
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        } = op;

        self.inner
            .draw_with_instance_count_and_first_vertex_and_first_instance(
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            );
    }

    fn draw_indexed(&mut self, op: DrawIndexed) {
        let DrawIndexed {
            index_count,
            instance_count,
            first_index,
            base_vertex,
            first_instance,
        } = op;

        self.inner
            .draw_indexed_with_instance_count_and_first_index_and_base_vertex_and_first_instance(
                index_count,
                instance_count,
                first_index,
                base_vertex as i32,
                first_instance,
            );
    }

    fn draw_indirect(&mut self, buffer_handle: &BufferHandle, offset: usize) {
        self.inner
            .draw_indirect_with_u32(&buffer_handle.inner, offset as u32);
    }

    fn draw_indexed_indirect(&mut self, buffer_handle: &BufferHandle, offset: usize) {
        self.inner
            .draw_indexed_indirect_with_u32(&buffer_handle.inner, offset as u32);
    }
}

impl RenderBundleEncoder<Driver> for RenderBundleEncoderHandle {
    fn finish(self) -> RenderBundleHandle {
        let inner = self.inner.finish();

        RenderBundleHandle { inner }
    }
}

pub fn address_mode_to_web_sys(access_mode: &AddressMode) -> web_sys::GpuAddressMode {
    match access_mode {
        AddressMode::ClampToEdge => web_sys::GpuAddressMode::ClampToEdge,
        AddressMode::Repeat => web_sys::GpuAddressMode::Repeat,
        AddressMode::MirrorRepeat => web_sys::GpuAddressMode::MirrorRepeat,
    }
}

pub fn filter_mode_to_web_sys(filter_mode: &FilterMode) -> web_sys::GpuFilterMode {
    match filter_mode {
        FilterMode::Nearest => web_sys::GpuFilterMode::Nearest,
        FilterMode::Linear => web_sys::GpuFilterMode::Linear,
    }
}

pub fn filter_mode_to_web_sys_mipmap(filter_mode: &FilterMode) -> web_sys::GpuMipmapFilterMode {
    match filter_mode {
        FilterMode::Nearest => web_sys::GpuMipmapFilterMode::Nearest,
        FilterMode::Linear => web_sys::GpuMipmapFilterMode::Linear,
    }
}

pub fn bind_group_layout_entry_to_web_sys(
    bind_group_layout_entry: &BindGroupLayoutEntry,
) -> web_sys::GpuBindGroupLayoutEntry {
    let mut entry = web_sys::GpuBindGroupLayoutEntry::new(
        bind_group_layout_entry.binding,
        bind_group_layout_entry.visibility.bits(),
    );

    match &bind_group_layout_entry.binding_type {
        BindingType::Buffer(binding_type) => {
            let mut layout = web_sys::GpuBufferBindingLayout::new();

            layout.type_(buffer_binding_type_to_web_sys(binding_type));

            entry.buffer(&layout);
        }
        BindingType::Sampler(binding_type) => {
            let mut layout = web_sys::GpuSamplerBindingLayout::new();

            layout.type_(sampler_binding_type_to_web_sys(binding_type));

            entry.sampler(&layout);
        }
        BindingType::Texture {
            sample_type,
            dimension,
            multisampled,
        } => {
            let mut layout = web_sys::GpuTextureBindingLayout::new();

            layout.sample_type(texture_sample_type_to_web_sys(sample_type));
            layout.view_dimension(texture_view_dimension_to_web_sys(dimension));
            layout.multisampled(*multisampled);

            entry.texture(&layout);
        }
        BindingType::StorageTexture {
            access,
            format,
            dimension,
        } => {
            let mut layout =
                web_sys::GpuStorageTextureBindingLayout::new(texture_format_to_web_sys(format));

            layout.access(storage_texture_access_to_web_sys(access));
            layout.view_dimension(texture_view_dimension_to_web_sys(dimension));

            entry.storage_texture(&layout);
        }
    }

    entry
}

pub fn buffer_binding_type_to_web_sys(
    binding_type: &BufferBindingType,
) -> web_sys::GpuBufferBindingType {
    match binding_type {
        BufferBindingType::Uniform => web_sys::GpuBufferBindingType::Uniform,
        BufferBindingType::Storage => web_sys::GpuBufferBindingType::Storage,
        BufferBindingType::ReadonlyStorage => web_sys::GpuBufferBindingType::ReadOnlyStorage,
    }
}

pub fn sampler_binding_type_to_web_sys(
    binding_type: &SamplerBindingType,
) -> web_sys::GpuSamplerBindingType {
    match binding_type {
        SamplerBindingType::Filtering => web_sys::GpuSamplerBindingType::Filtering,
        SamplerBindingType::NonFiltering => web_sys::GpuSamplerBindingType::NonFiltering,
        SamplerBindingType::Comparison => web_sys::GpuSamplerBindingType::Comparison,
    }
}

pub fn texture_sample_type_to_web_sys(
    sample_type: &TextureSampleType,
) -> web_sys::GpuTextureSampleType {
    match sample_type {
        TextureSampleType::Float => web_sys::GpuTextureSampleType::Float,
        TextureSampleType::UnfilterableFloat => web_sys::GpuTextureSampleType::UnfilterableFloat,
        TextureSampleType::SignedInteger => web_sys::GpuTextureSampleType::Sint,
        TextureSampleType::UnsignedInteger => web_sys::GpuTextureSampleType::Uint,
        TextureSampleType::Depth => web_sys::GpuTextureSampleType::Depth,
    }
}

pub fn storage_texture_access_to_web_sys(
    access: &StorageTextureAccess,
) -> web_sys::GpuStorageTextureAccess {
    match access {
        StorageTextureAccess::ReadOnly => panic!("not supported"),
        StorageTextureAccess::WriteOnly => web_sys::GpuStorageTextureAccess::WriteOnly,
        StorageTextureAccess::ReadWrite => panic!("not supported"),
    }
}

pub fn compare_function_to_web_sys(
    compare_function: &CompareFunction,
) -> web_sys::GpuCompareFunction {
    match compare_function {
        CompareFunction::Never => web_sys::GpuCompareFunction::Never,
        CompareFunction::Less => web_sys::GpuCompareFunction::Less,
        CompareFunction::Equal => web_sys::GpuCompareFunction::Equal,
        CompareFunction::LessEqual => web_sys::GpuCompareFunction::LessEqual,
        CompareFunction::Greater => web_sys::GpuCompareFunction::Greater,
        CompareFunction::NotEqual => web_sys::GpuCompareFunction::NotEqual,
        CompareFunction::GreaterEqual => web_sys::GpuCompareFunction::GreaterEqual,
        CompareFunction::Always => web_sys::GpuCompareFunction::Always,
    }
}

pub fn image_copy_buffer_to_web_sys(
    image_copy_buffer: ImageCopyBuffer<Driver>,
) -> web_sys::GpuImageCopyBuffer {
    let mut copy_buffer = web_sys::GpuImageCopyBuffer::new(&image_copy_buffer.buffer_handle.inner);

    copy_buffer.offset(image_copy_buffer.offset as f64);
    copy_buffer.bytes_per_row(image_copy_buffer.bytes_per_block * image_copy_buffer.blocks_per_row);
    copy_buffer.rows_per_image(image_copy_buffer.rows_per_image);

    copy_buffer
}

pub fn image_copy_texture_to_web_sys(
    image_copy_texture: ImageCopyTexture<Driver>,
) -> web_sys::GpuImageCopyTexture {
    let mut copy_texture =
        web_sys::GpuImageCopyTexture::new(&image_copy_texture.texture_handle.inner);

    copy_texture.aspect(texture_aspect_to_web_sys(&image_copy_texture.aspect));
    copy_texture.mip_level(image_copy_texture.mip_level);
    copy_texture.origin(origin_3d_to_web_sys(image_copy_texture.origin).as_ref());

    copy_texture
}

pub fn image_data_layout_to_web_sys(
    image_data_layout: &ImageDataLayout,
) -> web_sys::GpuImageDataLayout {
    let ImageDataLayout {
        offset,
        bytes_per_row,
        rows_per_image,
    } = *image_data_layout;

    let mut image_data_layout = web_sys::GpuImageDataLayout::new();

    image_data_layout.offset(offset as f64);
    image_data_layout.bytes_per_row(bytes_per_row);
    image_data_layout.rows_per_image(rows_per_image);

    image_data_layout
}

pub fn origin_3d_to_web_sys(origin: (u32, u32, u32)) -> web_sys::GpuOrigin3dDict {
    let (x, y, z) = origin;

    let mut origin = web_sys::GpuOrigin3dDict::new();

    origin.x(x);
    origin.y(y);
    origin.z(z);

    origin
}

pub fn size_3d_to_web_sys(size: (u32, u32, u32)) -> web_sys::GpuExtent3dDict {
    let (width, height, depth_or_layers) = size;

    let mut size = web_sys::GpuExtent3dDict::new(width);

    size.height(height);
    size.depth_or_array_layers(depth_or_layers);

    size
}

pub fn map_mode_to_web_sys(map_mode: &MapMode) -> u32 {
    match map_mode {
        MapMode::Read => 1,
        MapMode::Write => 2,
    }
}

pub fn texture_format_to_web_sys(texture_format: &TextureFormatId) -> web_sys::GpuTextureFormat {
    match texture_format {
        TextureFormatId::r8unorm => web_sys::GpuTextureFormat::R8unorm,
        TextureFormatId::r8snorm => web_sys::GpuTextureFormat::R8snorm,
        TextureFormatId::r8uint => web_sys::GpuTextureFormat::R8uint,
        TextureFormatId::r8sint => web_sys::GpuTextureFormat::R8sint,
        TextureFormatId::r16uint => web_sys::GpuTextureFormat::R16uint,
        TextureFormatId::r16sint => web_sys::GpuTextureFormat::R16sint,
        TextureFormatId::r16float => web_sys::GpuTextureFormat::R16float,
        TextureFormatId::rg8unorm => web_sys::GpuTextureFormat::Rg8unorm,
        TextureFormatId::rg8snorm => web_sys::GpuTextureFormat::Rg8snorm,
        TextureFormatId::rg8uint => web_sys::GpuTextureFormat::Rg8uint,
        TextureFormatId::rg8sint => web_sys::GpuTextureFormat::Rg8sint,
        TextureFormatId::r32uint => web_sys::GpuTextureFormat::R32uint,
        TextureFormatId::r32sint => web_sys::GpuTextureFormat::R32sint,
        TextureFormatId::r32float => web_sys::GpuTextureFormat::R32float,
        TextureFormatId::rg16uint => web_sys::GpuTextureFormat::Rg16uint,
        TextureFormatId::rg16sint => web_sys::GpuTextureFormat::Rg16sint,
        TextureFormatId::rg16float => web_sys::GpuTextureFormat::Rg16float,
        TextureFormatId::rgba8unorm => web_sys::GpuTextureFormat::Rgba8unorm,
        TextureFormatId::rgba8unorm_srgb => web_sys::GpuTextureFormat::Rgba8unormSrgb,
        TextureFormatId::rgba8snorm => web_sys::GpuTextureFormat::Rgba8snorm,
        TextureFormatId::rgba8uint => web_sys::GpuTextureFormat::Rgba8uint,
        TextureFormatId::rgba8sint => web_sys::GpuTextureFormat::Rgba8sint,
        TextureFormatId::bgra8unorm => web_sys::GpuTextureFormat::Bgra8unorm,
        TextureFormatId::bgra8unorm_srgb => web_sys::GpuTextureFormat::Bgra8unormSrgb,
        TextureFormatId::rgb9e5ufloat => web_sys::GpuTextureFormat::Rgb9e5ufloat,
        TextureFormatId::rgb10a2unorm => web_sys::GpuTextureFormat::Rgb10a2unorm,
        TextureFormatId::rg11b10ufloat => web_sys::GpuTextureFormat::Rg11b10ufloat,
        TextureFormatId::rg32uint => web_sys::GpuTextureFormat::Rg32uint,
        TextureFormatId::rg32sint => web_sys::GpuTextureFormat::Rg32sint,
        TextureFormatId::rg32float => web_sys::GpuTextureFormat::Rg32float,
        TextureFormatId::rgba16uint => web_sys::GpuTextureFormat::Rgba16uint,
        TextureFormatId::rgba16sint => web_sys::GpuTextureFormat::Rgba16sint,
        TextureFormatId::rgba16float => web_sys::GpuTextureFormat::Rgba16float,
        TextureFormatId::rgba32uint => web_sys::GpuTextureFormat::Rgba32uint,
        TextureFormatId::rgba32sint => web_sys::GpuTextureFormat::Rgba32sint,
        TextureFormatId::rgba32float => web_sys::GpuTextureFormat::Rgba32float,
        TextureFormatId::stencil8 => web_sys::GpuTextureFormat::Stencil8,
        TextureFormatId::depth16unorm => web_sys::GpuTextureFormat::Depth16unorm,
        TextureFormatId::depth24plus => web_sys::GpuTextureFormat::Depth24plus,
        TextureFormatId::depth24plus_stencil8 => web_sys::GpuTextureFormat::Depth24plusStencil8,
        TextureFormatId::depth32float => web_sys::GpuTextureFormat::Depth32float,
        TextureFormatId::depth32float_stencil8 => web_sys::GpuTextureFormat::Depth32floatStencil8,
        TextureFormatId::bc1_rgba_unorm => web_sys::GpuTextureFormat::Bc1RgbaUnorm,
        TextureFormatId::bc1_rgba_unorm_srgb => web_sys::GpuTextureFormat::Bc1RgbaUnormSrgb,
        TextureFormatId::bc2_rgba_unorm => web_sys::GpuTextureFormat::Bc2RgbaUnorm,
        TextureFormatId::bc2_rgba_unorm_srgb => web_sys::GpuTextureFormat::Bc2RgbaUnormSrgb,
        TextureFormatId::bc3_rgba_unorm => web_sys::GpuTextureFormat::Bc3RgbaUnorm,
        TextureFormatId::bc3_rgba_unorm_srgb => web_sys::GpuTextureFormat::Bc3RgbaUnormSrgb,
        TextureFormatId::bc4_r_unorm => web_sys::GpuTextureFormat::Bc4RUnorm,
        TextureFormatId::bc4_r_snorm => web_sys::GpuTextureFormat::Bc4RSnorm,
        TextureFormatId::bc5_rg_unorm => web_sys::GpuTextureFormat::Bc5RgUnorm,
        TextureFormatId::bc5_rg_snorm => web_sys::GpuTextureFormat::Bc5RgSnorm,
        TextureFormatId::bc6h_rgb_ufloat => web_sys::GpuTextureFormat::Bc6hRgbUfloat,
        TextureFormatId::bc6h_rgb_float => web_sys::GpuTextureFormat::Bc6hRgbFloat,
        TextureFormatId::bc7_rgba_unorm => web_sys::GpuTextureFormat::Bc7RgbaUnorm,
        TextureFormatId::bc7_rgba_unorm_srgb => web_sys::GpuTextureFormat::Bc7RgbaUnormSrgb,
        TextureFormatId::etc2_rgb8unorm => web_sys::GpuTextureFormat::Etc2Rgb8unorm,
        TextureFormatId::etc2_rgb8unorm_srgb => web_sys::GpuTextureFormat::Etc2Rgb8unormSrgb,
        TextureFormatId::etc2_rgb8a1unorm => web_sys::GpuTextureFormat::Etc2Rgb8a1unorm,
        TextureFormatId::etc2_rgb8a1unorm_srgb => web_sys::GpuTextureFormat::Etc2Rgb8a1unormSrgb,
        TextureFormatId::etc2_rgba8unorm => web_sys::GpuTextureFormat::Etc2Rgba8unorm,
        TextureFormatId::etc2_rgba8unorm_srgb => web_sys::GpuTextureFormat::Etc2Rgba8unormSrgb,
        TextureFormatId::eac_r11unorm => web_sys::GpuTextureFormat::EacR11unorm,
        TextureFormatId::eac_r11snorm => web_sys::GpuTextureFormat::EacR11snorm,
        TextureFormatId::eac_rg11unorm => web_sys::GpuTextureFormat::EacRg11unorm,
        TextureFormatId::eac_rg11snorm => web_sys::GpuTextureFormat::EacRg11snorm,
        TextureFormatId::astc_4x4_unorm => web_sys::GpuTextureFormat::Astc4x4Unorm,
        TextureFormatId::astc_4x4_unorm_srgb => web_sys::GpuTextureFormat::Astc4x4UnormSrgb,
        TextureFormatId::astc_5x4_unorm => web_sys::GpuTextureFormat::Astc5x4Unorm,
        TextureFormatId::astc_5x4_unorm_srgb => web_sys::GpuTextureFormat::Astc5x4UnormSrgb,
        TextureFormatId::astc_5x5_unorm => web_sys::GpuTextureFormat::Astc5x5Unorm,
        TextureFormatId::astc_5x5_unorm_srgb => web_sys::GpuTextureFormat::Astc5x5UnormSrgb,
        TextureFormatId::astc_6x5_unorm => web_sys::GpuTextureFormat::Astc6x5Unorm,
        TextureFormatId::astc_6x5_unorm_srgb => web_sys::GpuTextureFormat::Astc6x5UnormSrgb,
        TextureFormatId::astc_6x6_unorm => web_sys::GpuTextureFormat::Astc6x6Unorm,
        TextureFormatId::astc_6x6_unorm_srgb => web_sys::GpuTextureFormat::Astc6x6UnormSrgb,
        TextureFormatId::astc_8x5_unorm => web_sys::GpuTextureFormat::Astc8x5Unorm,
        TextureFormatId::astc_8x5_unorm_srgb => web_sys::GpuTextureFormat::Astc8x5UnormSrgb,
        TextureFormatId::astc_8x6_unorm => web_sys::GpuTextureFormat::Astc8x6Unorm,
        TextureFormatId::astc_8x6_unorm_srgb => web_sys::GpuTextureFormat::Astc8x6UnormSrgb,
        TextureFormatId::astc_8x8_unorm => web_sys::GpuTextureFormat::Astc8x8Unorm,
        TextureFormatId::astc_8x8_unorm_srgb => web_sys::GpuTextureFormat::Astc8x8UnormSrgb,
        TextureFormatId::astc_10x5_unorm => web_sys::GpuTextureFormat::Astc10x5Unorm,
        TextureFormatId::astc_10x5_unorm_srgb => web_sys::GpuTextureFormat::Astc10x5UnormSrgb,
        TextureFormatId::astc_10x6_unorm => web_sys::GpuTextureFormat::Astc10x6Unorm,
        TextureFormatId::astc_10x6_unorm_srgb => web_sys::GpuTextureFormat::Astc10x6UnormSrgb,
        TextureFormatId::astc_10x8_unorm => web_sys::GpuTextureFormat::Astc10x8Unorm,
        TextureFormatId::astc_10x8_unorm_srgb => web_sys::GpuTextureFormat::Astc10x8UnormSrgb,
        TextureFormatId::astc_10x10_unorm => web_sys::GpuTextureFormat::Astc10x10Unorm,
        TextureFormatId::astc_10x10_unorm_srgb => web_sys::GpuTextureFormat::Astc10x10UnormSrgb,
        TextureFormatId::astc_12x10_unorm => web_sys::GpuTextureFormat::Astc12x10Unorm,
        TextureFormatId::astc_12x10_unorm_srgb => web_sys::GpuTextureFormat::Astc12x10UnormSrgb,
        TextureFormatId::astc_12x12_unorm => web_sys::GpuTextureFormat::Astc12x12Unorm,
        TextureFormatId::astc_12x12_unorm_srgb => web_sys::GpuTextureFormat::Astc12x12UnormSrgb,
    }
}

pub fn texture_format_to_str(texture_format: &TextureFormatId) -> &str {
    match texture_format {
        TextureFormatId::r8unorm => "r8unorm",
        TextureFormatId::r8snorm => "r8snorm",
        TextureFormatId::r8uint => "r8uint",
        TextureFormatId::r8sint => "r8sint",
        TextureFormatId::r16uint => "r16uint",
        TextureFormatId::r16sint => "r16sint",
        TextureFormatId::r16float => "r16float",
        TextureFormatId::rg8unorm => "rg8unorm",
        TextureFormatId::rg8snorm => "rg8snorm",
        TextureFormatId::rg8uint => "rg8uint",
        TextureFormatId::rg8sint => "rg8sint",
        TextureFormatId::r32uint => "r32uint",
        TextureFormatId::r32sint => "r32sint",
        TextureFormatId::r32float => "r32float",
        TextureFormatId::rg16uint => "rg16uint",
        TextureFormatId::rg16sint => "rg16sint",
        TextureFormatId::rg16float => "rg16float",
        TextureFormatId::rgba8unorm => "rgba8unorm",
        TextureFormatId::rgba8unorm_srgb => "rgba8unorm-srgb",
        TextureFormatId::rgba8snorm => "rgba8snorm",
        TextureFormatId::rgba8uint => "rgba8uint",
        TextureFormatId::rgba8sint => "rgba8sint",
        TextureFormatId::bgra8unorm => "bgra8unorm",
        TextureFormatId::bgra8unorm_srgb => "bgra8unorm-srgb",
        TextureFormatId::rgb9e5ufloat => "rgb9e5ufloat",
        TextureFormatId::rgb10a2unorm => "rgb10a2unorm",
        TextureFormatId::rg11b10ufloat => "rg11b10ufloat",
        TextureFormatId::rg32uint => "rg32uint",
        TextureFormatId::rg32sint => "rg32sint",
        TextureFormatId::rg32float => "rg32float",
        TextureFormatId::rgba16uint => "rgba16uint",
        TextureFormatId::rgba16sint => "rgba16sint",
        TextureFormatId::rgba16float => "rgba16float",
        TextureFormatId::rgba32uint => "rgba32uint",
        TextureFormatId::rgba32sint => "rgba32sint",
        TextureFormatId::rgba32float => "rgba32float",
        TextureFormatId::stencil8 => "stencil8",
        TextureFormatId::depth16unorm => "depth16unorm",
        TextureFormatId::depth24plus => "depth24plus",
        TextureFormatId::depth24plus_stencil8 => "depth24plus-stencil8",
        TextureFormatId::depth32float => "depth32float",
        TextureFormatId::depth32float_stencil8 => "depth32float-stencil8",
        TextureFormatId::bc1_rgba_unorm => "bc1-rgba-unorm",
        TextureFormatId::bc1_rgba_unorm_srgb => "bc1-rgba-unorm-srgb",
        TextureFormatId::bc2_rgba_unorm => "bc2-rgba-unorm",
        TextureFormatId::bc2_rgba_unorm_srgb => "bc2-rgba-unorm-srgb",
        TextureFormatId::bc3_rgba_unorm => "bc3-rgba-unorm",
        TextureFormatId::bc3_rgba_unorm_srgb => "bc3-rgba-unorm-srgb",
        TextureFormatId::bc4_r_unorm => "bc4-r-unorm",
        TextureFormatId::bc4_r_snorm => "bc4-r-snorm",
        TextureFormatId::bc5_rg_unorm => "bc5-rg-unorm",
        TextureFormatId::bc5_rg_snorm => "bc5-rg-snorm",
        TextureFormatId::bc6h_rgb_ufloat => "bc6-rgb-ufloat",
        TextureFormatId::bc6h_rgb_float => "bc6h-rgb-float",
        TextureFormatId::bc7_rgba_unorm => "bc7-rgba-unorm",
        TextureFormatId::bc7_rgba_unorm_srgb => "bc7-rgba-unorm-srgb",
        TextureFormatId::etc2_rgb8unorm => "etc2-rgb8unorm",
        TextureFormatId::etc2_rgb8unorm_srgb => "etc2-rgb8unorm-srgb",
        TextureFormatId::etc2_rgb8a1unorm => "etc2-rgb8a1unorm",
        TextureFormatId::etc2_rgb8a1unorm_srgb => "etc2-rgb8a1unorm-srgb",
        TextureFormatId::etc2_rgba8unorm => "etc2-rgba8unorm",
        TextureFormatId::etc2_rgba8unorm_srgb => "etc2-rgba8unorm-srgb",
        TextureFormatId::eac_r11unorm => "eac-r11unorm",
        TextureFormatId::eac_r11snorm => "eac-r11snorm",
        TextureFormatId::eac_rg11unorm => "eac-rg11unorm",
        TextureFormatId::eac_rg11snorm => "eac-rg11snorm",
        TextureFormatId::astc_4x4_unorm => "astc-4x4-unorm",
        TextureFormatId::astc_4x4_unorm_srgb => "astc-4x4-unorm-srgb",
        TextureFormatId::astc_5x4_unorm => "astc-5x4-unorm",
        TextureFormatId::astc_5x4_unorm_srgb => "astc-5x4-unorm-srgb",
        TextureFormatId::astc_5x5_unorm => "astc-5x5-unorm",
        TextureFormatId::astc_5x5_unorm_srgb => "astc-5x5-unorm-srgb",
        TextureFormatId::astc_6x5_unorm => "astc-6x5-unorm",
        TextureFormatId::astc_6x5_unorm_srgb => "astc-6x5-unorm-srgb",
        TextureFormatId::astc_6x6_unorm => "astc-6x6-unorm",
        TextureFormatId::astc_6x6_unorm_srgb => "astc-6x6-unorm-srgb",
        TextureFormatId::astc_8x5_unorm => "astc-8x5-unorm",
        TextureFormatId::astc_8x5_unorm_srgb => "astc-8x5-unorm-srgb",
        TextureFormatId::astc_8x6_unorm => "astc-8x6-unorm",
        TextureFormatId::astc_8x6_unorm_srgb => "astc-8x6-unorm-srgb",
        TextureFormatId::astc_8x8_unorm => "astc-8x8-unorm",
        TextureFormatId::astc_8x8_unorm_srgb => "astc-8x8-unorm-srgb",
        TextureFormatId::astc_10x5_unorm => "astc-10x5-unorm",
        TextureFormatId::astc_10x5_unorm_srgb => "astc-10x5-unorm-srgb",
        TextureFormatId::astc_10x6_unorm => "astc-10x6-unorm",
        TextureFormatId::astc_10x6_unorm_srgb => "astc-10x6-unorm-srgb",
        TextureFormatId::astc_10x8_unorm => "astc-10x8-unorm",
        TextureFormatId::astc_10x8_unorm_srgb => "astc-10x8-unorm-srgb",
        TextureFormatId::astc_10x10_unorm => "astc-10x10-unorm",
        TextureFormatId::astc_10x10_unorm_srgb => "astc-10x10-unorm_srgb",
        TextureFormatId::astc_12x10_unorm => "astc-12x10-unorm",
        TextureFormatId::astc_12x10_unorm_srgb => "astc-12x10-unorm_srgb",
        TextureFormatId::astc_12x12_unorm => "astc-12x12-unorm",
        TextureFormatId::astc_12x12_unorm_srgb => "astc-12x12-unorm_srgb",
    }
}

pub fn texture_aspect_to_web_sys(texture_aspect: &TextureAspect) -> web_sys::GpuTextureAspect {
    match texture_aspect {
        TextureAspect::All => web_sys::GpuTextureAspect::All,
        TextureAspect::StencilOnly => web_sys::GpuTextureAspect::StencilOnly,
        TextureAspect::DepthOnly => web_sys::GpuTextureAspect::DepthOnly,
    }
}

pub fn texture_dimension_to_web_sys(
    texture_dimension: &TextureDimensions,
) -> web_sys::GpuTextureDimension {
    match texture_dimension {
        TextureDimensions::One => web_sys::GpuTextureDimension::N1d,
        TextureDimensions::Two => web_sys::GpuTextureDimension::N2d,
        TextureDimensions::Three => web_sys::GpuTextureDimension::N3d,
    }
}

pub fn texture_view_dimension_to_web_sys(
    texture_view_dimension: &TextureViewDimension,
) -> web_sys::GpuTextureViewDimension {
    match texture_view_dimension {
        TextureViewDimension::One => web_sys::GpuTextureViewDimension::N1d,
        TextureViewDimension::Two => web_sys::GpuTextureViewDimension::N2d,
        TextureViewDimension::Three => web_sys::GpuTextureViewDimension::N3d,
        TextureViewDimension::TwoArray => web_sys::GpuTextureViewDimension::N2dArray,
        TextureViewDimension::Cube => web_sys::GpuTextureViewDimension::Cube,
        TextureViewDimension::CubeArray => web_sys::GpuTextureViewDimension::CubeArray,
    }
}

pub fn pipeline_constants_to_web_sys(pipeline_constants: &HashMap<String, f64>) -> js_sys::Object {
    let record = js_sys::Object::new();

    for (identifier, value) in pipeline_constants {
        js_sys::Reflect::set(
            record.as_ref(),
            &JsValue::from_str(identifier),
            &JsValue::from(*value),
        )
        .unwrap_throw();
    }

    record
}

pub fn compute_pipeline_descriptor_to_web_sys(
    descriptor: &ComputePipelineDescriptor<Driver>,
) -> web_sys::GpuComputePipelineDescriptor {
    let mut compute_stage = web_sys::GpuProgrammableStage::new(&descriptor.shader_module.inner);

    compute_stage.entry_point(descriptor.entry_point);

    js_sys::Reflect::set(
        compute_stage.as_ref(),
        &JsValue::from("constants"),
        pipeline_constants_to_web_sys(descriptor.constants).as_ref(),
    )
    .unwrap_throw();

    web_sys::GpuComputePipelineDescriptor::new(descriptor.layout.inner.as_ref(), &compute_stage)
}

pub fn stencil_operation_to_web_sys(
    stencil_operation: &StencilOperation,
) -> web_sys::GpuStencilOperation {
    match stencil_operation {
        StencilOperation::Keep => web_sys::GpuStencilOperation::Keep,
        StencilOperation::Zero => web_sys::GpuStencilOperation::Zero,
        StencilOperation::Replace => web_sys::GpuStencilOperation::Replace,
        StencilOperation::Invert => web_sys::GpuStencilOperation::Invert,
        StencilOperation::IncrementClamp => web_sys::GpuStencilOperation::IncrementClamp,
        StencilOperation::DecrementClamp => web_sys::GpuStencilOperation::DecrementClamp,
        StencilOperation::IncrementWrap => web_sys::GpuStencilOperation::IncrementWrap,
        StencilOperation::DecrementWrap => web_sys::GpuStencilOperation::DecrementWrap,
    }
}

pub fn stencil_face_state_to_web_sys(
    stencil_face_state: &StencilFaceState,
) -> web_sys::GpuStencilFaceState {
    let StencilFaceState {
        compare,
        depth_fail_op,
        fail_op,
        pass_op,
    } = stencil_face_state;

    let mut state = web_sys::GpuStencilFaceState::new();

    state.compare(compare_function_to_web_sys(compare));
    state.depth_fail_op(stencil_operation_to_web_sys(depth_fail_op));
    state.fail_op(stencil_operation_to_web_sys(fail_op));
    state.pass_op(stencil_operation_to_web_sys(pass_op));

    state
}

pub fn depth_stencil_state_to_web_sys(
    depth_stencil_state: &DepthStencilState,
) -> web_sys::GpuDepthStencilState {
    let DepthStencilState {
        format,
        depth_write_enabled,
        depth_compare,
        stencil_front,
        stencil_back,
        stencil_read_mask,
        stencil_write_mask,
        depth_bias,
        depth_bias_slope_scale,
        depth_bias_clamp,
    } = depth_stencil_state;

    let mut state = web_sys::GpuDepthStencilState::new(texture_format_to_web_sys(format));

    state.depth_bias(*depth_bias);
    state.depth_bias_clamp(*depth_bias_clamp);
    state.depth_bias_slope_scale(*depth_bias_slope_scale);
    state.depth_compare(compare_function_to_web_sys(depth_compare));
    state.depth_write_enabled(*depth_write_enabled);
    state.stencil_back(&stencil_face_state_to_web_sys(stencil_back));
    state.stencil_front(&stencil_face_state_to_web_sys(stencil_front));
    state.stencil_read_mask(*stencil_read_mask);
    state.stencil_write_mask(*stencil_write_mask);

    state
}

pub fn multisample_state_to_web_sys(
    multisample_state: &MultisampleState,
) -> web_sys::GpuMultisampleState {
    let MultisampleState {
        count,
        mask,
        alpha_to_coverage_enabled,
    } = *multisample_state;

    let mut state = web_sys::GpuMultisampleState::new();

    state.alpha_to_coverage_enabled(alpha_to_coverage_enabled);
    state.count(count);
    state.mask(mask);

    state
}

pub fn index_format_to_web_sys(index_format: &IndexFormat) -> web_sys::GpuIndexFormat {
    match index_format {
        IndexFormat::U16 => web_sys::GpuIndexFormat::Uint16,
        IndexFormat::U32 => web_sys::GpuIndexFormat::Uint32,
    }
}

pub fn primitive_topology_to_web_sys(
    primitive_topology: &PrimitiveTopology,
) -> web_sys::GpuPrimitiveTopology {
    match primitive_topology {
        PrimitiveTopology::PointList => web_sys::GpuPrimitiveTopology::PointList,
        PrimitiveTopology::LineList => web_sys::GpuPrimitiveTopology::LineList,
        PrimitiveTopology::LineStrip => web_sys::GpuPrimitiveTopology::LineStrip,
        PrimitiveTopology::TriangleList => web_sys::GpuPrimitiveTopology::TriangleList,
        PrimitiveTopology::TriangleStrip => web_sys::GpuPrimitiveTopology::TriangleStrip,
    }
}

pub fn front_face_to_web_sys(front_face: &FrontFace) -> web_sys::GpuFrontFace {
    match front_face {
        FrontFace::Clockwise => web_sys::GpuFrontFace::Cw,
        FrontFace::CounterClockwise => web_sys::GpuFrontFace::Ccw,
    }
}

pub fn cull_mode_to_web_sys(cull_mode: &CullMode) -> web_sys::GpuCullMode {
    match cull_mode {
        CullMode::Front => web_sys::GpuCullMode::Front,
        CullMode::Back => web_sys::GpuCullMode::Back,
    }
}

pub fn primitive_state_to_web_sys(primitive_state: &PrimitiveState) -> web_sys::GpuPrimitiveState {
    let PrimitiveState {
        topology,
        strip_index_format,
        front_face,
        cull_mode,
    } = primitive_state;

    let mut state = web_sys::GpuPrimitiveState::new();

    state.topology(primitive_topology_to_web_sys(topology));
    state.front_face(front_face_to_web_sys(front_face));

    if let Some(cull_mode) = cull_mode {
        state.cull_mode(cull_mode_to_web_sys(cull_mode));
    }

    if let Some(strip_index_format) = strip_index_format {
        state.strip_index_format(index_format_to_web_sys(strip_index_format));
    }

    state
}

pub fn vertex_format_to_web_sys(vertex_format: &VertexFormat) -> web_sys::GpuVertexFormat {
    match vertex_format {
        VertexFormat::uint8x2 => web_sys::GpuVertexFormat::Uint8x2,
        VertexFormat::uint8x4 => web_sys::GpuVertexFormat::Uint8x4,
        VertexFormat::sint8x2 => web_sys::GpuVertexFormat::Sint8x2,
        VertexFormat::sint8x4 => web_sys::GpuVertexFormat::Sint8x4,
        VertexFormat::unorm8x2 => web_sys::GpuVertexFormat::Unorm8x2,
        VertexFormat::unorm8x4 => web_sys::GpuVertexFormat::Unorm8x4,
        VertexFormat::snorm8x2 => web_sys::GpuVertexFormat::Snorm8x2,
        VertexFormat::snorm8x4 => web_sys::GpuVertexFormat::Snorm8x4,
        VertexFormat::uint16x2 => web_sys::GpuVertexFormat::Uint16x2,
        VertexFormat::uint16x4 => web_sys::GpuVertexFormat::Uint16x4,
        VertexFormat::sint16x2 => web_sys::GpuVertexFormat::Sint16x2,
        VertexFormat::sint16x4 => web_sys::GpuVertexFormat::Sint16x4,
        VertexFormat::unorm16x2 => web_sys::GpuVertexFormat::Unorm16x2,
        VertexFormat::unorm16x4 => web_sys::GpuVertexFormat::Unorm16x4,
        VertexFormat::snorm16x2 => web_sys::GpuVertexFormat::Snorm16x2,
        VertexFormat::snorm16x4 => web_sys::GpuVertexFormat::Snorm16x4,
        VertexFormat::float16x2 => web_sys::GpuVertexFormat::Float16x2,
        VertexFormat::float16x4 => web_sys::GpuVertexFormat::Float16x4,
        VertexFormat::float32 => web_sys::GpuVertexFormat::Float32,
        VertexFormat::float32x2 => web_sys::GpuVertexFormat::Float32x2,
        VertexFormat::float32x3 => web_sys::GpuVertexFormat::Float32x3,
        VertexFormat::float32x4 => web_sys::GpuVertexFormat::Float32x4,
        VertexFormat::uint32 => web_sys::GpuVertexFormat::Uint32,
        VertexFormat::uint32x2 => web_sys::GpuVertexFormat::Uint32x2,
        VertexFormat::uint32x3 => web_sys::GpuVertexFormat::Uint32x3,
        VertexFormat::uint32x4 => web_sys::GpuVertexFormat::Uint32x4,
        VertexFormat::sint32 => web_sys::GpuVertexFormat::Sint32,
        VertexFormat::sint32x2 => web_sys::GpuVertexFormat::Sint32x2,
        VertexFormat::sint32x3 => web_sys::GpuVertexFormat::Sint32x3,
        VertexFormat::sint32x4 => web_sys::GpuVertexFormat::Sint32x4,
    }
}

pub fn blend_factor_to_web_sys(blend_factor: &BlendFactor) -> web_sys::GpuBlendFactor {
    match blend_factor {
        BlendFactor::Zero => web_sys::GpuBlendFactor::Zero,
        BlendFactor::One => web_sys::GpuBlendFactor::One,
        BlendFactor::Src => web_sys::GpuBlendFactor::Src,
        BlendFactor::OneMinusSrc => web_sys::GpuBlendFactor::OneMinusSrc,
        BlendFactor::SrcAlpha => web_sys::GpuBlendFactor::SrcAlpha,
        BlendFactor::OneMinusSrcAlpha => web_sys::GpuBlendFactor::OneMinusSrcAlpha,
        BlendFactor::Dst => web_sys::GpuBlendFactor::Dst,
        BlendFactor::OneMinusDst => web_sys::GpuBlendFactor::OneMinusDst,
        BlendFactor::DstAlpha => web_sys::GpuBlendFactor::DstAlpha,
        BlendFactor::OneMinusDstAlpha => web_sys::GpuBlendFactor::OneMinusDstAlpha,
        BlendFactor::SrcAlphaSaturated => web_sys::GpuBlendFactor::SrcAlphaSaturated,
        BlendFactor::Constant => web_sys::GpuBlendFactor::Constant,
        BlendFactor::OneMinusConstant => web_sys::GpuBlendFactor::OneMinusConstant,
    }
}

pub fn blend_component_to_web_sys(blend_component: &BlendComponent) -> web_sys::GpuBlendComponent {
    let mut blend = web_sys::GpuBlendComponent::new();

    match blend_component {
        BlendComponent::Add {
            src_factor,
            dst_factor,
        } => {
            blend.operation(web_sys::GpuBlendOperation::Add);
            blend.src_factor(blend_factor_to_web_sys(src_factor));
            blend.dst_factor(blend_factor_to_web_sys(dst_factor));
        }
        BlendComponent::Subtract {
            src_factor,
            dst_factor,
        } => {
            blend.operation(web_sys::GpuBlendOperation::Subtract);
            blend.src_factor(blend_factor_to_web_sys(src_factor));
            blend.dst_factor(blend_factor_to_web_sys(dst_factor));
        }
        BlendComponent::ReverseSubtract {
            src_factor,
            dst_factor,
        } => {
            blend.operation(web_sys::GpuBlendOperation::ReverseSubtract);
            blend.src_factor(blend_factor_to_web_sys(src_factor));
            blend.dst_factor(blend_factor_to_web_sys(dst_factor));
        }
        BlendComponent::Min => {
            blend.operation(web_sys::GpuBlendOperation::Min);
            blend.src_factor(web_sys::GpuBlendFactor::One);
            blend.dst_factor(web_sys::GpuBlendFactor::One);
        }
        BlendComponent::Max => {
            blend.operation(web_sys::GpuBlendOperation::Max);
            blend.src_factor(web_sys::GpuBlendFactor::One);
            blend.dst_factor(web_sys::GpuBlendFactor::One);
        }
    }

    blend
}

pub fn blend_state_to_web_sys(blend_state: &BlendState) -> web_sys::GpuBlendState {
    let alpha = blend_component_to_web_sys(&blend_state.alpha);
    let color = blend_component_to_web_sys(&blend_state.color);

    web_sys::GpuBlendState::new(&alpha, &color)
}

pub fn vertex_step_mode_to_web_sys(step_mode: &VertexStepMode) -> web_sys::GpuVertexStepMode {
    match step_mode {
        VertexStepMode::Vertex => web_sys::GpuVertexStepMode::Vertex,
        VertexStepMode::Instance => web_sys::GpuVertexStepMode::Instance,
    }
}

pub fn vertex_state_to_web_sys(vertex_state: &VertexState<Driver>) -> web_sys::GpuVertexState {
    let layouts = js_sys::Array::new();

    for buffer_layout in vertex_state.vertex_buffer_layouts {
        let attributes: js_sys::Array = buffer_layout
            .attributes
            .iter()
            .map(|a| {
                web_sys::GpuVertexAttribute::new(
                    vertex_format_to_web_sys(&a.format),
                    a.offset as f64,
                    a.shader_location,
                )
            })
            .collect();

        let mut web_sys_layout = web_sys::GpuVertexBufferLayout::new(
            buffer_layout.array_stride as f64,
            attributes.as_ref(),
        );

        web_sys_layout.step_mode(vertex_step_mode_to_web_sys(&buffer_layout.step_mode));

        layouts.push(web_sys_layout.as_ref());
    }

    let mut web_sys_state = web_sys::GpuVertexState::new(&vertex_state.shader_module.inner);

    web_sys_state.entry_point(vertex_state.entry_point);
    web_sys_state.buffers(layouts.as_ref());

    js_sys::Reflect::set(
        web_sys_state.as_ref(),
        &JsValue::from("constants"),
        pipeline_constants_to_web_sys(vertex_state.constants).as_ref(),
    )
    .unwrap_throw();

    web_sys_state
}

pub fn color_target_state_to_web_sys(
    color_target_state: &ColorTargetState,
) -> web_sys::GpuColorTargetState {
    let mut target =
        web_sys::GpuColorTargetState::new(texture_format_to_web_sys(&color_target_state.format));

    target.write_mask(color_target_state.write_mask.bits());

    if let Some(blend) = &color_target_state.blend {
        target.blend(&blend_state_to_web_sys(blend));
    }

    target
}

pub fn fragment_state_to_web_sys(
    fragment_state: &FragmentState<Driver>,
) -> web_sys::GpuFragmentState {
    let targets: js_sys::Array = fragment_state
        .targets
        .iter()
        .map(color_target_state_to_web_sys)
        .collect();

    let mut state =
        web_sys::GpuFragmentState::new(&fragment_state.shader_module.inner, targets.as_ref());

    state.entry_point(fragment_state.entry_point);

    state
}

pub fn render_pipeline_descriptor_to_web_sys(
    descriptor: &RenderPipelineDescriptor<Driver>,
) -> web_sys::GpuRenderPipelineDescriptor {
    let RenderPipelineDescriptor {
        layout,
        primitive_state,
        vertex_state,
        depth_stencil_state,
        fragment_state,
        multisample_state,
    } = descriptor;

    let mut desc = web_sys::GpuRenderPipelineDescriptor::new(
        layout.inner.as_ref(),
        &vertex_state_to_web_sys(vertex_state),
    );

    desc.primitive(&primitive_state_to_web_sys(primitive_state));

    if let Some(depth_stencil_state) = depth_stencil_state {
        desc.depth_stencil(&depth_stencil_state_to_web_sys(depth_stencil_state));
    }

    if let Some(fragment_state) = fragment_state {
        desc.fragment(&fragment_state_to_web_sys(fragment_state));
    }

    if let Some(multisample_state) = multisample_state {
        desc.multisample(&multisample_state_to_web_sys(multisample_state));
    }

    desc
}

pub fn load_op_to_web_sys<T>(load_op: &LoadOp<T>) -> web_sys::GpuLoadOp {
    match load_op {
        LoadOp::Load => web_sys::GpuLoadOp::Load,
        LoadOp::Clear(_) => web_sys::GpuLoadOp::Clear,
    }
}

pub fn store_op_to_web_sys(store_op: &StoreOp) -> web_sys::GpuStoreOp {
    match store_op {
        StoreOp::Store => web_sys::GpuStoreOp::Store,
        StoreOp::Discard => web_sys::GpuStoreOp::Discard,
    }
}

pub fn render_pass_color_attachment_to_web_sys(
    attachment: &RenderPassColorAttachment<Driver>,
) -> web_sys::GpuRenderPassColorAttachment {
    let RenderPassColorAttachment {
        view,
        resolve_target,
        load_op,
        store_op,
    } = attachment;

    let mut attachment = web_sys::GpuRenderPassColorAttachment::new(
        load_op_to_web_sys(load_op),
        store_op_to_web_sys(store_op),
        &view.inner,
    );

    if let LoadOp::Clear(clear_value) = load_op {
        let clear_value = web_sys::GpuColorDict::new(
            clear_value[0],
            clear_value[1],
            clear_value[2],
            clear_value[3],
        );

        attachment.clear_value(clear_value.as_ref());
    }

    if let Some(resolve_target) = resolve_target {
        attachment.resolve_target(&resolve_target.inner);
    }

    attachment
}

pub fn render_pass_depth_stencil_attachment_to_web_sys(
    attachment: &RenderPassDepthStencilAttachment<Driver>,
) -> web_sys::GpuRenderPassDepthStencilAttachment {
    let RenderPassDepthStencilAttachment {
        view,
        depth_operations,
        stencil_operations,
    } = attachment;

    let mut attachment = web_sys::GpuRenderPassDepthStencilAttachment::new(&view.inner);

    if let Some(DepthStencilOperations { load_op, store_op }) = depth_operations {
        attachment.depth_load_op(load_op_to_web_sys(load_op));
        attachment.depth_store_op(store_op_to_web_sys(store_op));

        if let LoadOp::Clear(clear_value) = load_op {
            attachment.depth_clear_value(*clear_value);
        }
    } else {
        attachment.depth_read_only(true);
    }

    if let Some(DepthStencilOperations { load_op, store_op }) = stencil_operations {
        attachment.stencil_load_op(load_op_to_web_sys(load_op));
        attachment.stencil_store_op(store_op_to_web_sys(store_op));

        if let LoadOp::Clear(clear_value) = load_op {
            attachment.stencil_clear_value(*clear_value);
        }
    } else {
        attachment.stencil_read_only(true);
    }

    attachment
}

pub fn query_type_to_web_sys(query_type: &QueryType) -> web_sys::GpuQueryType {
    match query_type {
        QueryType::Occlusion => web_sys::GpuQueryType::Occlusion,
        QueryType::Timestamp => web_sys::GpuQueryType::Timestamp,
    }
}

pub fn features_to_web_sys(features: &FlagSet<Feature>) -> js_sys::Array {
    let array = js_sys::Array::new();

    if features.contains(Feature::DepthClipControl) {
        array.push(&JsValue::from("depth-clip-control"));
    }

    if features.contains(Feature::Depth24UNormStencil8) {
        array.push(&JsValue::from("depth24unorm-stencil8"));
    }

    if features.contains(Feature::Depth32FloatStencil8) {
        array.push(&JsValue::from("depth32float-stencil8"));
    }

    if features.contains(Feature::TextureCompressionBc) {
        array.push(&JsValue::from("texture-compression-bc"));
    }

    if features.contains(Feature::TextureComporessionEtc2) {
        array.push(&JsValue::from("texture-compression-etc2"));
    }

    if features.contains(Feature::TextureCompressionAstc) {
        array.push(&JsValue::from("texture-compression-astc"));
    }

    if features.contains(Feature::TimestampQuery) {
        array.push(&JsValue::from("timestamp-query"));
    }

    if features.contains(Feature::IndirectFirstInstance) {
        array.push(&JsValue::from("indirect-first-instance"));
    }

    if features.contains(Feature::ShaderF16) {
        array.push(&JsValue::from("shader-f16"));
    }

    if features.contains(Feature::Bgra8UNormStorage) {
        array.push(&JsValue::from("bgra8unorm-storage"));
    }

    array
}

pub fn features_from_web_sys(raw: &GpuSupportedFeatures) -> FlagSet<Feature> {
    let mut features = FlagSet::from(Feature::None);

    if js_sys::Reflect::has(raw.as_ref(), &JsValue::from("depth-clip-control")).unwrap_or(false) {
        features |= Feature::DepthClipControl;
    }

    if js_sys::Reflect::has(raw.as_ref(), &JsValue::from("depth24unorm-stencil8")).unwrap_or(false)
    {
        features |= Feature::Depth24UNormStencil8;
    }

    if js_sys::Reflect::has(raw.as_ref(), &JsValue::from("depth32float-stencil8")).unwrap_or(false)
    {
        features |= Feature::Depth32FloatStencil8;
    }

    if js_sys::Reflect::has(raw.as_ref(), &JsValue::from("texture-compression-bc")).unwrap_or(false)
    {
        features |= Feature::TextureCompressionBc;
    }

    if js_sys::Reflect::has(raw.as_ref(), &JsValue::from("texture-compression-etc2"))
        .unwrap_or(false)
    {
        features |= Feature::TextureComporessionEtc2;
    }

    if js_sys::Reflect::has(raw.as_ref(), &JsValue::from("texture-compression-astc"))
        .unwrap_or(false)
    {
        features |= Feature::TextureCompressionAstc;
    }

    if js_sys::Reflect::has(raw.as_ref(), &JsValue::from("timestamp-query")).unwrap_or(false) {
        features |= Feature::TimestampQuery;
    }

    if js_sys::Reflect::has(raw.as_ref(), &JsValue::from("indirect-first-instance"))
        .unwrap_or(false)
    {
        features |= Feature::IndirectFirstInstance;
    }

    if js_sys::Reflect::has(raw.as_ref(), &JsValue::from("shader-f16")).unwrap_or(false) {
        features |= Feature::ShaderF16;
    }

    if js_sys::Reflect::has(raw.as_ref(), &JsValue::from("bgra8unorm-storage")).unwrap_or(false) {
        features |= Feature::Bgra8UNormStorage;
    }

    features
}

pub fn limits_from_web_sys(limits: &web_sys::GpuSupportedLimits) -> Limits {
    Limits {
        max_texture_dimension_1d: limits.max_texture_dimension_1d(),
        max_texture_dimension_2d: limits.max_texture_dimension_2d(),
        max_texture_dimension_3d: limits.max_texture_dimension_3d(),
        max_texture_array_layers: limits.max_texture_array_layers(),
        max_bind_groups: limits.max_bind_groups(),
        max_bindings_per_bind_group: limits.max_bindings_per_bind_group(),
        max_dynamic_uniform_buffers_per_pipeline_layout: limits
            .max_dynamic_uniform_buffers_per_pipeline_layout(),
        max_dynamic_storage_buffers_per_pipeline_layout: limits
            .max_dynamic_storage_buffers_per_pipeline_layout(),
        max_sampled_textures_per_shader_stage: limits.max_sampled_textures_per_shader_stage(),
        max_samplers_per_shader_stage: limits.max_samplers_per_shader_stage(),
        max_storage_buffers_per_shader_stage: limits.max_storage_buffers_per_shader_stage(),
        max_storage_textures_per_shader_stage: limits.max_storage_textures_per_shader_stage(),
        max_uniform_buffers_per_shader_stage: limits.max_uniform_buffers_per_shader_stage(),
        max_uniform_buffer_binding_size: limits.max_uniform_buffer_binding_size() as u64,
        max_storage_buffer_binding_size: limits.max_storage_buffer_binding_size() as u64,
        min_uniform_buffer_offset_alignment: limits.min_uniform_buffer_offset_alignment(),
        min_storage_buffer_offset_alignment: limits.min_storage_buffer_offset_alignment(),
        max_vertex_buffers: limits.max_vertex_buffers(),
        max_buffer_size: limits.max_buffer_size() as u64,
        max_vertex_attributes: limits.max_vertex_attributes(),
        max_vertex_buffer_array_stride: limits.max_vertex_buffer_array_stride(),
        max_inter_stage_shader_components: limits.max_inter_stage_shader_components(),
        max_color_attachments: limits.max_color_attachments(),
        max_color_attachment_bytes_per_sample: limits.max_color_attachment_bytes_per_sample(),
        max_compute_workgroup_storage_size: limits.max_compute_workgroup_storage_size(),
        max_compute_invocations_per_workgroup: limits.max_compute_invocations_per_workgroup(),
        max_compute_workgroup_size_x: limits.max_compute_workgroup_size_x(),
        max_compute_workgroup_size_y: limits.max_compute_workgroup_size_y(),
        max_compute_workgroup_size_z: limits.max_compute_workgroup_size_z(),
        max_compute_workgroups_per_dimension: limits.max_compute_workgroups_per_dimension(),
    }
}

#[wasm_bindgen(module = "/src/js_support.js")]
extern "C" {
    #[wasm_bindgen(js_name = __empa_js_copy_buffer_to_memory)]
    fn copy_buffer_to_memory(
        buffer: &Uint8Array,
        offset: u32,
        size: u32,
        wasm_memory: &JsValue,
        pointer: *mut (),
    );

    #[wasm_bindgen(js_name = __empa_write_timestamp)]
    fn write_timestamp(
        encoder: &web_sys::GpuCommandEncoder,
        query_set: &web_sys::GpuQuerySet,
        index: u32,
    );
}
