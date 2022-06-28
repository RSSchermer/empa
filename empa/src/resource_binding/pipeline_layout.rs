use std::{iter, marker};

use web_sys::{GpuPipelineLayout, GpuPipelineLayoutDescriptor};

use crate::device::Device;
use crate::resource_binding::{
    BindGroupLayout, BindGroupLayoutEncoding, BindGroupLayoutEntry, TypedBindGroupLayout,
};

pub struct PipelineLayout<T> {
    pub(crate) inner: GpuPipelineLayout,
    _marker: marker::PhantomData<*const T>,
}

impl<T> PipelineLayout<T>
where
    T: TypedPipelineLayout,
{
    pub(crate) fn typed<B>(device: &Device, bind_group_layouts: B) -> Self
    where
        B: BindGroupLayouts<PipelineLayout = T>,
    {
        let bind_group_layouts = bind_group_layouts
            .encodings()
            .map(|l| l.inner)
            .collect::<js_sys::Array>();

        let desc = GpuPipelineLayoutDescriptor::new(bind_group_layouts.as_ref());
        let inner = device.inner.create_pipeline_layout(&desc);

        PipelineLayout {
            inner,
            _marker: Default::default(),
        }
    }
}

mod typed_pipeline_layout_seal {
    pub trait Seal {}
}

pub trait TypedPipelineLayout: typed_pipeline_layout_seal::Seal {
    const BIND_GROUP_LAYOUTS: &'static [&'static [Option<BindGroupLayoutEntry>]];
}

impl typed_pipeline_layout_seal::Seal for () {}
impl TypedPipelineLayout for () {
    const BIND_GROUP_LAYOUTS: &'static [&'static [Option<BindGroupLayoutEntry>]] = &[];
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

mod bind_group_layouts_seal {
    pub trait Seal {}
}

pub trait BindGroupLayouts: bind_group_layouts_seal::Seal {
    type PipelineLayout: TypedPipelineLayout;

    type Encodings: Iterator<Item = BindGroupLayoutEncoding>;

    fn encodings(&self) -> Self::Encodings;
}

impl bind_group_layouts_seal::Seal for () {}
impl BindGroupLayouts for () {
    type PipelineLayout = ();
    type Encodings = iter::Empty<BindGroupLayoutEncoding>;

    fn encodings(&self) -> Self::Encodings {
        iter::empty()
    }
}

macro_rules! impl_bind_group_layouts {
    ($n:literal, $($B:ident),*) => {
        #[allow(unused_parens)]
        impl<$($B),*> bind_group_layouts_seal::Seal for ($(&'_ BindGroupLayout<$B>),*) where $($B: TypedBindGroupLayout),* {}

        #[allow(unused_parens)]
        impl<$($B),*> BindGroupLayouts for ($(&'_ BindGroupLayout<$B>),*) where $($B: TypedBindGroupLayout),* {
            type PipelineLayout = ($($B,)*);

            type Encodings = <[BindGroupLayoutEncoding; $n] as IntoIterator>::IntoIter;

            fn encodings(&self) -> Self::Encodings {
                #[allow(unused_parens, non_snake_case)]
                let ($($B),*) = self;

                [$($B.to_encoding()),*].into_iter()
            }
        }
    }
}

impl_bind_group_layouts!(1, B0);
impl_bind_group_layouts!(2, B0, B1);
impl_bind_group_layouts!(3, B0, B1, B2);
impl_bind_group_layouts!(4, B0, B1, B2, B3);
