use std::sync::Arc;

use web_sys::GpuBindGroup;

use crate::resource_binding::{
    BindGroup, BindGroupResource, TypedBindGroupLayout, TypedPipelineLayout,
};
use std::iter;

pub struct BindGroupEncoding {
    pub(crate) bind_group: GpuBindGroup,
    pub(crate) id: usize,
    pub(crate) _resource_handles: Arc<Vec<BindGroupResource>>,
}

mod bind_groups_seal {
    pub trait Seal {}
}

pub trait BindGroups: bind_groups_seal::Seal {
    type Layout: TypedPipelineLayout;

    type Encodings: Iterator<Item = BindGroupEncoding>;

    fn encodings(&self) -> Self::Encodings;
}

impl bind_groups_seal::Seal for () {}
impl BindGroups for () {
    type Layout = ();
    type Encodings = iter::Empty<BindGroupEncoding>;

    fn encodings(&self) -> Self::Encodings {
        iter::empty()
    }
}

macro_rules! impl_bind_groups {
    ($n:literal, $($B:ident),*) => {
        #[allow(unused_parens)]
        impl<'a, $($B),*> bind_groups_seal::Seal for ($(&'a BindGroup<$B>),*) where $($B: TypedBindGroupLayout),* {}

        #[allow(unused_parens)]
        impl<'a, $($B),*> BindGroups for ($(&'a BindGroup<$B>),*) where $($B: TypedBindGroupLayout),* {
            type Layout = ($($B,)*);

            type Encodings = <[BindGroupEncoding; $n] as IntoIterator>::IntoIter;

            fn encodings(&self) -> Self::Encodings {
                #[allow(non_snake_case)]
                let ($($B),*) = self;

                [$($B.to_encoding()),*].into_iter()
            }
        }
    }
}

impl_bind_groups!(1, B0);
impl_bind_groups!(2, B0, B1);
impl_bind_groups!(3, B0, B1, B2);
impl_bind_groups!(4, B0, B1, B2, B3);
