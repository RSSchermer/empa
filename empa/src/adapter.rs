use std::error::Error;
use std::fmt;
use std::future::Future;
use std::lazy::SyncOnceCell;
use std::pin::Pin;
use std::task::{Context, Poll};

use bitflags::bitflags;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{GpuAdapter, GpuDeviceDescriptor};

use crate::device::{Device, DeviceDescriptor};

bitflags! {
    #[repr(transparent)]
    pub struct Features: u64 {
        const NONE = 0;
        const DEPTH_CLIP_CONTROL = 1 << 0;
        const DEPTH24UNORM_STENCIL8 = 1 << 1;
        const DEPTH32FLOAT_STENCIL8 = 1 << 2;
        const TEXTURE_COMPRESSION_BC = 1 << 3;
        const TEXTURE_COMPRESSION_ETC2 = 1 << 4;
        const TEXTURE_COMPRESSION_ASTC = 1 << 5;
        const TIMESTAMP_QUERY = 1 << 6;
        const INDIRECT_FIRST_INSTANCE = 1 << 7;
        const SHADER_F16 = 1 << 8;
        const BGRA8UNORM_STORAGE = 1 << 9;
    }
}

impl Features {
    pub(crate) fn to_js_array(&self) -> js_sys::Array {
        let array = js_sys::Array::new();

        if self.intersects(Features::DEPTH_CLIP_CONTROL) {
            array.push(&JsValue::from("depth-clip-control"));
        }

        if self.intersects(Features::DEPTH24UNORM_STENCIL8) {
            array.push(&JsValue::from("depth24unorm-stencil8"));
        }

        if self.intersects(Features::DEPTH32FLOAT_STENCIL8) {
            array.push(&JsValue::from("depth32float-stencil8"));
        }

        if self.intersects(Features::TEXTURE_COMPRESSION_BC) {
            array.push(&JsValue::from("texture-compression-bc"));
        }

        if self.intersects(Features::TEXTURE_COMPRESSION_ETC2) {
            array.push(&JsValue::from("texture-compression-etc2"));
        }

        if self.intersects(Features::TEXTURE_COMPRESSION_ASTC) {
            array.push(&JsValue::from("texture-compression-astc"));
        }

        if self.intersects(Features::TIMESTAMP_QUERY) {
            array.push(&JsValue::from("timestamp-query"));
        }

        if self.intersects(Features::INDIRECT_FIRST_INSTANCE) {
            array.push(&JsValue::from("indirect-first-instance"));
        }

        if self.intersects(Features::SHADER_F16) {
            array.push(&JsValue::from("shader-f16"));
        }

        if self.intersects(Features::BGRA8UNORM_STORAGE) {
            array.push(&JsValue::from("bgra8unorm-storage"));
        }

        array
    }
}

impl Default for Features {
    fn default() -> Self {
        Features::NONE
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    pub max_texture_dimension_1d: u32,
    pub max_texture_dimension_2d: u32,
    pub max_texture_dimension_3d: u32,
    pub max_texture_array_layers: u32,
    pub max_bind_groups: u32,
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
    pub max_vertex_attributes: u32,
    pub max_vertex_buffer_array_stride: u32,
    pub max_inter_stage_shader_components: u32,
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
            max_vertex_attributes: 16,
            max_vertex_buffer_array_stride: 2048,
            max_inter_stage_shader_components: 60,
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
    inner: GpuAdapter,
    features_cache: SyncOnceCell<Features>,
    limits_cache: SyncOnceCell<Limits>,
}

impl Adapter {
    #[doc(hidden)]
    pub fn from_web_sys(inner: GpuAdapter) -> Self {
        Adapter {
            inner,
            features_cache: Default::default(),
            limits_cache: Default::default(),
        }
    }

