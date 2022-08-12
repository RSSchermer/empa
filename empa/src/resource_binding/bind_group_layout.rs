use std::marker;

use bitflags::bitflags;

use crate::abi::MemoryUnit;
use crate::device::Device;
use crate::resource_binding::typed_bind_group_entry::TypedSlotBinding;
use crate::Untyped;

use crate::texture::format::TextureFormatId;
use web_sys::{
    GpuBindGroupLayout, GpuBindGroupLayoutDescriptor, GpuBindGroupLayoutEntry,
    GpuBufferBindingLayout, GpuBufferBindingType, GpuSamplerBindingLayout, GpuSamplerBindingType,
    GpuStorageTextureAccess, GpuStorageTextureBindingLayout, GpuTextureBindingLayout,
    GpuTextureSampleType, GpuTextureViewDimension,
};

pub struct BindGroupLayoutEncoding {
    pub(crate) inner: web_sys::GpuBindGroupLayout,
}

pub struct BindGroupLayout<T = Untyped> {
    pub(crate) inner: web_sys::GpuBindGroupLayout,
    _marker: marker::PhantomData<*const T>,
}

impl<T> BindGroupLayout<T> {
    pub(crate) fn new(device: &Device, layout: &[Option<BindGroupLayoutEntry>]) -> Self {
        let layout = layout
            .iter()
            .enumerate()
            .filter(|(_, e)| e.is_some())
            .map(|(i, e)| {
                let e = e.as_ref().unwrap();

                let mut entry = GpuBindGroupLayoutEntry::new(i as u32, e.visibility.bits());

                match &e.binding_type {
                    BindingType::Texture1D(texel_type) => {
                        let mut texture = GpuTextureBindingLayout::new();

                        texture.sample_type(texel_type.to_web_sys());
                        texture.view_dimension(GpuTextureViewDimension::N1d);

                        entry.texture(&texture);
                    }
                    BindingType::Texture2D(texel_type) => {
                        let mut texture = GpuTextureBindingLayout::new();

                        texture.sample_type(texel_type.to_web_sys());
                        texture.view_dimension(GpuTextureViewDimension::N2d);

                        entry.texture(&texture);
                    }
                    BindingType::Texture3D(texel_type) => {
                        let mut texture = GpuTextureBindingLayout::new();

                        texture.sample_type(texel_type.to_web_sys());
                        texture.view_dimension(GpuTextureViewDimension::N3d);

                        entry.texture(&texture);
                    }
                    BindingType::Texture2DArray(texel_type) => {
                        let mut texture = GpuTextureBindingLayout::new();

                        texture.sample_type(texel_type.to_web_sys());
                        texture.view_dimension(GpuTextureViewDimension::N2dArray);

                        entry.texture(&texture);
                    }
                    BindingType::TextureCube(texel_type) => {
                        let mut texture = GpuTextureBindingLayout::new();

                        texture.sample_type(texel_type.to_web_sys());
                        texture.view_dimension(GpuTextureViewDimension::Cube);

                        entry.texture(&texture);
                    }
                    BindingType::TextureCubeArray(texel_type) => {
                        let mut texture = GpuTextureBindingLayout::new();

                        texture.sample_type(texel_type.to_web_sys());
                        texture.view_dimension(GpuTextureViewDimension::CubeArray);

                        entry.texture(&texture);
                    }
                    BindingType::TextureMultisampled2D(texel_type) => {
                        let mut texture = GpuTextureBindingLayout::new();

                        texture.multisampled(true);
                        texture.sample_type(texel_type.to_web_sys());
                        texture.view_dimension(GpuTextureViewDimension::N2d);

                        entry.texture(&texture);
                    }
                    BindingType::TextureDepth2D => {
                        let mut texture = GpuTextureBindingLayout::new();

                        texture.sample_type(GpuTextureSampleType::Depth);
                        texture.view_dimension(GpuTextureViewDimension::N2d);

                        entry.texture(&texture);
                    }
                    BindingType::TextureDepth2DArray => {
                        let mut texture = GpuTextureBindingLayout::new();

                        texture.sample_type(GpuTextureSampleType::Depth);
                        texture.view_dimension(GpuTextureViewDimension::N2dArray);

                        entry.texture(&texture);
                    }
                    BindingType::TextureDepthCube => {
                        let mut texture = GpuTextureBindingLayout::new();

                        texture.sample_type(GpuTextureSampleType::Depth);
                        texture.view_dimension(GpuTextureViewDimension::Cube);

                        entry.texture(&texture);
                    }
                    BindingType::TextureDepthCubeArray => {
                        let mut texture = GpuTextureBindingLayout::new();

                        texture.sample_type(GpuTextureSampleType::Depth);
                        texture.view_dimension(GpuTextureViewDimension::CubeArray);

                        entry.texture(&texture);
                    }
                    BindingType::TextureDepthMultisampled2D => {
                        let mut texture = GpuTextureBindingLayout::new();

                        texture.multisampled(true);
                        texture.sample_type(GpuTextureSampleType::Depth);
                        texture.view_dimension(GpuTextureViewDimension::N2d);

                        entry.texture(&texture);
                    }
                    BindingType::StorageTexture1D(format) => {
                        let mut storage_texture =
                            GpuStorageTextureBindingLayout::new(format.to_web_sys());

                        storage_texture.access(GpuStorageTextureAccess::WriteOnly);
                        storage_texture.view_dimension(GpuTextureViewDimension::N1d);

                        entry.storage_texture(&storage_texture);
                    }
                    BindingType::StorageTexture2D(format) => {
                        let mut storage_texture =
                            GpuStorageTextureBindingLayout::new(format.to_web_sys());

                        storage_texture.access(GpuStorageTextureAccess::WriteOnly);
                        storage_texture.view_dimension(GpuTextureViewDimension::N2d);

                        entry.storage_texture(&storage_texture);
                    }
                    BindingType::StorageTexture2DArray(format) => {
                        let mut storage_texture =
                            GpuStorageTextureBindingLayout::new(format.to_web_sys());

                        storage_texture.access(GpuStorageTextureAccess::WriteOnly);
                        storage_texture.view_dimension(GpuTextureViewDimension::N2dArray);

                        entry.storage_texture(&storage_texture);
                    }
                    BindingType::StorageTexture3D(format) => {
                        let mut storage_texture =
                            GpuStorageTextureBindingLayout::new(format.to_web_sys());

                        storage_texture.access(GpuStorageTextureAccess::WriteOnly);
                        storage_texture.view_dimension(GpuTextureViewDimension::N3d);

                        entry.storage_texture(&storage_texture);
                    }
                    BindingType::FilteringSampler => {
                        let mut sampler = GpuSamplerBindingLayout::new();

                        sampler.type_(GpuSamplerBindingType::Filtering);

                        entry.sampler(&sampler);
                    }
                    BindingType::NonFilteringSampler => {
                        let mut sampler = GpuSamplerBindingLayout::new();

                        sampler.type_(GpuSamplerBindingType::NonFiltering);

                        entry.sampler(&sampler);
                    }
                    BindingType::ComparisonSampler => {
                        let mut sampler = GpuSamplerBindingLayout::new();

                        sampler.type_(GpuSamplerBindingType::Comparison);

                        entry.sampler(&sampler);
                    }
                    // TODO: min_binding_size
                    // TODO: dynamic offsets
                    BindingType::Uniform(_) => {
                        let mut buffer = GpuBufferBindingLayout::new();

                        buffer.type_(GpuBufferBindingType::Uniform);

                        entry.buffer(&buffer);
                    }
                    BindingType::Storage(_) => {
                        let mut buffer = GpuBufferBindingLayout::new();

                        buffer.type_(GpuBufferBindingType::Storage);

                        entry.buffer(&buffer);
                    }
                    BindingType::ReadOnlyStorage(_) => {
                        let mut buffer = GpuBufferBindingLayout::new();

                        buffer.type_(GpuBufferBindingType::ReadOnlyStorage);

                        entry.buffer(&buffer);
                    }
                }

                entry
            })
            .collect::<js_sys::Array>();

        let desc = GpuBindGroupLayoutDescriptor::new(&layout);
        let inner = device.inner.create_bind_group_layout(&desc);

        BindGroupLayout {
            inner,
            _marker: marker::PhantomData,
        }
    }

