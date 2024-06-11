use std::marker;

use atomic_counter::AtomicCounter;

use crate::access_mode::{Read, ReadWrite};
use crate::buffer::{Storage, Uniform};
use crate::command::BindGroupEncoding;
use crate::device::{Device, ID_GEN};
use crate::driver::{BindGroupDescriptor, BindingResource, Device as _, Driver, Dvr};
use crate::resource_binding::typed_bind_group_entry::{
    f32_unfiltered, ShaderStages, TypedSlotBinding,
};
use crate::resource_binding::{typed_bind_group_entry, BindGroupLayout, TypedBindGroupLayout};
use crate::sampler::{ComparisonSampler, NonFilteringSampler, Sampler};
use crate::texture::format::Storable;
use crate::texture::{
    Sampled1DFloat, Sampled1DSignedInteger, Sampled1DUnfilteredFloat, Sampled1DUnsignedInteger,
    Sampled2DArrayDepth, Sampled2DArrayFloat, Sampled2DArraySignedInteger,
    Sampled2DArrayUnfilteredFloat, Sampled2DArrayUnsignedInteger, Sampled2DDepth, Sampled2DFloat,
    Sampled2DSignedInteger, Sampled2DUnfilteredFloat, Sampled2DUnsignedInteger, Sampled3DFloat,
    Sampled3DSignedInteger, Sampled3DUnfilteredFloat, Sampled3DUnsignedInteger,
    SampledCubeArrayDepth, SampledCubeArrayFloat, SampledCubeArraySignedInteger,
    SampledCubeArrayUnfilteredFloat, SampledCubeArrayUnsignedInteger, SampledCubeDepth,
    SampledCubeFloat, SampledCubeSignedInteger, SampledCubeUnfilteredFloat,
    SampledCubeUnsignedInteger, Storage1D, Storage2D, Storage2DArray, Storage3D,
};
use crate::type_flag::O;
use crate::{abi, driver};

pub struct BindGroup<T> {
    handle: <Dvr as Driver>::BindGroupHandle,
    id: usize,
    _marker: marker::PhantomData<*const T>,
}

impl<T> BindGroup<T>
where
    T: TypedBindGroupLayout,
{
    pub(crate) fn new<R>(device: &Device, layout: &BindGroupLayout<T>, resources: R) -> Self
    where
        R: Resources<Layout = T>,
    {
        let id = ID_GEN.get();
        let handle = device.device_handle.create_bind_group(BindGroupDescriptor {
            layout: &layout.handle,
            entries: resources
                .to_entries()
                .as_ref()
                .iter()
                .map(|e| driver::BindGroupEntry {
                    binding: e.binding,
                    resource: e.resource.inner.clone(),
                }),
        });

        BindGroup {
            handle,
            id,
            _marker: Default::default(),
        }
    }
}

impl<T> BindGroup<T> {
    pub fn to_encoding(&self) -> BindGroupEncoding {
        BindGroupEncoding {
            bind_group_handle: self.handle.clone(),
            id: self.id,
        }
    }
}

pub struct BindGroupEntry<'a> {
    pub binding: u32,
    pub resource: ResourceEncoding<'a>,
}

pub unsafe trait Resources {
    type Layout: TypedBindGroupLayout;

    type ToEntries<'a>: AsRef<[BindGroupEntry<'a>]>
    where
        Self: 'a;

    fn to_entries<'a>(&'a self) -> Self::ToEntries<'a>;
}

pub unsafe trait Resource {
    type Binding: TypedSlotBinding;

    fn to_encoding(&self) -> ResourceEncoding;
}

pub struct ResourceEncoding<'a> {
    pub(crate) inner: BindingResource<'a, Dvr>,
}

impl<'a> From<BindingResource<'a, Dvr>> for ResourceEncoding<'a> {
    fn from(inner: BindingResource<'a, Dvr>) -> Self {
        ResourceEncoding { inner }
    }
}

unsafe impl<T> Resource for &'_ T
where
    T: Resource,
{
    type Binding = T::Binding;

    fn to_encoding(&self) -> ResourceEncoding {
        <T as Resource>::to_encoding(self)
    }
}

unsafe impl Resource for Sampled1DFloat<'_> {
    type Binding = typed_bind_group_entry::Texture1D<f32, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl Resource for Sampled1DUnfilteredFloat<'_> {
    type Binding = typed_bind_group_entry::Texture1D<f32_unfiltered, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl Resource for Sampled1DSignedInteger<'_> {
    type Binding = typed_bind_group_entry::Texture1D<i32, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl Resource for Sampled1DUnsignedInteger<'_> {
    type Binding = typed_bind_group_entry::Texture1D<u32, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl<F> Resource for Storage1D<'_, F>
