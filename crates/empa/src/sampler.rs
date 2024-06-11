use std::ops::RangeInclusive;

use crate::device::Device;
use crate::driver::{Device as _, Driver, Dvr};
use crate::{driver, CompareFunction};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AddressMode {
    ClampToEdge,
    Repeat,
    MirrorRepeat,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FilterMode {
    Nearest,
    Linear,
}

pub struct Sampler {
    pub(crate) handle: <Dvr as Driver>::SamplerHandle,
}

impl Sampler {
    pub(crate) fn new(device: &Device, descriptor: &SamplerDescriptor) -> Self {
        let handle = device.device_handle.create_sampler(&descriptor.to_driver());

        Sampler { handle }
    }

    pub(crate) fn anisotropic(device: &Device, descriptor: &AnisotropicSamplerDescriptor) -> Self {
        if descriptor.max_anisotropy <= 1 {
            panic!("`max_anisotropy` must be set to a value greater than `1`")
        }

        let handle = device.device_handle.create_sampler(&descriptor.to_driver());

        Sampler { handle }
    }
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
}

impl SamplerDescriptor {
    fn to_driver(&self) -> driver::SamplerDescriptor {
        let SamplerDescriptor {
            address_mode_u,
            address_mode_v,
            address_mode_w,
            magnification_filter,
            minification_filter,
            mipmap_filter,
            lod_clamp,
        } = self;

        driver::SamplerDescriptor {
            address_mode_u: *address_mode_u,
            address_mode_v: *address_mode_v,
            address_mode_w: *address_mode_w,
            magnification_filter: *magnification_filter,
            minification_filter: *minification_filter,
            mipmap_filter: *mipmap_filter,
            lod_clamp: lod_clamp.clone(),
            ..Default::default()
        }
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
            mipmap_filter: FilterMode::Nearest,
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
    fn to_driver(&self) -> driver::SamplerDescriptor {
        let AnisotropicSamplerDescriptor {
            max_anisotropy,
            address_mode_u,
            address_mode_v,
            address_mode_w,
            lod_clamp,
        } = self;

        driver::SamplerDescriptor {
            address_mode_u: *address_mode_u,
            address_mode_v: *address_mode_v,
            address_mode_w: *address_mode_w,
            magnification_filter: FilterMode::Linear,
            minification_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            lod_clamp: lod_clamp.clone(),
            max_anisotropy: *max_anisotropy,
            ..Default::default()
        }
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
    pub(crate) handle: <Dvr as Driver>::SamplerHandle,
}

impl ComparisonSampler {
    pub(crate) fn new(device: &Device, descriptor: &ComparisonSamplerDescriptor) -> Self {
        let handle = device.device_handle.create_sampler(&descriptor.to_driver());

        ComparisonSampler { handle }
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
    pub mipmap_filter: FilterMode,
    pub lod_clamp: RangeInclusive<f32>,
    pub max_anisotropy: u16,
}

impl ComparisonSamplerDescriptor {
    fn to_driver(&self) -> driver::SamplerDescriptor {
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

        driver::SamplerDescriptor {
            address_mode_u: *address_mode_u,
            address_mode_v: *address_mode_v,
            address_mode_w: *address_mode_w,
            magnification_filter: *magnification_filter,
            minification_filter: *minification_filter,
            mipmap_filter: *mipmap_filter,
            lod_clamp: lod_clamp.clone(),
            max_anisotropy: *max_anisotropy,
            compare: Some(*compare),
        }
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
            mipmap_filter: FilterMode::Nearest,
            lod_clamp: 0.0..=32.0,
            max_anisotropy: 1,
        }
    }
}

pub struct NonFilteringSampler {
    pub(crate) handle: <Dvr as Driver>::SamplerHandle,
}

impl NonFilteringSampler {
    pub(crate) fn new(device: &Device, descriptor: &NonFilteringSamplerDescriptor) -> Self {
        let handle = device.device_handle.create_sampler(&descriptor.to_driver());

        NonFilteringSampler { handle }
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
    fn to_driver(&self) -> driver::SamplerDescriptor {
        let NonFilteringSamplerDescriptor {
            address_mode_u,
            address_mode_v,
            address_mode_w,
            lod_clamp,
            max_anisotropy,
        } = self;

        driver::SamplerDescriptor {
            address_mode_u: *address_mode_u,
            address_mode_v: *address_mode_v,
            address_mode_w: *address_mode_w,
            lod_clamp: lod_clamp.clone(),
            max_anisotropy: *max_anisotropy,
            ..Default::default()
        }
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
