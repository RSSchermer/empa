use std::error::Error;
use std::fmt;
use std::future::Future;
use std::sync::OnceLock;

use flagset::{flags, FlagSet};
use futures::TryFutureExt;

use crate::device::{Device, DeviceDescriptor};
use crate::driver::{Adapter as _, Driver, Dvr};

flags! {
    pub enum Feature: u64 {
        None = 0,
        DepthClipControl = 1 << 0,
        Depth24UNormStencil8 = 1 << 1,
        Depth32FloatStencil8 = 1 << 2,
        TextureCompressionBc = 1 << 3,
        TextureComporessionEtc2 = 1 << 4,
        TextureCompressionAstc = 1 << 5,
        TimestampQuery = 1 << 6,
        IndirectFirstInstance = 1 << 7,
        ShaderF16 = 1 << 8,
        Bgra8UNormStorage = 1 << 9,
    }
}

impl Default for Feature {
    fn default() -> Self {
        Feature::None
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    pub max_texture_dimension_1d: u32,
    pub max_texture_dimension_2d: u32,
    pub max_texture_dimension_3d: u32,
    pub max_texture_array_layers: u32,
    pub max_bind_groups: u32,
    pub max_bindings_per_bind_group: u32,
    pub max_dynamic_uniform_buffers_per_pipeline_layout: u32,
    pub max_dynamic_storage_buffers_per_pipeline_layout: u32,
    pub max_sampled_textures_per_shader_stage: u32,
    pub max_samplers_per_shader_stage: u32,
    pub max_storage_buffers_per_shader_stage: u32,
    pub max_storage_textures_per_shader_stage: u32,
    pub max_uniform_buffers_per_shader_stage: u32,
    pub max_uniform_buffer_binding_size: u64,
    pub max_storage_buffer_binding_size: u64,
    pub min_uniform_buffer_offset_alignment: u32,
    pub min_storage_buffer_offset_alignment: u32,
    pub max_vertex_buffers: u32,
    pub max_buffer_size: u64,
    pub max_vertex_attributes: u32,
    pub max_vertex_buffer_array_stride: u32,
    pub max_inter_stage_shader_components: u32,
    pub max_color_attachments: u32,
    pub max_color_attachment_bytes_per_sample: u32,
    pub max_compute_workgroup_storage_size: u32,
    pub max_compute_invocations_per_workgroup: u32,
    pub max_compute_workgroup_size_x: u32,
    pub max_compute_workgroup_size_y: u32,
    pub max_compute_workgroup_size_z: u32,
    pub max_compute_workgroups_per_dimension: u32,
}

impl Default for Limits {
    fn default() -> Self {
        Limits {
            max_texture_dimension_1d: 8192,
            max_texture_dimension_2d: 8192,
            max_texture_dimension_3d: 2048,
            max_texture_array_layers: 256,
            max_bind_groups: 4,
            max_bindings_per_bind_group: 1000,
            max_dynamic_uniform_buffers_per_pipeline_layout: 8,
            max_dynamic_storage_buffers_per_pipeline_layout: 4,
            max_sampled_textures_per_shader_stage: 16,
            max_samplers_per_shader_stage: 16,
            max_storage_buffers_per_shader_stage: 8,
            max_storage_textures_per_shader_stage: 4,
            max_uniform_buffers_per_shader_stage: 4,
            max_uniform_buffer_binding_size: 65536,
            max_storage_buffer_binding_size: 134217728,
            min_uniform_buffer_offset_alignment: 256,
            min_storage_buffer_offset_alignment: 256,
            max_vertex_buffers: 8,
            max_buffer_size: 268435456,
            max_vertex_attributes: 16,
            max_vertex_buffer_array_stride: 2048,
            max_inter_stage_shader_components: 60,
            max_color_attachments: 8,
            max_color_attachment_bytes_per_sample: 32,
            max_compute_workgroup_storage_size: 16384,
            max_compute_invocations_per_workgroup: 256,
            max_compute_workgroup_size_x: 256,
            max_compute_workgroup_size_y: 256,
            max_compute_workgroup_size_z: 64,
            max_compute_workgroups_per_dimension: 65535,
        }
    }
}

pub struct Adapter {
    handle: <Dvr as Driver>::AdapterHandle,
    features_cache: OnceLock<FlagSet<Feature>>,
    limits_cache: OnceLock<Limits>,
}

impl Adapter {
    pub(crate) fn from_handle(handle: <Dvr as Driver>::AdapterHandle) -> Self {
        Adapter {
            handle,
            features_cache: Default::default(),
            limits_cache: Default::default(),
        }
    }

    pub fn supported_features(&self) -> &FlagSet<Feature> {
        self.features_cache
            .get_or_init(|| self.handle.supported_features())
    }

    pub fn supported_limits(&self) -> &Limits {
        self.limits_cache
            .get_or_init(|| self.handle.supported_limits())
    }

    pub fn request_device<Flags>(
        &self,
        descriptor: &DeviceDescriptor<Flags>,
    ) -> impl Future<Output = Result<Device, RequestDeviceError>>
    where
        Flags: Into<FlagSet<Feature>> + Copy,
    {
        self.handle
            .request_device(descriptor)
            .map_ok(|handle| Device { handle })
            .map_err(|inner| RequestDeviceError { inner })
    }
}

pub struct RequestDeviceError {
    inner: Box<dyn Error>,
}

impl fmt::Display for RequestDeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl fmt::Debug for RequestDeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}

impl Error for RequestDeviceError {}
