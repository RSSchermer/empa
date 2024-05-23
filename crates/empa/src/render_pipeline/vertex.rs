use std::borrow::Cow;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(non_camel_case_types)]
pub enum VertexFormat {
    uint8x2,
    uint8x4,
    sint8x2,
    sint8x4,
    unorm8x2,
    unorm8x4,
    snorm8x2,
    snorm8x4,
    uint16x2,
    uint16x4,
    sint16x2,
    sint16x4,
    unorm16x2,
    unorm16x4,
    snorm16x2,
    snorm16x4,
    float16x2,
    float16x4,
    float32,
    float32x2,
    float32x3,
    float32x4,
    uint32,
    uint32x2,
    uint32x3,
    uint32x4,
    sint32,
    sint32x2,
    sint32x3,
    sint32x4,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VertexStepMode {
    Vertex,
    Instance,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VertexAttribute {
    pub format: VertexFormat,
    pub offset: usize,
    pub shader_location: u32,
}

#[derive(Clone)]
pub struct VertexBufferLayout<'a> {
    pub array_stride: usize,
    pub step_mode: VertexStepMode,
    pub attributes: Cow<'a, [VertexAttribute]>,
}

pub unsafe trait Vertex: Sized {
    const LAYOUT: VertexBufferLayout<'static>;
}

mod typed_vertex_layout_seal {
    pub trait Seal {}
}

pub trait TypedVertexLayout: typed_vertex_layout_seal::Seal {
    const LAYOUT: &'static [VertexBufferLayout<'static>];
}

impl typed_vertex_layout_seal::Seal for () {}

impl TypedVertexLayout for () {
    const LAYOUT: &'static [VertexBufferLayout<'static>] = &[];
}

macro_rules! impl_typed_vertex_layout {
    ($($vertex:ident),*) => {
        #[allow(unused_parens)]
        impl<$($vertex),*> typed_vertex_layout_seal::Seal for ($($vertex),*) where $($vertex: Vertex),* {}

        #[allow(unused_parens)]
        impl<$($vertex),*> TypedVertexLayout for ($($vertex),*) where $($vertex: Vertex),* {
            const LAYOUT: &'static [VertexBufferLayout<'static>] = &[
                $($vertex::LAYOUT),*
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
