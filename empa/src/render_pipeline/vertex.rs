use web_sys::{GpuVertexAttribute, GpuVertexStepMode};

use crate::render_pipeline::vertex_attribute::VertexAttributeFormatId;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VertexInputRate {
    PerVertex,
    PerInstance,
}

impl VertexInputRate {
    pub(crate) fn to_web_sys(&self) -> GpuVertexStepMode {
        match self {
            VertexInputRate::PerVertex => GpuVertexStepMode::Vertex,
            VertexInputRate::PerInstance => GpuVertexStepMode::Instance,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VertexAttributeDescriptor {
    pub format: VertexAttributeFormatId,
    pub offset: u32,
    pub shader_location: u32,
}

impl VertexAttributeDescriptor {
    pub(crate) fn to_web_sys(&self) -> GpuVertexAttribute {
        GpuVertexAttribute::new(
            self.format.to_web_sys(),
            self.offset as f64,
            self.shader_location,
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VertexDescriptor<'a> {
    pub array_stride: u32,
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
        #[allow(unused_parens)]
        impl<$($vertex),*> typed_vertex_layout_seal::Seal for ($($vertex),*) where $($vertex: Vertex),* {}

        #[allow(unused_parens)]
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
