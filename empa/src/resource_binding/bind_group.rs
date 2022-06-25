use std::marker;
use std::sync::Arc;

use atomic_counter::AtomicCounter;
use wasm_bindgen::JsValue;
use web_sys::{
    GpuBindGroup, GpuBindGroupDescriptor, GpuBindGroupEntry, GpuBufferBinding, GpuSampler,
    GpuTextureView,
};

use crate::abi;
use crate::buffer::{BufferDestroyer, ReadOnlyStorage, Storage, Uniform};
use crate::command::BindGroupEncoding;
use crate::device::{Device, ID_GEN};
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
    SampledCubeUnsignedInteger, Storage1D, Storage2D, Storage2DArray, Storage3D, TextureDestroyer,
};
use crate::type_flag::O;

pub(crate) enum EntryDestroyer {
    Buffer(Arc<BufferDestroyer>),
    Texture(Arc<TextureDestroyer>),
}

pub struct BindGroup<T> {
    inner: GpuBindGroup,
    id: usize,
    _resource_destroyers: Arc<Vec<EntryDestroyer>>,
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
        let entries = js_sys::Array::new();
        let mut resource_destroyers = Vec::new();

        for (binding, entry) in resources.to_entries().enumerate() {
            if let Some(entry) = entry {
                let web_sys_entry =
                    GpuBindGroupEntry::new(binding as u32, entry.as_web_sys().as_ref());

                entries.push(web_sys_entry.as_ref());

                if let Some(destroyer) = entry.entry_destroyer() {
                    resource_destroyers.push(destroyer);
                }
            }
        }

        let desc = GpuBindGroupDescriptor::new(entries.as_ref(), layout.as_web_sys());
        let inner = device.inner.create_bind_group(&desc);

        BindGroup {
            inner,
            id,
            _resource_destroyers: Arc::new(resource_destroyers),
            _marker: Default::default(),
        }
    }
}

impl<T> BindGroup<T> {
    pub fn to_encoding(&self) -> BindGroupEncoding {
        BindGroupEncoding {
            bind_group: self.inner.clone(),
            id: self.id,
            _resource_destroyers: self._resource_destroyers.clone(),
        }
    }
}

pub enum BindGroupEntry {
    BufferView(BufferViewResource),
    TextureView(TextureViewResource),
    Sampler(SamplerResource),
}

impl BindGroupEntry {
    pub(crate) fn as_web_sys(&self) -> &JsValue {
        match self {
            BindGroupEntry::BufferView(e) => e.inner.as_ref(),
            BindGroupEntry::TextureView(e) => e.inner.as_ref(),
            BindGroupEntry::Sampler(e) => e.inner.as_ref(),
        }
    }

    pub(crate) fn entry_destroyer(&self) -> Option<EntryDestroyer> {
        match self {
            BindGroupEntry::BufferView(e) => Some(EntryDestroyer::Buffer(e._destroyer.clone())),
            BindGroupEntry::TextureView(e) => Some(EntryDestroyer::Texture(e._destroyer.clone())),
            BindGroupEntry::Sampler(_) => None,
        }
    }
}

pub struct BufferViewResource {
    inner: GpuBufferBinding,
    _destroyer: Arc<BufferDestroyer>,
}

pub struct TextureViewResource {
    inner: GpuTextureView,
    _destroyer: Arc<TextureDestroyer>,
}

pub struct SamplerResource {
    inner: GpuSampler,
}

pub unsafe trait Resources {
    type Layout: TypedBindGroupLayout;

    type ToEntries: Iterator<Item = Option<BindGroupEntry>>;

    fn to_entries(&self) -> Self::ToEntries;
}

pub unsafe trait Resource {
    type Binding: TypedSlotBinding;

    fn to_entry(&self) -> BindGroupEntry;
}