where
    F: Storable,
{
    type Binding = typed_bind_group_entry::StorageTexture1D<F, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl Resource for Sampled2DFloat<'_> {
    type Binding = typed_bind_group_entry::Texture2D<f32, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl Resource for Sampled2DUnfilteredFloat<'_> {
    type Binding = typed_bind_group_entry::Texture2D<f32_unfiltered, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl Resource for Sampled2DSignedInteger<'_> {
    type Binding = typed_bind_group_entry::Texture2D<i32, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl Resource for Sampled2DUnsignedInteger<'_> {
    type Binding = typed_bind_group_entry::Texture2D<u32, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl Resource for Sampled2DDepth<'_> {
    type Binding = typed_bind_group_entry::TextureDepth2D<ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl Resource for Sampled2DArrayFloat<'_> {
    type Binding = typed_bind_group_entry::Texture2DArray<f32, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl Resource for Sampled2DArrayUnfilteredFloat<'_> {
    type Binding = typed_bind_group_entry::Texture2DArray<f32_unfiltered, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl Resource for Sampled2DArraySignedInteger<'_> {
    type Binding = typed_bind_group_entry::Texture2DArray<i32, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl Resource for Sampled2DArrayUnsignedInteger<'_> {
    type Binding = typed_bind_group_entry::Texture2DArray<u32, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl Resource for Sampled2DArrayDepth<'_> {
    type Binding = typed_bind_group_entry::TextureDepth2DArray<ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl Resource for SampledCubeFloat<'_> {
    type Binding = typed_bind_group_entry::TextureCube<f32, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl Resource for SampledCubeUnfilteredFloat<'_> {
    type Binding = typed_bind_group_entry::TextureCube<f32_unfiltered, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl Resource for SampledCubeSignedInteger<'_> {
    type Binding = typed_bind_group_entry::TextureCube<i32, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl Resource for SampledCubeUnsignedInteger<'_> {
    type Binding = typed_bind_group_entry::TextureCube<u32, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl Resource for SampledCubeDepth<'_> {
    type Binding = typed_bind_group_entry::TextureDepthCube<ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl Resource for SampledCubeArrayFloat<'_> {
    type Binding = typed_bind_group_entry::TextureCubeArray<f32, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl Resource for SampledCubeArrayUnfilteredFloat<'_> {
    type Binding = typed_bind_group_entry::TextureCubeArray<f32_unfiltered, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl Resource for SampledCubeArraySignedInteger<'_> {
    type Binding = typed_bind_group_entry::TextureCubeArray<i32, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl Resource for SampledCubeArrayUnsignedInteger<'_> {
    type Binding = typed_bind_group_entry::TextureCubeArray<u32, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl Resource for SampledCubeArrayDepth<'_> {
    type Binding = typed_bind_group_entry::TextureDepthCubeArray<ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl<F> Resource for Storage2D<'_, F>
where
    F: Storable,
{
    type Binding = typed_bind_group_entry::StorageTexture2D<F, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl<F> Resource for Storage2DArray<'_, F>
where
    F: Storable,
{
    type Binding = typed_bind_group_entry::StorageTexture2DArray<F, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl Resource for Sampled3DFloat<'_> {
    type Binding = typed_bind_group_entry::Texture3D<f32, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl Resource for Sampled3DUnfilteredFloat<'_> {
    type Binding = typed_bind_group_entry::Texture3D<f32_unfiltered, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl Resource for Sampled3DSignedInteger<'_> {
    type Binding = typed_bind_group_entry::Texture3D<i32, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl Resource for Sampled3DUnsignedInteger<'_> {
    type Binding = typed_bind_group_entry::Texture3D<u32, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl<F> Resource for Storage3D<'_, F>
where
    F: Storable,
{
    type Binding = typed_bind_group_entry::StorageTexture3D<F, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::TextureView(self.inner.clone()).into()
    }
}

unsafe impl Resource for Sampler {
    type Binding = typed_bind_group_entry::FilteringSampler<ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::Sampler(&self.handle).into()
    }
}

unsafe impl Resource for ComparisonSampler {
    type Binding = typed_bind_group_entry::ComparisonSampler<ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::Sampler(&self.handle).into()
    }
}

unsafe impl Resource for NonFilteringSampler {
    type Binding = typed_bind_group_entry::NonFilteringSampler<ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::Sampler(&self.handle).into()
    }
}

unsafe impl<T> Resource for Uniform<'_, T>
where
    T: abi::Sized,
{
    type Binding = typed_bind_group_entry::Uniform<T, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::BufferBinding(self.inner.clone()).into()
    }
}

unsafe impl<T> Resource for Storage<'_, T, Read>
where
    T: abi::Unsized + ?Sized,
{
    type Binding = typed_bind_group_entry::Storage<T, Read, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::BufferBinding(self.inner.clone()).into()
    }
}

unsafe impl<T> Resource for Storage<'_, T, ReadWrite>
where
    T: abi::Unsized + ?Sized,
{
    type Binding = typed_bind_group_entry::Storage<T, ReadWrite, ShaderStages<O, O, O>>;

    fn to_encoding(&self) -> ResourceEncoding {
        BindingResource::BufferBinding(self.inner.clone()).into()
    }
}
