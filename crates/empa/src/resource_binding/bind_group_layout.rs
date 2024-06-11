use std::marker;

use flagset::FlagSet;

use crate::abi::MemoryUnit;
use crate::device::Device;
use crate::driver::{
    BufferBindingType, Device as _, Driver, Dvr, SamplerBindingType, ShaderStage,
    StorageTextureAccess, TextureSampleType, TextureViewDimension,
};
use crate::resource_binding::typed_bind_group_entry::TypedSlotBinding;
use crate::texture::format::TextureFormatId;
use crate::{driver, Untyped};

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
                    binding_type: e.binding_type.to_driver(),
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

    pub fn to_encoding(&self) -> BindGroupLayoutEncoding {
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
impl_typed_bind_group_layout!(B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11, B12, B13, B14);
impl_typed_bind_group_layout!(B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11, B12, B13, B14, B15);
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
    pub binding_type: BindingType,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BindingType {
    Texture1D(TexelType),
    Texture2D(TexelType),
    Texture3D(TexelType),
    Texture2DArray(TexelType),
    TextureCube(TexelType),
    TextureCubeArray(TexelType),
    TextureMultisampled2D(TexelType),
    TextureDepth2D,
    TextureDepth2DArray,
    TextureDepthCube,
    TextureDepthCubeArray,
    TextureDepthMultisampled2D,
    StorageTexture1D(TextureFormatId),
    StorageTexture2D(TextureFormatId),
    StorageTexture2DArray(TextureFormatId),
    StorageTexture3D(TextureFormatId),
    FilteringSampler,
    NonFilteringSampler,
    ComparisonSampler,
    Uniform(SizedBufferLayout),
    Storage(UnsizedBufferLayout),
    ReadOnlyStorage(UnsizedBufferLayout),
}

impl BindingType {
    fn to_driver(&self) -> driver::BindingType {
        match self {
            BindingType::Texture1D(texel_type) => driver::BindingType::Texture {
                sample_type: texel_type.to_driver(),
                dimension: TextureViewDimension::One,
                multisampled: false,
            },
            BindingType::Texture2D(texel_type) => driver::BindingType::Texture {
                sample_type: texel_type.to_driver(),
                dimension: TextureViewDimension::Two,
                multisampled: false,
            },
            BindingType::Texture3D(texel_type) => driver::BindingType::Texture {
                sample_type: texel_type.to_driver(),
                dimension: TextureViewDimension::Three,
                multisampled: false,
            },
            BindingType::Texture2DArray(texel_type) => driver::BindingType::Texture {
                sample_type: texel_type.to_driver(),
                dimension: TextureViewDimension::TwoArray,
                multisampled: false,
            },
            BindingType::TextureCube(texel_type) => driver::BindingType::Texture {
                sample_type: texel_type.to_driver(),
                dimension: TextureViewDimension::Cube,
                multisampled: false,
            },
            BindingType::TextureCubeArray(texel_type) => driver::BindingType::Texture {
                sample_type: texel_type.to_driver(),
                dimension: TextureViewDimension::CubeArray,
                multisampled: false,
            },
            BindingType::TextureMultisampled2D(texel_type) => driver::BindingType::Texture {
                sample_type: texel_type.to_driver(),
                dimension: TextureViewDimension::Two,
                multisampled: true,
            },
            BindingType::TextureDepth2D => driver::BindingType::Texture {
                sample_type: TextureSampleType::Depth,
                dimension: TextureViewDimension::Two,
                multisampled: false,
            },
            BindingType::TextureDepth2DArray => driver::BindingType::Texture {
                sample_type: TextureSampleType::Depth,
                dimension: TextureViewDimension::TwoArray,
                multisampled: false,
            },
            BindingType::TextureDepthCube => driver::BindingType::Texture {
                sample_type: TextureSampleType::Depth,
                dimension: TextureViewDimension::Cube,
                multisampled: false,
            },
            BindingType::TextureDepthCubeArray => driver::BindingType::Texture {
                sample_type: TextureSampleType::Depth,
                dimension: TextureViewDimension::CubeArray,
                multisampled: false,
            },
            BindingType::TextureDepthMultisampled2D => driver::BindingType::Texture {
                sample_type: TextureSampleType::Depth,
                dimension: TextureViewDimension::Two,
                multisampled: true,
            },
            BindingType::StorageTexture1D(format) => driver::BindingType::StorageTexture {
                access: StorageTextureAccess::WriteOnly,
                dimension: TextureViewDimension::One,
                format: *format,
            },
            BindingType::StorageTexture2D(format) => driver::BindingType::StorageTexture {
                access: StorageTextureAccess::WriteOnly,
                dimension: TextureViewDimension::Two,
                format: *format,
            },
            BindingType::StorageTexture2DArray(format) => driver::BindingType::StorageTexture {
                access: StorageTextureAccess::WriteOnly,
                dimension: TextureViewDimension::TwoArray,
                format: *format,
            },
            BindingType::StorageTexture3D(format) => driver::BindingType::StorageTexture {
                access: StorageTextureAccess::WriteOnly,
                dimension: TextureViewDimension::Three,
                format: *format,
            },
            BindingType::FilteringSampler => {
                driver::BindingType::Sampler(SamplerBindingType::Filtering)
            }
            BindingType::NonFilteringSampler => {
                driver::BindingType::Sampler(SamplerBindingType::NonFiltering)
            }
            BindingType::ComparisonSampler => {
                driver::BindingType::Sampler(SamplerBindingType::Comparison)
            }
            // TODO: min_binding_size
            // TODO: dynamic offsets
            BindingType::Uniform(_) => driver::BindingType::Buffer(BufferBindingType::Uniform),
            BindingType::Storage(_) => driver::BindingType::Buffer(BufferBindingType::Storage),
            BindingType::ReadOnlyStorage(_) => {
                driver::BindingType::Buffer(BufferBindingType::ReadonlyStorage)
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TexelType {
    Float,
    UnfilterableFloat,
    SignedInteger,
    UnsignedInteger,
}

impl TexelType {
    pub(crate) fn to_driver(&self) -> driver::TextureSampleType {
        match self {
            TexelType::Float => driver::TextureSampleType::Float,
            TexelType::UnfilterableFloat => driver::TextureSampleType::UnfilterableFloat,
            TexelType::SignedInteger => driver::TextureSampleType::SignedInteger,
            TexelType::UnsignedInteger => driver::TextureSampleType::UnsignedInteger,
        }
    }
}

impl PartialEq for TexelType {
    fn eq(&self, other: &Self) -> bool {
        // TODO: this is a temporary stop-gap solution around Naga's lack of distinction between
        // filtered and unfiltered float types.
        match (*self, *other) {
            (TexelType::Float, TexelType::Float) => true,
            (TexelType::UnfilterableFloat, TexelType::UnfilterableFloat) => true,
            (TexelType::SignedInteger, TexelType::SignedInteger) => true,
            (TexelType::UnsignedInteger, TexelType::UnsignedInteger) => true,
            (TexelType::Float, TexelType::UnfilterableFloat) => true,
            (TexelType::UnfilterableFloat, TexelType::Float) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SizedBufferLayout(pub &'static [MemoryUnit]);

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct UnsizedBufferLayout {
    pub sized_head: &'static [MemoryUnit],
    pub unsized_tail: Option<&'static [MemoryUnit]>,
}