    pub fn to_encoding(&self) -> BindGroupLayoutEncoding {
        BindGroupLayoutEncoding {
            inner: self.inner.clone(),
        }
    }

    pub(crate) fn as_web_sys(&self) -> &GpuBindGroupLayout {
        &self.inner
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
    pub visibility: ShaderStages,
    pub binding_type: BindingType,
}

// Copied/Modified from `wgpu::ShaderStages`.
bitflags! {
    /// Describes the shader stages that a resource will be visible from.
    ///
    /// These can be combined so something that is visible from both vertex and fragment shaders can be defined as:
    ///
    /// `ShaderStages::VERTEX | ShaderStages::FRAGMENT`
    #[repr(transparent)]
    pub struct ShaderStages: u32 {
        /// Resource is not visible from any shader stage.
        const NONE = 0;
        /// Resource is visible from the vertex shader of a render pipeline.
        const VERTEX = 1 << 0;
        /// Resource is visible from the fragment shader of a render pipeline.
        const FRAGMENT = 1 << 1;
        /// Resource is visible from the compute shader of a compute pipeline.
        const COMPUTE = 1 << 2;
        /// Resource is visible from the vertex and fragment shaders of a render pipeline.
        const VERTEX_FRAGMENT = Self::VERTEX.bits | Self::FRAGMENT.bits;
    }
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

#[derive(Clone, Copy, Debug)]
pub enum TexelType {
    Float,
    UnfilterableFloat,
    Integer,
    UnsignedInteger,
}

impl TexelType {
    pub(crate) fn to_web_sys(&self) -> GpuTextureSampleType {
        match self {
            TexelType::Float => GpuTextureSampleType::Float,
            TexelType::UnfilterableFloat => GpuTextureSampleType::UnfilterableFloat,
            TexelType::Integer => GpuTextureSampleType::Sint,
            TexelType::UnsignedInteger => GpuTextureSampleType::Uint,
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
            (TexelType::Integer, TexelType::Integer) => true,
            (TexelType::UnsignedInteger, TexelType::UnsignedInteger) => true,
            (TexelType::Float, TexelType::UnfilterableFloat) => true,
            (TexelType::UnfilterableFloat, TexelType::Float) => true,
            _ => false
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
