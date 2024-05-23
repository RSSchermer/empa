use std::marker;

use crate::device::Device;
use crate::driver::{Device as _, Driver, Dvr, PipelineLayoutDescriptor};
use crate::resource_binding::{
    BindGroupLayout, BindGroupLayoutEncoding, BindGroupLayoutEntry, TypedBindGroupLayout,
};

pub struct PipelineLayout<T> {
    pub(crate) handle: <Dvr as Driver>::PipelineLayoutHandle,
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
        let bind_group_layouts = bind_group_layouts.encodings().into_iter().map(|l| l.handle);

        let handle = device
            .handle
            .create_pipeline_layout(PipelineLayoutDescriptor { bind_group_layouts });

        PipelineLayout {
            handle,
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

    type Encodings<'a>: IntoIterator<Item = BindGroupLayoutEncoding<'a>>
    where
        Self: 'a;

    fn encodings<'a>(&'a self) -> Self::Encodings<'a>;
}

impl bind_group_layouts_seal::Seal for () {}
impl BindGroupLayouts for () {
    type PipelineLayout = ();
    type Encodings<'a> = [BindGroupLayoutEncoding<'a>; 0];

    fn encodings<'a>(&'a self) -> Self::Encodings<'a> {
        []
    }
}

macro_rules! impl_bind_group_layouts {
    ($n:literal, $($B:ident),*) => {
        #[allow(unused_parens)]
        impl<$($B),*> bind_group_layouts_seal::Seal for ($(&'_ BindGroupLayout<$B>),*) where $($B: TypedBindGroupLayout),* {}

        #[allow(unused_parens)]
        impl<$($B),*> BindGroupLayouts for ($(&'_ BindGroupLayout<$B>),*) where $($B: TypedBindGroupLayout),* {
            type PipelineLayout = ($($B,)*);

            type Encodings<'a> = [BindGroupLayoutEncoding<'a>; $n] where Self: 'a;

            fn encodings<'a>(&'a self) -> Self::Encodings<'a> {
                #[allow(unused_parens, non_snake_case)]
                let ($($B),*) = self;

                [$($B.to_encoding()),*]
            }
        }
    }
}

impl_bind_group_layouts!(1, B0);
impl_bind_group_layouts!(2, B0, B1);
impl_bind_group_layouts!(3, B0, B1, B2);
impl_bind_group_layouts!(4, B0, B1, B2, B3);
