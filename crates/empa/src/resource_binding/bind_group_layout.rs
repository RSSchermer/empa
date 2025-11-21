use std::marker;

use empa_smi::{ResourceType, StorageTextureFormat, TexelType};
use flagset::FlagSet;

use crate::device::Device;
use crate::driver::{
    BufferBindingType, Device as _, Driver, Dvr, SamplerBindingType, ShaderStage,
    StorageTextureAccess, TextureSampleType, TextureViewDimension,
};
use crate::resource_binding::typed_bind_group_entry::TypedSlotBinding;
use crate::texture::format::TextureFormatId;
use crate::{Untyped, driver};

pub struct BindGroupLayoutEncoding<'a> {
    pub(crate) handle: &'a <Dvr as Driver>::BindGroupLayoutHandle,
}

pub struct BindGroupLayout<T = Untyped> {
    pub(crate) handle: <Dvr as Driver>::BindGroupLayoutHandle,
    _marker: marker::PhantomData<*const T>,
}

impl<T> BindGroupLayout<T> {
    pub(crate) fn new(device: &Device, layout: &[Option<BindGroupLayoutEntry>]) -> Self {
        let entries = layout
            .iter()
            .enumerate()
            .filter(|(_, e)| e.is_some())
            .map(|(i, e)| {
                let e = e.as_ref().unwrap();

                driver::BindGroupLayoutEntry {
                    binding: i as u32,
                    binding_type: resource_type_to_driver(&e.resource_type),
                    visibility: e.visibility,
                }
            });

        let handle = device
            .device_handle
            .create_bind_group_layout(driver::BindGroupLayoutDescriptor { entries });

        BindGroupLayout {
            handle,
            _marker: marker::PhantomData,
        }
    }

    pub fn to_encoding(&self) -> BindGroupLayoutEncoding<'_> {
        BindGroupLayoutEncoding {
            handle: &self.handle,
        }
    }
}

impl BindGroupLayout {
    pub(crate) fn untyped(device: &Device, layout: &[Option<BindGroupLayoutEntry>]) -> Self {
        BindGroupLayout::new(device, layout)
    }
}

impl<T> BindGroupLayout<T>
where
    T: TypedBindGroupLayout,
{
    pub(crate) fn typed(device: &Device) -> Self {
        BindGroupLayout::new(device, T::BIND_GROUP_LAYOUT)
    }
}

pub unsafe trait TypedBindGroupLayout {
    const BIND_GROUP_LAYOUT: &'static [Option<BindGroupLayoutEntry>];
}

macro_rules! impl_typed_bind_group_layout {
    ($($binding:ident),*) => {
        #[allow(unused_parens)]
        unsafe impl<$($binding),*> TypedBindGroupLayout for ($($binding,)*)
        where
            $($binding: TypedSlotBinding),*
        {
            const BIND_GROUP_LAYOUT: &'static [Option<BindGroupLayoutEntry>] = &[
                $($binding::ENTRY),*
            ];
        }
    }
}

