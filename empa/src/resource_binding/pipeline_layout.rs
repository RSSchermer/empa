use crate::device::Device;
use crate::resource_binding::{BindGroupLayout, BindGroupLayoutEntry, TypedBindGroupLayout};
use crate::Untyped;
use std::marker;
use web_sys::{GpuPipelineLayout, GpuPipelineLayoutDescriptor};

mod typed_pipeline_layout_seal {
    pub trait Seal {}
}

pub struct PipelineLayout<T> {
    pub(crate) inner: GpuPipelineLayout,
    _marker: marker::PhantomData<*const T>,
}

impl<T> PipelineLayout<T>
where
    T: TypedPipelineLayout,
{
    pub(crate) fn typed(device: &Device) -> Self {
        let layout = T::BIND_GROUP_LAYOUTS;

        let bind_group_layouts = layout
            .iter()
            .map(|entries| BindGroupLayout::<Untyped>::new(device, entries).inner)
            .collect::<js_sys::Array>();

        let desc = GpuPipelineLayoutDescriptor::new(bind_group_layouts.as_ref());
        let inner = device.inner.create_pipeline_layout(&desc);

        PipelineLayout {
            inner,
            _marker: Default::default(),
        }
    }
}

pub trait TypedPipelineLayout: typed_pipeline_layout_seal::Seal {
    const BIND_GROUP_LAYOUTS: &'static [&'static [Option<BindGroupLayoutEntry>]];
}

macro_rules! impl_typed_pipeline_layout {
    ($($B:ident),*) => {
        impl<$($B),*> typed_pipeline_layout_seal::Seal for ($($B,)*) where $($B: TypedBindGroupLayout),* {}
        impl<$($B),*> TypedPipelineLayout for ($($B,)*) where $($B: TypedBindGroupLayout),* {
            const BIND_GROUP_LAYOUTS: &'static [&'static [Option<BindGroupLayoutEntry>]] = &[
                $($B::BIND_GROUP_LAYOUT),*
            ];
        }
    }
}

impl_typed_pipeline_layout!(B0);
impl_typed_pipeline_layout!(B0, B1);
impl_typed_pipeline_layout!(B0, B1, B2);
impl_typed_pipeline_layout!(B0, B1, B2, B3);
