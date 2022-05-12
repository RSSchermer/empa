use crate::device::Device;
use crate::CompareFunction;
use std::ops::RangeInclusive;
use web_sys::{
    GpuAddressMode, GpuCompareFunction, GpuFilterMode, GpuSampler, GpuSamplerDescriptor,
};

pub struct Sampler {
    pub(crate) inner: GpuSampler,
}

impl Sampler {
    pub(crate) fn new(device: &Device, descriptor: &SamplerDescriptor) -> Self {
        let inner = device
            .inner
            .create_sampler_with_descriptor(&descriptor.to_web_sys());

        Sampler { inner }
    }

    pub(crate) fn anisotropic(device: &Device, descriptor: &AnisotropicSamplerDescriptor) -> Self {
        if descriptor.max_anisotropy <= 1 {
            panic!("`max_anisotropy` must be set to a value greater than `1`")
        }

        let inner = device
            .inner
            .create_sampler_with_descriptor(&descriptor.to_web_sys());

        Sampler { inner }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct SamplerDescriptor {
    pub address_mode_u: AddressMode,
    pub address_mode_v: AddressMode,
    pub address_mode_w: AddressMode,
    pub magnification_filter: FilterMode,
    pub minification_filter: FilterMode,
    pub mipmap_filter: MipmapFilterMode,
    pub lod_clamp: RangeInclusive<f32>,
}

impl SamplerDescriptor {
    fn to_web_sys(&self) -> GpuSamplerDescriptor {
        let SamplerDescriptor {
            address_mode_u,
            address_mode_v,
            address_mode_w,
            magnification_filter,
            minification_filter,
            mipmap_filter,
            lod_clamp,
        } = self;

        let mut desc = GpuSamplerDescriptor::new();

        desc.address_mode_u(address_mode_u.to_web_sys());
        desc.address_mode_v(address_mode_v.to_web_sys());
        desc.address_mode_w(address_mode_w.to_web_sys());
        desc.mag_filter(magnification_filter.to_web_sys());
        desc.min_filter(minification_filter.to_web_sys());
        desc.mipmap_filter(mipmap_filter.to_web_sys());
        desc.lod_min_clamp(*lod_clamp.start());
        desc.lod_max_clamp(*lod_clamp.end());

        desc
    }
}

impl Default for SamplerDescriptor {
    fn default() -> Self {
        SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            magnification_filter: FilterMode::Nearest,
            minification_filter: FilterMode::Nearest,
            mipmap_filter: MipmapFilterMode::Nearest,
            lod_clamp: 0.0..=32.0,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct AnisotropicSamplerDescriptor {
    pub max_anisotropy: u16,
    pub address_mode_u: AddressMode,
    pub address_mode_v: AddressMode,
    pub address_mode_w: AddressMode,
    pub lod_clamp: RangeInclusive<f32>,
}

impl AnisotropicSamplerDescriptor {
    fn to_web_sys(&self) -> GpuSamplerDescriptor {
        let AnisotropicSamplerDescriptor {
            max_anisotropy,
            address_mode_u,
            address_mode_v,
            address_mode_w,
            lod_clamp,
        } = self;

        let mut desc = GpuSamplerDescriptor::new();

        desc.address_mode_u(address_mode_u.to_web_sys());
        desc.address_mode_v(address_mode_v.to_web_sys());
        desc.address_mode_w(address_mode_w.to_web_sys());
        desc.mag_filter(GpuFilterMode::Linear);
        desc.min_filter(GpuFilterMode::Linear);
        desc.mipmap_filter(GpuFilterMode::Linear);
        desc.lod_min_clamp(*lod_clamp.start());
        desc.lod_max_clamp(*lod_clamp.end());
        desc.max_anisotropy(*max_anisotropy);

        desc
    }
}

impl Default for AnisotropicSamplerDescriptor {
    fn default() -> Self {
        AnisotropicSamplerDescriptor {
            max_anisotropy: 1,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            lod_clamp: 0.0..=32.0,
        }
    }
}

pub struct ComparisonSampler {
    pub(crate) inner: GpuSampler,
}

impl ComparisonSampler {
    pub(crate) fn new(device: &Device, descriptor: &ComparisonSamplerDescriptor) -> Self {
        let inner = device
            .inner
            .create_sampler_with_descriptor(&descriptor.to_web_sys());

        ComparisonSampler { inner }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ComparisonSamplerDescriptor {
    pub compare: CompareFunction,
    pub address_mode_u: AddressMode,
    pub address_mode_v: AddressMode,
    pub address_mode_w: AddressMode,
    pub magnification_filter: FilterMode,
    pub minification_filter: FilterMode,
    pub mipmap_filter: MipmapFilterMode,
    pub lod_clamp: RangeInclusive<f32>,
    pub max_anisotropy: u16,
}

impl ComparisonSamplerDescriptor {
    fn to_web_sys(&self) -> GpuSamplerDescriptor {
        let ComparisonSamplerDescriptor {
            compare,
            address_mode_u,
            address_mode_v,
            address_mode_w,
            magnification_filter,
            minification_filter,
            mipmap_filter,
            lod_clamp,
            max_anisotropy,
        } = self;

        let mut desc = GpuSamplerDescriptor::new();

        desc.compare(compare.to_web_sys());
        desc.address_mode_u(address_mode_u.to_web_sys());
        desc.address_mode_v(address_mode_v.to_web_sys());
        desc.address_mode_w(address_mode_w.to_web_sys());
        desc.mag_filter(magnification_filter.to_web_sys());
        desc.min_filter(minification_filter.to_web_sys());
        desc.mipmap_filter(mipmap_filter.to_web_sys());
        desc.lod_min_clamp(*lod_clamp.start());
        desc.lod_max_clamp(*lod_clamp.end());
        desc.max_anisotropy(*max_anisotropy);

        desc
    }
}

impl Default for ComparisonSamplerDescriptor {
    fn default() -> Self {
        ComparisonSamplerDescriptor {
            compare: CompareFunction::Always,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            magnification_filter: FilterMode::Nearest,
            minification_filter: FilterMode::Nearest,
            mipmap_filter: MipmapFilterMode::Nearest,
            lod_clamp: 0.0..=32.0,
            max_anisotropy: 1,
        }
    }
}

pub struct NonFilteringSampler {
    pub(crate) inner: GpuSampler,
}

impl NonFilteringSampler {
    pub(crate) fn new(device: &Device, descriptor: &NonFilteringSamplerDescriptor) -> Self {
        let inner = device
            .inner
            .create_sampler_with_descriptor(&descriptor.to_web_sys());

        NonFilteringSampler { inner }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct NonFilteringSamplerDescriptor {
    pub address_mode_u: AddressMode,
    pub address_mode_v: AddressMode,
    pub address_mode_w: AddressMode,
    pub lod_clamp: RangeInclusive<f32>,
    pub max_anisotropy: u16,
}

impl NonFilteringSamplerDescriptor {
    fn to_web_sys(&self) -> GpuSamplerDescriptor {
        let NonFilteringSamplerDescriptor {
            address_mode_u,
            address_mode_v,
            address_mode_w,
            lod_clamp,
            max_anisotropy,
        } = self;

        let mut desc = GpuSamplerDescriptor::new();

        desc.address_mode_u(address_mode_u.to_web_sys());
        desc.address_mode_v(address_mode_v.to_web_sys());
        desc.address_mode_w(address_mode_w.to_web_sys());
        desc.lod_min_clamp(*lod_clamp.start());
        desc.lod_max_clamp(*lod_clamp.end());
        desc.max_anisotropy(*max_anisotropy);

        desc
    }
}

impl Default for NonFilteringSamplerDescriptor {
    fn default() -> Self {
        NonFilteringSamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            lod_clamp: 0.0..=32.0,
            max_anisotropy: 1,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AddressMode {
    ClampToEdge,
    Repeat,
    MirrorRepeat,
}

impl AddressMode {
    fn to_web_sys(&self) -> GpuAddressMode {
        match self {
            AddressMode::ClampToEdge => GpuAddressMode::ClampToEdge,
            AddressMode::Repeat => GpuAddressMode::Repeat,
            AddressMode::MirrorRepeat => GpuAddressMode::MirrorRepeat,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FilterMode {
    Nearest,
    Linear,
}

impl FilterMode {
    fn to_web_sys(&self) -> GpuFilterMode {
        match self {
            FilterMode::Nearest => GpuFilterMode::Nearest,
            FilterMode::Linear => GpuFilterMode::Linear,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MipmapFilterMode {
    Nearest,
    Linear,
}

impl MipmapFilterMode {
    fn to_web_sys(&self) -> GpuFilterMode {
        match self {
            MipmapFilterMode::Nearest => GpuFilterMode::Nearest,
            MipmapFilterMode::Linear => GpuFilterMode::Linear,
        }
    }
}
