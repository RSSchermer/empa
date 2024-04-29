use std::sync::Arc;

use web_sys::GpuIndexFormat;

use crate::buffer;
use crate::buffer::{Buffer, BufferHandle};
use crate::render_pipeline::IndexData;

pub struct IndexBufferEncoding {
    pub(crate) buffer: Arc<BufferHandle>,
    pub(crate) id: usize,
    pub(crate) format: GpuIndexFormat,
    pub(crate) offset: u32,
    pub(crate) size: u32,
}

mod index_buffer_seal {
    pub trait Seal {}
}

pub trait IndexBuffer: index_buffer_seal::Seal {
    type IndexData: IndexData;

    fn to_encoding(&self) -> IndexBufferEncoding;
}

impl<'a, I, U> index_buffer_seal::Seal for &'a Buffer<[I], U>
where
    I: IndexData,
    U: buffer::Index,
{
}
impl<'a, I, U> IndexBuffer for &'a Buffer<[I], U>
where
    I: IndexData,
    U: buffer::Index,
{
    type IndexData = I;

    fn to_encoding(&self) -> IndexBufferEncoding {
        IndexBufferEncoding {
            buffer: self.internal.inner.clone(),
            id: self.id(),
            format: I::FORMAT_ID.inner,
            offset: 0,
            size: self.size_in_bytes() as u32,
        }
    }
}

impl<'a, I, U> index_buffer_seal::Seal for buffer::View<'a, [I], U>
where
    I: IndexData,
    U: buffer::Index,
{
}
impl<'a, I, U> IndexBuffer for buffer::View<'a, [I], U>
where
    I: IndexData,
    U: buffer::Index,
{
    type IndexData = I;

    fn to_encoding(&self) -> IndexBufferEncoding {
        IndexBufferEncoding {
            buffer: self.buffer.inner.clone(),
            id: self.id(),
            format: I::FORMAT_ID.inner,
            offset: 0,
            size: self.size_in_bytes() as u32,
        }
    }
}
