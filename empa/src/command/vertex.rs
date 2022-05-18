use std::sync::Arc;

use crate::buffer;
use crate::buffer::{Buffer, BufferDestroyer};
use crate::render_pipeline::{TypedVertexLayout, Vertex};

pub struct VertexBufferEncoding {
    pub(crate) buffer: Arc<BufferDestroyer>,
    pub(crate) id: usize,
    pub(crate) offset: u32,
    pub(crate) size: u32,
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
        VertexBufferEncoding {
            buffer: self.inner.clone(),
            id: self.id(),
            offset: 0,
            size: self.size_in_bytes() as u32,
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
        VertexBufferEncoding {
            buffer: self.buffer.inner.clone(),
            id: self.id(),
            offset: 0,
            size: self.size_in_bytes() as u32,
        }
    }
}

mod vertex_buffers_seal {
    pub trait Seal {}
}

pub trait VertexBuffers: vertex_buffers_seal::Seal {
    type Layout: TypedVertexLayout;

    type Encodings: Iterator<Item = VertexBufferEncoding>;

    fn encodings(&self) -> Self::Encodings;
}

macro_rules! impl_vertex_buffers {
    ($n:literal, $($B:ident),*) => {
        #[allow(unused_parens)]
        impl<$($B),*> vertex_buffers_seal::Seal for ($($B),*) where $($B: VertexBuffer),* {}

        #[allow(unused_parens)]
        impl<$($B),*> VertexBuffers for ($($B),*) where $($B: VertexBuffer),* {
            type Layout = ($($B::Vertex),*);

            type Encodings = <[VertexBufferEncoding; $n] as IntoIterator>::IntoIter;

            fn encodings(&self) -> Self::Encodings {
                #[allow(non_snake_case)]
                let ($($B),*) = self;

                [$($B.to_encoding()),*].into_iter()
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
