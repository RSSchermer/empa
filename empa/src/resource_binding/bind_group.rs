use crate::buffer::{Buffer, BufferDestroyer, ReadOnlyStorage, Storage, Uniform};
use crate::command::BindGroupEncoding;
use crate::device::Device;
use crate::device::ID_GEN;
use crate::resource_binding::typed_bind_group_entry::{
    f32_unfiltered, ShaderStages, TypedSlotBinding,
};
use crate::resource_binding::{
    typed_bind_group_entry, BindGroupLayout, TypedBindGroupLayout, TypedPipelineLayout,
};
use crate::sampler::{ComparisonSampler, NonFilteringSampler, Sampler};
use crate::texture::format::{
    FloatSamplable, SignedIntegerSamplable, Storable, UnfilteredFloatSamplable,
    UnsignedIntegerSamplable,
};
use crate::texture::{
    Sampled1DFloat, Sampled1DSignedInteger, Sampled1DUnfilteredFloat, Sampled1DUnsignedInteger,
    Sampled3DFloat, Sampled3DSignedInteger, Sampled3DUnfilteredFloat, Sampled3DUnsignedInteger,
    Storage1D, Storage3D, TextureDestroyer,
};
use crate::type_flag::O;
use crate::{abi, buffer, texture};
use atomic_counter::AtomicCounter;
use std::any::Any;
use std::marker;
use std::sync::Arc;
use wasm_bindgen::JsValue;
use web_sys::{
    GpuBindGroup, GpuBindGroupDescriptor, GpuBuffer, GpuBufferBinding, GpuSampler, GpuTextureView,
};

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

        for entry in resources.to_entries() {
            entries.push(entry.as_web_sys().as_ref());

            if let Some(destroyer) = entry.entry_destroyer() {
                resource_destroyers.push(destroyer);
            }
        }

        let mut desc = GpuBindGroupDescriptor::new(entries.as_ref(), layout.as_web_sys());
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

    type ToEntries: Iterator<Item = BindGroupEntry>;

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
