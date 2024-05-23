use std::borrow::Borrow;
use std::collections::HashMap;
use std::error::Error;
use std::future::Future;
use std::ops::{Range, RangeInclusive};

use flagset::{flags, FlagSet};

use crate::adapter::{Feature, Limits};
use crate::buffer::MapError;
use crate::command::{BlendConstant, Draw, DrawIndexed, ScissorRect, Viewport};
use crate::device::DeviceDescriptor;
use crate::render_pipeline::{
    BlendState, ColorWrite, CullMode, FrontFace, IndexFormat, VertexBufferLayout,
};
use crate::render_target::{LoadOp, StoreOp};
use crate::sampler::{AddressMode, FilterMode};
use crate::texture::format::TextureFormatId;
use crate::CompareFunction;

pub trait Driver: Sized {
    type AdapterHandle: Adapter<Self> + 'static;
    type BindGroupHandle: Clone + 'static;
    type DeviceHandle: Device<Self> + 'static;
    type BufferHandle: Buffer<Self> + 'static;
    type BufferBinding<'a>: Clone;
    type TextureHandle: Texture<Self> + 'static;
    type TextureView<'a>: Clone;
    type CommandEncoderHandle: CommandEncoder<Self> + 'static;
    type ComputePassEncoderHandle: ComputePassEncoder<Self> + 'static;
    type RenderPassEncoderHandle: RenderPassEncoder<Self> + 'static;
    type ExecuteRenderBundlesEncoder<'a>: ExecuteRenderBundlesEncoder<Self> + 'a;
    type RenderBundleEncoderHandle: RenderBundleEncoder<Self> + 'static;
    type CommandBufferHandle: Clone + 'static;
    type RenderBundleHandle: Clone + 'static;
    type QueueHandle: Queue<Self> + 'static;
    type SamplerHandle: Clone + 'static;
    type BindGroupLayoutHandle: Clone + 'static;
    type PipelineLayoutHandle: Clone + 'static;
    type ComputePipelineHandle: Clone + 'static;
    type RenderPipelineHandle: Clone + 'static;
    type QuerySetHandle: Clone + 'static;
    type ShaderModuleHandle: Clone + 'static;
}

pub trait Adapter<D>: Clone + Sized
where
    D: Driver,
{
    type RequestDevice: Future<Output = Result<D::DeviceHandle, Box<dyn Error>>>;

    fn supported_features(&self) -> FlagSet<Feature>;

    fn supported_limits(&self) -> Limits;

    fn request_device(&self, descriptor: &DeviceDescriptor) -> Self::RequestDevice;
}

pub trait Device<D>: Clone + Sized
where
    D: Driver,
{
    type CreateComputePipelineAsync: Future<Output = D::ComputePipelineHandle>;

    type CreateRenderPipelineAsync: Future<Output = D::RenderPipelineHandle>;

    fn create_buffer(&self, descriptor: &BufferDescriptor) -> D::BufferHandle;

    fn create_texture(&self, descriptor: &TextureDescriptor) -> D::TextureHandle;

    fn create_sampler(&self, descriptor: &SamplerDescriptor) -> D::SamplerHandle;

    fn create_bind_group_layout<I>(
        &self,
        descriptor: BindGroupLayoutDescriptor<I>,
    ) -> D::BindGroupLayoutHandle
    where
        I: IntoIterator<Item = BindGroupLayoutEntry>;

    fn create_pipeline_layout<I>(
        &self,
        descriptor: PipelineLayoutDescriptor<I>,
    ) -> D::PipelineLayoutHandle
    where
        I: IntoIterator,
        I::Item: Borrow<D::BindGroupLayoutHandle>;

    fn create_bind_group<'a, E>(&self, descriptor: BindGroupDescriptor<D, E>) -> D::BindGroupHandle
    where
        E: IntoIterator<Item = BindGroupEntry<'a, D>>;

    fn create_query_set(&self, descriptor: &QuerySetDescriptor) -> D::QuerySetHandle;

    fn create_shader_module(&self, source: &str) -> D::ShaderModuleHandle;

    fn create_compute_pipeline(
        &self,
        descriptor: &ComputePipelineDescriptor<D>,
    ) -> D::ComputePipelineHandle;

    fn create_compute_pipeline_async(
        &self,
        descriptor: &ComputePipelineDescriptor<D>,
    ) -> Self::CreateComputePipelineAsync;

    fn create_render_pipeline(
        &self,
        descriptor: &RenderPipelineDescriptor<D>,
    ) -> D::RenderPipelineHandle;

    fn create_render_pipeline_async(
        &self,
        descriptor: &RenderPipelineDescriptor<D>,
    ) -> Self::CreateRenderPipelineAsync;

    fn create_command_encoder(&self) -> D::CommandEncoderHandle;

    fn create_render_bundle_encoder(
        &self,
        descriptor: &RenderBundleEncoderDescriptor,
    ) -> D::RenderBundleEncoderHandle;

    fn queue_handle(&self) -> D::QueueHandle;
}

