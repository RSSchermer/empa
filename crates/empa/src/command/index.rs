use std::ops::Range;

use crate::buffer;
use crate::buffer::Buffer;
use crate::driver::{Driver, Dvr};
use crate::render_pipeline::{IndexData, IndexFormat};

pub struct IndexBufferEncoding {
    pub(crate) buffer: <Dvr as Driver>::BufferHandle,
    pub(crate) id: usize,
    pub(crate) format: IndexFormat,
    pub(crate) range: Range<usize>,
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
            buffer: self.internal.handle.clone(),
            id: self.id(),
            format: I::FORMAT,
            range: 0..self.size_in_bytes(),
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
        let start = self.offset_in_bytes();
        let end = start + self.size_in_bytes();

        IndexBufferEncoding {
            buffer: self.buffer.handle.clone(),
            id: self.id(),
            format: I::FORMAT,
            range: start..end,
        }
    }
}
