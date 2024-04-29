use flagset::{flags, FlagSet};
use std::iter::Map;
use std::ops::Range;
use std::future::Future;
use std::fmt;
use std::borrow::{Borrow, BorrowMut};

pub trait Driver: Sized {
    type DeviceHandle: DeviceHandle<Self>;
    type BufferHandle: BufferHandle<Self>;
    type TextureHandle: BufferHandle<Self>;
    type CommandBufferHandle: CommandBufferHandle<Self>;
    type QueueHandle: QueueHandle<Self>;
}

pub trait DeviceHandle<T>: Sized where T: Driver {
    fn create_buffer(&self, descriptor: &BufferDescriptor) -> T::BufferHandle;
}

flags! {
    pub enum BufferUsage: u32 {
        MapRead      = 0x0001,
        MapWrite     = 0x0002,
        CopySrc      = 0x0004,
        CopyDst      = 0x0008,
        Index        = 0x0010,
        Vertex       = 0x0020,
        Uniform      = 0x0040,
        Storage      = 0x0080,
        Indirect     = 0x0100,
        QueryResolve = 0x0200,
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BufferDescriptor {
    pub size: u64,
    pub usage_flags: FlagSet<BufferUsage>,
    pub mapped_at_creation: bool
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MapMode {
    Read,
    Write
}

/// Signals that an error occurred when trying to map a buffer.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MapError;

impl fmt::Display for MapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error occurred when trying to map a buffer")
    }
}

pub trait BufferHandle<T>: Sized where T: Driver {
    type Map: Future<Output=Result<(), MapError>>;

    type Mapped<'a>: AsRef<[u8]> where Self: 'a;

    type MappedMut<'a>: AsMut<[u8]> where Self: 'a;

    fn map(&self, mode: MapMode, range: Range<u64>) -> Self::Map;

    fn mapped<'a>(&'a self, range: Range<u64>) -> Self::Mapped<'a>;

    fn mapped_mut<'a>(&'a self, range: Range<u64>) -> Self::MappedMut<'a>;
}

pub trait TextureHandle<T>: Sized where T: Driver {

}

pub trait CommandBufferHandle<T> where T: Driver {}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TextureAspect {
    All,
    StencilOnly,
    DepthOnly,
}

pub struct ImageCopyTexture<'a, T> where T: Driver {
    pub texture_handle: &'a T::TextureHandle,
    pub mip_level: u32,
    pub origin: (u32, u32, u32),
    pub aspect: TextureAspect,
}

pub struct WriteBufferOperation<'a, T> where T: Driver {
    pub buffer_handle: &'a T::BufferHandle,
    pub offset: u64,
    pub data: &'a [u8],
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ImageDataLayout {
    pub offset: u64,
    pub bytes_per_row: u32,
    pub rows_per_image: u32,
}

pub struct WriteTextureOperation<'a, T> where T: Driver {
    pub image_copy_texture: ImageCopyTexture<'a, T>,
    pub image_data_layout: ImageDataLayout,
    pub extent: (u32, u32, u32),
    pub data: &'a [u8]
}

pub trait QueueHandle<T>: Sized where T: Driver {
    fn submit(&self, command_buffer: &T::CommandBufferHandle);

    fn write_buffer(&self, operation: WriteBufferOperation<T>);

    fn write_texture(&self, operation: WriteTextureOperation<T>);
}