unsafe impl<'a> Resource for &'a Sampled1DFloat {
    type Binding = typed_bind_group_entry::Texture1D<f32, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a Sampled1DUnfilteredFloat {
    type Binding = typed_bind_group_entry::Texture1D<f32_unfiltered, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a Sampled1DSignedInteger {
    type Binding = typed_bind_group_entry::Texture1D<i32, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a Sampled1DUnsignedInteger {
    type Binding = typed_bind_group_entry::Texture1D<u32, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a, F> Resource for &'a Storage1D<F>
where
    F: Storable,
{
    type Binding = typed_bind_group_entry::StorageTexture1D<F, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a Sampled2DFloat {
    type Binding = typed_bind_group_entry::Texture2D<f32, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a Sampled2DUnfilteredFloat {
    type Binding = typed_bind_group_entry::Texture2D<f32_unfiltered, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a Sampled2DSignedInteger {
    type Binding = typed_bind_group_entry::Texture2D<i32, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a Sampled2DUnsignedInteger {
    type Binding = typed_bind_group_entry::Texture2D<u32, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a Sampled2DDepth {
    type Binding = typed_bind_group_entry::TextureDepth2D<ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a Sampled2DArrayFloat {
    type Binding = typed_bind_group_entry::Texture2DArray<f32, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a Sampled2DArrayUnfilteredFloat {
    type Binding = typed_bind_group_entry::Texture2DArray<f32_unfiltered, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a Sampled2DArraySignedInteger {
    type Binding = typed_bind_group_entry::Texture2DArray<i32, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a Sampled2DArrayUnsignedInteger {
    type Binding = typed_bind_group_entry::Texture2DArray<u32, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a Sampled2DArrayDepth {
    type Binding = typed_bind_group_entry::TextureDepth2DArray<ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a SampledCubeFloat {
    type Binding = typed_bind_group_entry::TextureCube<f32, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a SampledCubeUnfilteredFloat {
    type Binding = typed_bind_group_entry::TextureCube<f32_unfiltered, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a SampledCubeSignedInteger {
    type Binding = typed_bind_group_entry::TextureCube<i32, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a SampledCubeUnsignedInteger {
    type Binding = typed_bind_group_entry::TextureCube<u32, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a SampledCubeDepth {
    type Binding = typed_bind_group_entry::TextureDepthCube<ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a SampledCubeArrayFloat {
    type Binding = typed_bind_group_entry::TextureCubeArray<f32, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a SampledCubeArrayUnfilteredFloat {
    type Binding = typed_bind_group_entry::TextureCubeArray<f32_unfiltered, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a SampledCubeArraySignedInteger {
    type Binding = typed_bind_group_entry::TextureCubeArray<i32, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a SampledCubeArrayUnsignedInteger {
    type Binding = typed_bind_group_entry::TextureCubeArray<u32, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a SampledCubeArrayDepth {
    type Binding = typed_bind_group_entry::TextureDepthCubeArray<ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a, F> Resource for &'a Storage2D<F>
where
    F: Storable,
{
    type Binding = typed_bind_group_entry::StorageTexture2D<F, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a, F> Resource for &'a Storage2DArray<F>
where
    F: Storable,
{
    type Binding = typed_bind_group_entry::StorageTexture2DArray<F, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a Sampled3DFloat {
    type Binding = typed_bind_group_entry::Texture3D<f32, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a Sampled3DUnfilteredFloat {
    type Binding = typed_bind_group_entry::Texture3D<f32_unfiltered, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a Sampled3DSignedInteger {
    type Binding = typed_bind_group_entry::Texture3D<i32, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a Sampled3DUnsignedInteger {
    type Binding = typed_bind_group_entry::Texture3D<u32, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a, F> Resource for &'a Storage3D<F>
where
    F: Storable,
{
    type Binding = typed_bind_group_entry::StorageTexture3D<F, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::TextureView(TextureViewResource {
            inner: self.inner.clone(),
            _destroyer: self.texture_destroyer.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a Sampler {
    type Binding = typed_bind_group_entry::FilteringSampler<ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::Sampler(SamplerResource {
            inner: self.inner.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a ComparisonSampler {
    type Binding = typed_bind_group_entry::ComparisonSampler<ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::Sampler(SamplerResource {
            inner: self.inner.clone(),
        })
    }
}

unsafe impl<'a> Resource for &'a NonFilteringSampler {
    type Binding = typed_bind_group_entry::NonFilteringSampler<ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        BindGroupEntry::Sampler(SamplerResource {
            inner: self.inner.clone(),
        })
    }
}

unsafe impl<'a, T> Resource for &'a Uniform<T>
where
    T: abi::Sized,
{
    type Binding = typed_bind_group_entry::Uniform<T, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        let mut inner = GpuBufferBinding::new(&self.inner.buffer);

        inner.offset(self.offset as f64);
        inner.size(self.size as f64);

        BindGroupEntry::BufferView(BufferViewResource {
            inner,
            _destroyer: self.inner.clone(),
        })
    }
}

unsafe impl<'a, T> Resource for &'a Storage<T>
where
    T: abi::Unsized + ?Sized,
{
    type Binding = typed_bind_group_entry::Storage<T, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        let mut inner = GpuBufferBinding::new(&self.inner.buffer);

        inner.offset(self.offset as f64);
        inner.size(self.size as f64);

        BindGroupEntry::BufferView(BufferViewResource {
            inner,
            _destroyer: self.inner.clone(),
        })
    }
}

unsafe impl<'a, T> Resource for &'a ReadOnlyStorage<T>
where
    T: abi::Unsized + ?Sized,
{
    type Binding = typed_bind_group_entry::ReadOnlyStorage<T, ShaderStages<O, O, O>>;

    fn to_entry(&self) -> BindGroupEntry {
        let mut inner = GpuBufferBinding::new(&self.inner.buffer);

        inner.offset(self.offset as f64);
        inner.size(self.size as f64);

        BindGroupEntry::BufferView(BufferViewResource {
            inner,
            _destroyer: self.inner.clone(),
        })
    }
}