    pub fn supported_features(&self) -> &Features {
        self.features_cache.get_or_init(|| {
            let raw = self.inner.features();
            let mut features = Features::NONE;

            if js_sys::Reflect::has(raw.as_ref(), &JsValue::from("depth-clip-control"))
                .unwrap_or(false)
            {
                features |= Features::DEPTH_CLIP_CONTROL;
            }

            if js_sys::Reflect::has(raw.as_ref(), &JsValue::from("depth24unorm-stencil8"))
                .unwrap_or(false)
            {
                features |= Features::DEPTH24UNORM_STENCIL8;
            }

            if js_sys::Reflect::has(raw.as_ref(), &JsValue::from("depth32float-stencil8"))
                .unwrap_or(false)
            {
                features |= Features::DEPTH32FLOAT_STENCIL8;
            }

            if js_sys::Reflect::has(raw.as_ref(), &JsValue::from("texture-compression-bc"))
                .unwrap_or(false)
            {
                features |= Features::TEXTURE_COMPRESSION_BC;
            }

            if js_sys::Reflect::has(raw.as_ref(), &JsValue::from("texture-compression-etc2"))
                .unwrap_or(false)
            {
                features |= Features::TEXTURE_COMPRESSION_ETC2;
            }

            if js_sys::Reflect::has(raw.as_ref(), &JsValue::from("texture-compression-astc"))
                .unwrap_or(false)
            {
                features |= Features::TEXTURE_COMPRESSION_ASTC;
            }

            if js_sys::Reflect::has(raw.as_ref(), &JsValue::from("timestamp-query"))
                .unwrap_or(false)
            {
                features |= Features::TIMESTAMP_QUERY;
            }

            if js_sys::Reflect::has(raw.as_ref(), &JsValue::from("indirect-first-instance"))
                .unwrap_or(false)
            {
                features |= Features::INDIRECT_FIRST_INSTANCE;
            }

            if js_sys::Reflect::has(raw.as_ref(), &JsValue::from("shader-f16")).unwrap_or(false) {
                features |= Features::SHADER_F16;
            }

            if js_sys::Reflect::has(raw.as_ref(), &JsValue::from("bgra8unorm-storage"))
                .unwrap_or(false)
            {
                features |= Features::BGRA8UNORM_STORAGE;
            }

            features
        })
    }

    pub fn supported_limits(&self) -> &Limits {
        self.limits_cache.get_or_init(|| {
            let raw = self.inner.limits();

            Limits {
                max_texture_dimension_1d: raw.max_texture_dimension_1d(),
                max_texture_dimension_2d: raw.max_texture_dimension_2d(),
                max_texture_dimension_3d: raw.max_texture_dimension_3d(),
                max_texture_array_layers: raw.max_texture_array_layers(),
                max_bind_groups: raw.max_bind_groups(),
                max_dynamic_uniform_buffers_per_pipeline_layout: raw
                    .max_dynamic_uniform_buffers_per_pipeline_layout(),
                max_dynamic_storage_buffers_per_pipeline_layout: raw
                    .max_dynamic_storage_buffers_per_pipeline_layout(),
                max_sampled_textures_per_shader_stage: raw.max_sampled_textures_per_shader_stage(),
                max_samplers_per_shader_stage: raw.max_samplers_per_shader_stage(),
                max_storage_buffers_per_shader_stage: raw.max_storage_buffers_per_shader_stage(),
                max_storage_textures_per_shader_stage: raw.max_storage_textures_per_shader_stage(),
                max_uniform_buffers_per_shader_stage: raw.max_uniform_buffers_per_shader_stage(),
                max_uniform_buffer_binding_size: raw.max_uniform_buffer_binding_size() as u64,
                max_storage_buffer_binding_size: raw.max_storage_buffer_binding_size() as u64,
                min_uniform_buffer_offset_alignment: raw.min_uniform_buffer_offset_alignment(),
                min_storage_buffer_offset_alignment: raw.min_storage_buffer_offset_alignment(),
                max_vertex_buffers: raw.max_vertex_buffers(),
                max_vertex_attributes: raw.max_vertex_attributes(),
                max_vertex_buffer_array_stride: raw.max_vertex_buffer_array_stride(),
                max_inter_stage_shader_components: raw.max_inter_stage_shader_components(),
                max_compute_workgroup_storage_size: raw.max_compute_workgroup_storage_size(),
                max_compute_invocations_per_workgroup: raw.max_compute_invocations_per_workgroup(),
                max_compute_workgroup_size_x: raw.max_compute_workgroup_size_x(),
                max_compute_workgroup_size_y: raw.max_compute_workgroup_size_y(),
                max_compute_workgroup_size_z: raw.max_compute_workgroup_size_z(),
                max_compute_workgroups_per_dimension: raw.max_compute_workgroups_per_dimension(),
            }
        })
    }

    pub fn request_device(&self, descriptor: &DeviceDescriptor) -> RequestDevice {
        let DeviceDescriptor {
            required_features,
            required_limits,
        } = descriptor;

        let mut desc = GpuDeviceDescriptor::new();

        if required_features != &Features::default() {
            desc.required_features(required_features.to_js_array().as_ref());
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
    type Output = Result<Device, RequestDeviceError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.get_mut().inner)
            .poll(cx)
            .map_ok(|device| {
                Device {
                    inner: device.unchecked_into(),
                }
            })
            .map_err(|err| RequestDeviceError {
                inner: err.unchecked_into(),
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
