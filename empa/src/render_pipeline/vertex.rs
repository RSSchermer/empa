use crate::buffer;
use crate::buffer::Buffer;
use crate::render_pipeline::vertex_attribute::VertexAttributeFormatId;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VertexInputRate {
    PerVertex,
    PerInstance,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VertexAttributeDescriptor {
    pub format: VertexAttributeFormatId,
    pub offset: u32,
    pub shader_location: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VertexDescriptor<'a> {
    pub attribute_descriptors: &'a [VertexAttributeDescriptor],
    pub input_rate: VertexInputRate,
}

pub unsafe trait Vertex: Sized {
    const DESCRIPTOR: VertexDescriptor<'static>;
}

mod typed_vertex_layout_seal {
    pub trait Seal {}
}

pub trait TypedVertexLayout: typed_vertex_layout_seal::Seal {
    const LAYOUT: &'static [VertexDescriptor<'static>];
}

macro_rules! impl_typed_vertex_layout {
    ($($vertex:ident),*) => {
        impl<$($vertex),*> typed_vertex_layout_seal::Seal for ($($vertex),*) where $($vertex: Vertex),* {}
        impl<$($vertex),*> TypedVertexLayout for ($($vertex),*) where $($vertex: Vertex),* {
            const LAYOUT: &'static [VertexDescriptor<'static>] = &[
                $($vertex::DESCRIPTOR),*
            ];
        }
    }
}

impl_typed_vertex_layout!(V0);
impl_typed_vertex_layout!(V0, V1);
impl_typed_vertex_layout!(V0, V1, V2);
impl_typed_vertex_layout!(V0, V1, V2, V3);
impl_typed_vertex_layout!(V0, V1, V2, V3, V4);
impl_typed_vertex_layout!(V0, V1, V2, V3, V4, V5);
impl_typed_vertex_layout!(V0, V1, V2, V3, V4, V5, V6);
impl_typed_vertex_layout!(V0, V1, V2, V3, V4, V5, V6, V7);
