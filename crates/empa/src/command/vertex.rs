use std::ops::Range;

use crate::buffer;
use crate::buffer::Buffer;
use crate::driver::{Driver, Dvr};
use crate::render_pipeline::{TypedVertexLayout, Vertex};

pub struct VertexBufferEncoding {
    pub(crate) buffer: <Dvr as Driver>::BufferHandle,
    pub(crate) id: usize,
    pub(crate) range: Range<usize>,
}

mod vertex_buffer_seal {
    pub trait Seal {}
}

pub trait VertexBuffer: vertex_buffer_seal::Seal {
    type Vertex: Vertex;

    fn to_encoding(&self) -> VertexBufferEncoding;
}

impl<'a, V, U> vertex_buffer_seal::Seal for &'a Buffer<[V], U>
where
    V: Vertex,
    U: buffer::Vertex,
{
}
impl<'a, V, U> VertexBuffer for &'a Buffer<[V], U>
where
    V: Vertex,
    U: buffer::Vertex,
{
    type Vertex = V;

    fn to_encoding(&self) -> VertexBufferEncoding {
        let start = 0;
        let end = self.size_in_bytes();

        VertexBufferEncoding {
            buffer: self.internal.handle.clone(),
            id: self.id(),
            range: start..end,
        }
    }
}

impl<'a, V, U> vertex_buffer_seal::Seal for buffer::View<'a, [V], U>
where
    V: Vertex,
    U: buffer::Vertex,
{
}
impl<'a, V, U> VertexBuffer for buffer::View<'a, [V], U>
where
    V: Vertex,
    U: buffer::Vertex,
{
    type Vertex = V;

    fn to_encoding(&self) -> VertexBufferEncoding {
        let start = self.offset_in_bytes();
        let end = start + self.size_in_bytes();

        VertexBufferEncoding {
            buffer: self.buffer.handle.clone(),
            id: self.id(),
            range: start..end,
        }
    }
}

mod vertex_buffers_seal {
    pub trait Seal {}
}

pub trait VertexBuffers: vertex_buffers_seal::Seal {
    type Layout: TypedVertexLayout;

    type Encodings: AsRef<[VertexBufferEncoding]>;

    fn encodings(&self) -> Self::Encodings;
}

impl vertex_buffers_seal::Seal for () {}
impl VertexBuffers for () {
    type Layout = ();

    type Encodings = [VertexBufferEncoding; 0];

    fn encodings(&self) -> Self::Encodings {
        []
    }
}

macro_rules! impl_vertex_buffers {
    ($n:literal, $($B:ident),*) => {
        #[allow(unused_parens)]
        impl<$($B),*> vertex_buffers_seal::Seal for ($($B),*) where $($B: VertexBuffer),* {}

        #[allow(unused_parens)]
        impl<$($B),*> VertexBuffers for ($($B),*) where $($B: VertexBuffer),* {
            type Layout = ($($B::Vertex),*);

            type Encodings = [VertexBufferEncoding; $n];

            fn encodings(&self) -> Self::Encodings {
                #[allow(non_snake_case)]
                let ($($B),*) = self;

                [$($B.to_encoding()),*]
            }
        }
    }
}

impl_vertex_buffers!(1, V0);
impl_vertex_buffers!(2, V0, V1);
impl_vertex_buffers!(3, V0, V1, V2);
impl_vertex_buffers!(4, V0, V1, V2, V3);
impl_vertex_buffers!(5, V0, V1, V2, V3, V4);
impl_vertex_buffers!(6, V0, V1, V2, V3, V4, V5);
impl_vertex_buffers!(7, V0, V1, V2, V3, V4, V5, V6);
impl_vertex_buffers!(8, V0, V1, V2, V3, V4, V5, V6, V7);