impl_typed_bind_group_layout!(B);
impl_typed_bind_group_layout!(B0, B1);
impl_typed_bind_group_layout!(B0, B1, B2);
impl_typed_bind_group_layout!(B0, B1, B2, B3);
impl_typed_bind_group_layout!(B0, B1, B2, B3, B4);
impl_typed_bind_group_layout!(B0, B1, B2, B3, B4, B5);
impl_typed_bind_group_layout!(B0, B1, B2, B3, B4, B5, B6);
impl_typed_bind_group_layout!(B0, B1, B2, B3, B4, B5, B6, B7);
impl_typed_bind_group_layout!(B0, B1, B2, B3, B4, B5, B6, B7, B8);
impl_typed_bind_group_layout!(B0, B1, B2, B3, B4, B5, B6, B7, B8, B9);
impl_typed_bind_group_layout!(B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10);
impl_typed_bind_group_layout!(B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11);
impl_typed_bind_group_layout!(B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11, B12);
impl_typed_bind_group_layout!(B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11, B12, B13);
impl_typed_bind_group_layout!(
    B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11, B12, B13, B14
);
impl_typed_bind_group_layout!(
    B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11, B12, B13, B14, B15
);
impl_typed_bind_group_layout!(
    B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11, B12, B13, B14, B15, B16
);
impl_typed_bind_group_layout!(
    B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11, B12, B13, B14, B15, B16, B17
);
impl_typed_bind_group_layout!(
    B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11, B12, B13, B14, B15, B16, B17, B18
);
impl_typed_bind_group_layout!(
    B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11, B12, B13, B14, B15, B16, B17, B18, B19
);
impl_typed_bind_group_layout!(
    B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11, B12, B13, B14, B15, B16, B17, B18, B19, B20
);
impl_typed_bind_group_layout!(
    B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11, B12, B13, B14, B15, B16, B17, B18, B19, B20,
    B21
);
impl_typed_bind_group_layout!(
    B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11, B12, B13, B14, B15, B16, B17, B18, B19, B20,
    B21, B22
);
impl_typed_bind_group_layout!(
    B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11, B12, B13, B14, B15, B16, B17, B18, B19, B20,
    B21, B22, B23
);
impl_typed_bind_group_layout!(
    B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11, B12, B13, B14, B15, B16, B17, B18, B19, B20,
    B21, B22, B23, B24
);
impl_typed_bind_group_layout!(
    B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11, B12, B13, B14, B15, B16, B17, B18, B19, B20,
    B21, B22, B23, B24, B25
);
impl_typed_bind_group_layout!(
    B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11, B12, B13, B14, B15, B16, B17, B18, B19, B20,
    B21, B22, B23, B24, B25, B26
);
impl_typed_bind_group_layout!(
    B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11, B12, B13, B14, B15, B16, B17, B18, B19, B20,
    B21, B22, B23, B24, B25, B26, B27
);
impl_typed_bind_group_layout!(
    B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11, B12, B13, B14, B15, B16, B17, B18, B19, B20,
    B21, B22, B23, B24, B25, B26, B27, B28
);
impl_typed_bind_group_layout!(
    B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11, B12, B13, B14, B15, B16, B17, B18, B19, B20,
    B21, B22, B23, B24, B25, B26, B27, B28, B29
);
impl_typed_bind_group_layout!(
    B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11, B12, B13, B14, B15, B16, B17, B18, B19, B20,
    B21, B22, B23, B24, B25, B26, B27, B28, B29, B30
);
impl_typed_bind_group_layout!(
    B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11, B12, B13, B14, B15, B16, B17, B18, B19, B20,
    B21, B22, B23, B24, B25, B26, B27, B28, B29, B30, B31
);

pub struct BindGroupLayoutEntry {
    pub visibility: FlagSet<ShaderStage>,
    pub resource_type: ResourceType,
}