flags! {
    pub enum BufferUsage: u32 {
        None         = 0x0000,
        MapRead      = 0x0001,
        MapWrite     = 0x0002,
        CopySrc      = 0x0004,
        CopyDst      = 0x0008,
        Index        = 0x0010,
        Vertex       = 0x0020,
        Uniform      = 0x0040,
        Storage      = 0x0080,
        Indirect     = 0x0100,
        QueryResolve = 0x0200,
    }

    pub enum TextureUsage: u32 {
        None             = 0x0000,
        RenderAttachment = 0x0001,
        StorageBinding   = 0x0002,
        TextureBinding   = 0x0004,
        CopyDst          = 0x0008,
        CopySrc          = 0x0010,
    }

    pub enum ShaderStage: u32 {
        None    = 0x0000,
        Vertex  = 0x0001,
        Fragment = 0x0002,
        Compute = 0x0004,
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BufferDescriptor {
    pub size: usize,
    pub usage_flags: FlagSet<BufferUsage>,
    pub mapped_at_creation: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TextureDimensions {
    One,
    Two,
    Three,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TextureViewDimension {
    One,
    Two,
    Three,
    TwoArray,
    Cube,
    CubeArray,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct TextureDescriptor<'a> {
    pub size: (u32, u32, u32),
    pub mipmap_levels: u32,
    pub sample_count: u32,
    pub dimensions: TextureDimensions,
    pub format: TextureFormatId,
    pub usage_flags: FlagSet<TextureUsage>,
    pub view_formats: &'a [TextureFormatId],
}

#[derive(Clone, PartialEq, Debug)]
pub struct SamplerDescriptor {
    pub address_mode_u: AddressMode,
    pub address_mode_v: AddressMode,
    pub address_mode_w: AddressMode,
    pub magnification_filter: FilterMode,
    pub minification_filter: FilterMode,
    pub mipmap_filter: FilterMode,
    pub lod_clamp: RangeInclusive<f32>,
    pub max_anisotropy: u16,
    pub compare: Option<CompareFunction>,
}

impl Default for SamplerDescriptor {
    fn default() -> Self {
        SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            magnification_filter: FilterMode::Nearest,
            minification_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            lod_clamp: 0.0..=32.0,
            max_anisotropy: 1,
            compare: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum QueryType {
    Occlusion,
    Timestamp,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct QuerySetDescriptor {
    pub query_type: QueryType,
    pub len: usize,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MapMode {
    Read,
    Write,
}

pub trait Buffer<D>: Clone + Sized
where
    D: Driver,
{
    type Map: Future<Output = Result<(), MapError>>;

    type Mapped<'a>: AsRef<[u8]>
    where
        Self: 'a;

    type MappedMut<'a>: AsMut<[u8]>
    where
        Self: 'a;

    fn map(&self, mode: MapMode, range: Range<usize>) -> Self::Map;

    fn mapped<'a>(&'a self, range: Range<usize>) -> Self::Mapped<'a>;

    fn mapped_mut<'a>(&'a self, range: Range<usize>) -> Self::MappedMut<'a>;

    fn unmap(&self);

    fn binding<'a>(&'a self, offset: usize, size: usize) -> D::BufferBinding<'a>;
}

pub struct TextureViewDescriptor {
    pub format: TextureFormatId,
    pub dimensions: TextureViewDimension,
    pub aspect: TextureAspect,
    pub mip_levels: Range<u32>,
    pub layers: Range<u32>,
}

pub trait Texture<D>: Clone + Sized
where
    D: Driver,
{
    fn texture_view<'a>(&'a self, descriptor: &TextureViewDescriptor) -> D::TextureView<'a>;
}

pub struct CopyBufferToBuffer<'a, D>
where
    D: Driver,
{
    pub source: &'a D::BufferHandle,
    pub source_offset: usize,
    pub destination: &'a D::BufferHandle,
    pub destination_offset: usize,
    pub size: usize,
}

pub struct CopyBufferToTexture<'a, D>
where
    D: Driver,
{
    pub source: ImageCopyBuffer<'a, D>,
    pub destination: ImageCopyTexture<'a, D>,
    pub copy_size: (u32, u32, u32),
}

pub struct CopyTextureToBuffer<'a, D>
where
    D: Driver,
{
    pub source: ImageCopyTexture<'a, D>,
    pub destination: ImageCopyBuffer<'a, D>,
    pub copy_size: (u32, u32, u32),
}

pub struct CopyTextureToTexture<'a, D>
where
    D: Driver,
{
    pub source: ImageCopyTexture<'a, D>,
    pub destination: ImageCopyTexture<'a, D>,
    pub copy_size: (u32, u32, u32),
}

pub struct ClearBuffer<'a, D>
where
    D: Driver,
{
    pub buffer: &'a D::BufferHandle,
    pub range: Range<usize>,
}

pub struct ResolveQuerySet<'a, D>
where
    D: Driver,
{
    pub query_set: &'a D::QuerySetHandle,
    pub query_range: Range<usize>,
    pub destination: &'a D::BufferHandle,
    pub destination_offset: usize,
}

pub trait CommandEncoder<D>
where
    D: Driver,
{
    fn copy_buffer_to_buffer(&mut self, op: CopyBufferToBuffer<D>);

    fn copy_buffer_to_texture(&mut self, op: CopyBufferToTexture<D>);

    fn copy_texture_to_buffer(&mut self, op: CopyTextureToBuffer<D>);

    fn copy_texture_to_texture(&mut self, op: CopyTextureToTexture<D>);

    fn clear_buffer(&mut self, op: ClearBuffer<D>);

    fn begin_compute_pass(&mut self) -> D::ComputePassEncoderHandle;

    fn begin_render_pass<'a, I>(
        &mut self,
        descriptor: RenderPassDescriptor<'a, D, I>,
    ) -> D::RenderPassEncoderHandle
    where
        I: IntoIterator<Item = Option<RenderPassColorAttachment<'a, D>>>,
        D: 'a;

    fn write_timestamp(&mut self, query_set: &D::QuerySetHandle, index: usize);

    fn resolve_query_set(&mut self, op: ResolveQuerySet<D>);

    fn finish(self) -> D::CommandBufferHandle;
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TextureAspect {
    All,
    StencilOnly,
    DepthOnly,
}

pub struct ImageCopyBuffer<'a, D>
where
    D: Driver,
{
    pub buffer_handle: &'a D::BufferHandle,
    pub offset: usize,
    pub size: usize,
    pub bytes_per_block: u32,
    pub blocks_per_row: u32,
    pub rows_per_image: u32,
}

impl<D> Clone for ImageCopyBuffer<'_, D>
where
    D: Driver,
{
    fn clone(&self) -> Self {
        ImageCopyBuffer {
            buffer_handle: self.buffer_handle,
            offset: self.offset,
            size: self.size,
            bytes_per_block: self.bytes_per_block,
            blocks_per_row: self.blocks_per_row,
            rows_per_image: self.rows_per_image,
        }
    }
}

impl<D> Copy for ImageCopyBuffer<'_, D> where D: Driver {}

#[derive(Clone, Copy)]
pub struct ImageCopyTexture<'a, D>
where
    D: Driver,
{
    pub texture_handle: &'a D::TextureHandle,
    pub mip_level: u32,
    pub origin: (u32, u32, u32),
    pub aspect: TextureAspect,
}

#[derive(Clone, Copy)]
pub struct WriteBufferOperation<'a, D>
where
    D: Driver,
{
    pub buffer_handle: &'a D::BufferHandle,
    pub offset: usize,
    pub data: &'a [u8],
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ImageDataLayout {
    pub offset: usize,
    pub bytes_per_row: u32,
    pub rows_per_image: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ImageCopySize2D {
    pub width: u32,
    pub height: u32,
}

impl Default for ImageCopySize2D {
    fn default() -> Self {
        ImageCopySize2D {
            width: 1,
            height: 1,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ImageCopySize3D {
    pub width: u32,
    pub height: u32,
    pub depth_or_layers: u32,
}

impl Default for ImageCopySize3D {
    fn default() -> Self {
        ImageCopySize3D {
            width: 1,
            height: 1,
            depth_or_layers: 1,
        }
    }
}

pub struct WriteTextureOperation<'a, D>
where
    D: Driver,
{
    pub image_copy_texture: ImageCopyTexture<'a, D>,
    pub image_data_layout: ImageDataLayout,
    pub extent: (u32, u32, u32),
    pub data: &'a [u8],
}

pub trait Queue<D>: Sized
where
    D: Driver,
{
    fn submit(&self, command_buffer: &D::CommandBufferHandle);

    fn write_buffer(&self, operation: WriteBufferOperation<D>);

    fn write_texture(&self, operation: WriteTextureOperation<D>);
}

pub enum BindingResource<'a, D>
where
    D: Driver,
{
    BufferBinding(D::BufferBinding<'a>),
    TextureView(D::TextureView<'a>),
    Sampler(&'a D::SamplerHandle),
}

impl<'a, D> Clone for BindingResource<'a, D>
where
    D: Driver,
{
    fn clone(&self) -> Self {
        match self {
            BindingResource::BufferBinding(r) => BindingResource::BufferBinding(r.clone()),
            BindingResource::TextureView(r) => BindingResource::TextureView(r.clone()),
            BindingResource::Sampler(r) => BindingResource::Sampler(*r),
        }
    }
}

pub struct BindGroupEntry<'a, D>
where
    D: Driver,
{
    pub binding: u32,
    pub resource: BindingResource<'a, D>,
}

pub struct BindGroupDescriptor<'a, D, E>
where
    D: Driver,
{
    pub layout: &'a D::BindGroupLayoutHandle,
    pub entries: E,
}

pub struct ComputePipelineDescriptor<'a, D>
where
    D: Driver,
{
    pub layout: &'a D::PipelineLayoutHandle,
    pub shader_module: &'a D::ShaderModuleHandle,
    pub entry_point: &'a str,
    pub constants: &'a HashMap<String, f64>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BufferBindingType {
    Uniform,
    Storage,
    ReadonlyStorage,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SamplerBindingType {
    Filtering,
    NonFiltering,
    Comparison,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TextureSampleType {
    Float,
    UnfilterableFloat,
    SignedInteger,
    UnsignedInteger,
    Depth,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(unused)]
pub enum StorageTextureAccess {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BindingType {
    Buffer(BufferBindingType),
    Sampler(SamplerBindingType),
    Texture {
        sample_type: TextureSampleType,
        dimension: TextureViewDimension,
        multisampled: bool,
    },
    StorageTexture {
        access: StorageTextureAccess,
        format: TextureFormatId,
        dimension: TextureViewDimension,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BindGroupLayoutEntry {
    pub binding: u32,
    pub binding_type: BindingType,
    pub visibility: FlagSet<ShaderStage>,
}

pub struct BindGroupLayoutDescriptor<I> {
    pub entries: I,
}

pub struct PipelineLayoutDescriptor<I> {
    pub bind_group_layouts: I,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StencilOperation {
    Keep,
    Zero,
    Replace,
    Invert,
    IncrementClamp,
    DecrementClamp,
    IncrementWrap,
    DecrementWrap,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct StencilFaceState {
    pub compare: CompareFunction,
    pub depth_fail_op: StencilOperation,
    pub fail_op: StencilOperation,
    pub pass_op: StencilOperation,
}

impl Default for StencilFaceState {
    fn default() -> Self {
        StencilFaceState {
            compare: CompareFunction::Always,
            depth_fail_op: StencilOperation::Keep,
            fail_op: StencilOperation::Keep,
            pass_op: StencilOperation::Keep,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct DepthStencilState {
    pub format: TextureFormatId,
    pub depth_write_enabled: bool,
    pub depth_compare: CompareFunction,
    pub stencil_front: StencilFaceState,
    pub stencil_back: StencilFaceState,
    pub stencil_read_mask: u32,
    pub stencil_write_mask: u32,
    pub depth_bias: i32,
    pub depth_bias_slope_scale: f32,
    pub depth_bias_clamp: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MultisampleState {
    pub count: u32,
    pub mask: u32,
    pub alpha_to_coverage_enabled: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PrimitiveTopology {
    PointList,
    LineList,
    LineStrip,
    TriangleList,
    TriangleStrip,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PrimitiveState {
    pub topology: PrimitiveTopology,
    pub strip_index_format: Option<IndexFormat>,
    pub front_face: FrontFace,
    pub cull_mode: Option<CullMode>,
}

pub struct VertexState<'a, D>
where
    D: Driver,
{
    pub shader_module: &'a D::ShaderModuleHandle,
    pub entry_point: &'a str,
    pub constants: &'a HashMap<String, f64>,
    pub vertex_buffer_layouts: &'a [VertexBufferLayout<'a>],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ColorTargetState {
    pub format: TextureFormatId,
    pub blend: Option<BlendState>,
    pub write_mask: ColorWrite,
}

pub struct FragmentState<'a, D>
where
    D: Driver,
{
    pub shader_module: &'a D::ShaderModuleHandle,
    pub entry_point: &'a str,
    pub constants: &'a HashMap<String, f64>,
    pub targets: &'a [ColorTargetState],
}

pub struct RenderPipelineDescriptor<'a, D>
where
    D: Driver,
{
    pub layout: &'a D::PipelineLayoutHandle,
    pub primitive_state: &'a PrimitiveState,
    pub vertex_state: VertexState<'a, D>,
    pub depth_stencil_state: Option<&'a DepthStencilState>,
    pub fragment_state: Option<FragmentState<'a, D>>,
    pub multisample_state: Option<&'a MultisampleState>,
}

pub trait ProgrammablePassEncoder<D>: Clone + Sized
where
    D: Driver,
{
    fn set_bind_group(&mut self, index: u32, handle: &D::BindGroupHandle);
}

pub trait ComputePassEncoder<D>: ProgrammablePassEncoder<D>
where
    D: Driver,
{
    fn set_pipeline(&mut self, handle: &D::ComputePipelineHandle);

    fn dispatch_workgroups(&mut self, x: u32, y: u32, z: u32);

    fn dispatch_workgroups_indirect(&mut self, buffer_handle: &D::BufferHandle, offset: usize);

    fn end(self);
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SetIndexBuffer<'a, D>
where
    D: Driver,
{
    pub buffer_handle: &'a D::BufferHandle,
    pub index_format: IndexFormat,
    pub range: Option<Range<usize>>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SetVertexBuffer<'a, D>
where
    D: Driver,
{
    pub slot: u32,
    pub buffer_handle: Option<&'a D::BufferHandle>,
    pub range: Option<Range<usize>>,
}

pub trait RenderEncoder<D>: ProgrammablePassEncoder<D>
where
    D: Driver,
{
    fn set_pipeline(&mut self, handle: &D::RenderPipelineHandle);

    fn set_index_buffer(&mut self, op: SetIndexBuffer<D>);

    fn set_vertex_buffer(&mut self, op: SetVertexBuffer<D>);

    fn draw(&mut self, op: Draw);

    fn draw_indexed(&mut self, op: DrawIndexed);

    fn draw_indirect(&mut self, buffer_handle: &D::BufferHandle, offset: usize);

    fn draw_indexed_indirect(&mut self, buffer_handle: &D::BufferHandle, offset: usize);
}

pub trait RenderPassEncoder<D>: RenderEncoder<D>
where
    D: Driver,
{
    fn set_viewport(&mut self, viewport: &Viewport);

    fn set_scissor_rect(&mut self, scissor_rect: &ScissorRect);

    fn set_blend_constant(&mut self, blend_constant: &BlendConstant);

    fn set_stencil_reference(&mut self, stencil_reference: u32);

    fn begin_occlusion_query(&mut self, query_index: u32);

    fn end_occlusion_query(&mut self);

    fn execute_bundles<'a>(&'a mut self) -> D::ExecuteRenderBundlesEncoder<'a>;

    fn end(self);
}

pub trait ExecuteRenderBundlesEncoder<D>
where
    D: Driver,
{
    fn push_bundle(&mut self, bundle: &D::RenderBundleHandle);

    fn finish(self);
}

pub trait RenderBundleEncoder<D>: RenderEncoder<D>
where
    D: Driver,
{
    fn finish(self) -> D::RenderBundleHandle;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RenderBundleEncoderDescriptor<'a> {
    pub color_formats: &'a [TextureFormatId],
    pub depth_stencil_format: Option<TextureFormatId>,
    pub sample_count: u32,
    pub depth_read_only: bool,
    pub stencil_read_only: bool,
}

#[derive(Clone)]
pub struct RenderPassColorAttachment<'a, D>
where
    D: Driver,
{
    pub view: D::TextureView<'a>,
    pub resolve_target: Option<D::TextureView<'a>>,
    pub load_op: LoadOp<[f64; 4]>,
    pub store_op: StoreOp,
}

pub struct DepthStencilOperations<T> {
    pub load_op: LoadOp<T>,
    pub store_op: StoreOp,
}

pub struct RenderPassDepthStencilAttachment<'a, D>
where
    D: Driver,
{
    pub view: D::TextureView<'a>,
    pub depth_operations: Option<DepthStencilOperations<f32>>,
    pub stencil_operations: Option<DepthStencilOperations<u32>>,
}

pub struct RenderPassDescriptor<'a, D, I>
where
    D: Driver,
{
    pub color_attachments: I,
    pub depth_stencil_attachment: Option<RenderPassDepthStencilAttachment<'a, D>>,
    pub occlusion_query_set: Option<&'a D::QuerySetHandle>,
}