fn resource_type_to_driver(resource_type: &ResourceType) -> driver::BindingType {
    match resource_type {
        ResourceType::Texture1D(texel_type) => driver::BindingType::Texture {
            sample_type: texel_type_to_driver(texel_type),
            dimension: TextureViewDimension::One,
            multisampled: false,
        },
        ResourceType::Texture2D(texel_type) => driver::BindingType::Texture {
            sample_type: texel_type_to_driver(texel_type),
            dimension: TextureViewDimension::Two,
            multisampled: false,
        },
        ResourceType::Texture3D(texel_type) => driver::BindingType::Texture {
            sample_type: texel_type_to_driver(texel_type),
            dimension: TextureViewDimension::Three,
            multisampled: false,
        },
        ResourceType::Texture2DArray(texel_type) => driver::BindingType::Texture {
            sample_type: texel_type_to_driver(texel_type),
            dimension: TextureViewDimension::TwoArray,
            multisampled: false,
        },
        ResourceType::TextureCube(texel_type) => driver::BindingType::Texture {
            sample_type: texel_type_to_driver(texel_type),
            dimension: TextureViewDimension::Cube,
            multisampled: false,
        },
        ResourceType::TextureCubeArray(texel_type) => driver::BindingType::Texture {
            sample_type: texel_type_to_driver(texel_type),
            dimension: TextureViewDimension::CubeArray,
            multisampled: false,
        },
        ResourceType::TextureMultisampled2D(texel_type) => driver::BindingType::Texture {
            sample_type: texel_type_to_driver(texel_type),
            dimension: TextureViewDimension::Two,
            multisampled: true,
        },
        ResourceType::TextureDepth2D => driver::BindingType::Texture {
            sample_type: TextureSampleType::Depth,
            dimension: TextureViewDimension::Two,
            multisampled: false,
        },
        ResourceType::TextureDepth2DArray => driver::BindingType::Texture {
            sample_type: TextureSampleType::Depth,
            dimension: TextureViewDimension::TwoArray,
            multisampled: false,
        },
        ResourceType::TextureDepthCube => driver::BindingType::Texture {
            sample_type: TextureSampleType::Depth,
            dimension: TextureViewDimension::Cube,
            multisampled: false,
        },
        ResourceType::TextureDepthCubeArray => driver::BindingType::Texture {
            sample_type: TextureSampleType::Depth,
            dimension: TextureViewDimension::CubeArray,
            multisampled: false,
        },
        ResourceType::TextureDepthMultisampled2D => driver::BindingType::Texture {
            sample_type: TextureSampleType::Depth,
            dimension: TextureViewDimension::Two,
            multisampled: true,
        },
        ResourceType::StorageTexture1D(format) => driver::BindingType::StorageTexture {
            access: StorageTextureAccess::WriteOnly,
            dimension: TextureViewDimension::One,
            format: storage_texture_format_to_driver(format),
        },
        ResourceType::StorageTexture2D(format) => driver::BindingType::StorageTexture {
            access: StorageTextureAccess::WriteOnly,
            dimension: TextureViewDimension::Two,
            format: storage_texture_format_to_driver(format),
        },
        ResourceType::StorageTexture2DArray(format) => driver::BindingType::StorageTexture {
            access: StorageTextureAccess::WriteOnly,
            dimension: TextureViewDimension::TwoArray,
            format: storage_texture_format_to_driver(format),
        },
        ResourceType::StorageTexture3D(format) => driver::BindingType::StorageTexture {
            access: StorageTextureAccess::WriteOnly,
            dimension: TextureViewDimension::Three,
            format: storage_texture_format_to_driver(format),
        },
        ResourceType::FilteringSampler => {
            driver::BindingType::Sampler(SamplerBindingType::Filtering)
        }
        ResourceType::NonFilteringSampler => {
            driver::BindingType::Sampler(SamplerBindingType::NonFiltering)
        }
        ResourceType::ComparisonSampler => {
            driver::BindingType::Sampler(SamplerBindingType::Comparison)
        }
        // TODO: min_binding_size
        // TODO: dynamic offsets
        ResourceType::Uniform(_) => driver::BindingType::Buffer(BufferBindingType::Uniform),
        ResourceType::StorageRead(_) => {
            driver::BindingType::Buffer(BufferBindingType::ReadonlyStorage)
        }
        ResourceType::StorageReadWrite(_) => {
            driver::BindingType::Buffer(BufferBindingType::Storage)
        }
    }
}

fn texel_type_to_driver(texel_type: &TexelType) -> driver::TextureSampleType {
    match texel_type {
        TexelType::Float => driver::TextureSampleType::Float,
        TexelType::UnfilterableFloat => driver::TextureSampleType::UnfilterableFloat,
        TexelType::Integer => driver::TextureSampleType::SignedInteger,
        TexelType::UnsignedInteger => driver::TextureSampleType::UnsignedInteger,
    }
}

fn storage_texture_format_to_driver(format: &StorageTextureFormat) -> TextureFormatId {
    match format {
        StorageTextureFormat::rgba8unorm => TextureFormatId::rgba8unorm,
        StorageTextureFormat::rgba8snorm => TextureFormatId::rgba8snorm,
        StorageTextureFormat::rgba8uint => TextureFormatId::rgba8uint,
        StorageTextureFormat::rgba8sint => TextureFormatId::rgba8sint,
        StorageTextureFormat::rgba16uint => TextureFormatId::rgba16uint,
        StorageTextureFormat::rgba16sint => TextureFormatId::rgba16sint,
        StorageTextureFormat::rgba16float => TextureFormatId::rgba16float,
        StorageTextureFormat::r32uint => TextureFormatId::r32uint,
        StorageTextureFormat::r32sint => TextureFormatId::r32sint,
        StorageTextureFormat::r32float => TextureFormatId::r32float,
        StorageTextureFormat::rg32uint => TextureFormatId::rg32uint,
        StorageTextureFormat::rg32sint => TextureFormatId::rg32sint,
        StorageTextureFormat::rg32float => TextureFormatId::rg32float,
        StorageTextureFormat::rgba32uint => TextureFormatId::rgba32uint,
        StorageTextureFormat::rgba32sint => TextureFormatId::rgba32sint,
        StorageTextureFormat::rgba32float => TextureFormatId::rgba32float,
    }
}
