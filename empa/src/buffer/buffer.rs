use std::borrow::Borrow;
use std::fmt::Display;
use std::future::Future;
use std::mem::MaybeUninit;
use std::ops::{
    Deref, DerefMut, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive, Rem,
};
use std::sync::{Arc, Mutex};
use std::{error, marker, mem, slice};

use atomic_counter::AtomicCounter;
use futures::TryFutureExt;
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{GpuBuffer, GpuBufferDescriptor, GpuImageCopyBuffer};

use crate::abi;
use crate::buffer::{
    CopyDst, CopySrc, MapRead, MapWrite, StorageBinding, UniformBinding, ValidUsageFlags,
};
use crate::device::{Device, ID_GEN};
use crate::texture::{ImageCopySize3D, ImageDataByteLayout, ImageDataLayout};

/// Signals that an error occurred when trying to map a buffer.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MapError;

impl Display for MapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error occurred when trying to map a buffer")
    }
}

impl error::Error for MapError {}

pub trait AsBuffer<T>
where
    T: ?Sized,
{
    fn as_buffer<Usage>(
        &self,
        device: &Device,
        mapped_at_creation: bool,
        usage: Usage,
    ) -> Buffer<T, Usage>
    where
        Usage: ValidUsageFlags;
}

impl<T, D> AsBuffer<T> for D
where
    D: Borrow<T>,
    T: Copy + 'static,
{
    fn as_buffer<Usage>(
        &self,
        device: &Device,
        mapped_at_creation: bool,
        _usage: Usage,
    ) -> Buffer<T, Usage>
    where
        Usage: ValidUsageFlags,
    {
        let id = ID_GEN.get();
        let size_in_bytes = mem::size_of::<T>();

        let mut desc = GpuBufferDescriptor::new(size_in_bytes as f64, Usage::BITS);

        desc.mapped_at_creation(true);

        let buffer = device.inner.create_buffer(&desc);
        let view = Uint8Array::new(
            buffer
                .get_mapped_range_with_u32_and_u32(0, size_in_bytes as u32)
                .as_ref(),
        );

        let data_bytes = unsafe { value_to_bytes(self.borrow()) };

        view.copy_from(data_bytes);

        let mut map_context = MapContext::new();

        if !mapped_at_creation {
            buffer.unmap();
        } else {
            map_context.initial_range = 0..size_in_bytes as u32;
        }

        Buffer {
            inner: Arc::new(BufferDestroyer::new(buffer)),
            id,
            len: 1,
            map_context: Mutex::new(map_context),
            _marker: Default::default(),
            _usages: Default::default(),
        }
    }
}

impl<T, D> AsBuffer<[T]> for D
where
    D: Borrow<[T]>,
    T: Copy + 'static,
{
    fn as_buffer<Usage>(
        &self,
        device: &Device,
        mapped_at_creation: bool,
        _usage: Usage,
    ) -> Buffer<[T], Usage>
    where
        Usage: ValidUsageFlags,
    {
        let id = ID_GEN.get();
        let data = self.borrow();
        let slice_len = data.len();
        let size_in_bytes = mem::size_of::<T>() * slice_len;

        let mut desc = GpuBufferDescriptor::new(size_in_bytes as f64, Usage::BITS);

        desc.mapped_at_creation(true);

        let buffer = device.inner.create_buffer(&desc);

        let view = Uint8Array::new(
            buffer
                .get_mapped_range_with_u32_and_u32(0, size_in_bytes as u32)
                .as_ref(),
        );

        let data_bytes = unsafe { slice_to_bytes(self.borrow()) };

        view.copy_from(data_bytes);

        let mut map_context = MapContext::new();

        if !mapped_at_creation {
            buffer.unmap();
        } else {
            map_context.initial_range = 0..size_in_bytes as u32;
        }

        Buffer {
            inner: Arc::new(BufferDestroyer::new(buffer)),
            id,
            len: slice_len,
            map_context: Mutex::new(map_context),
            _marker: Default::default(),
            _usages: Default::default(),
        }
    }
}

pub(crate) struct BufferDestroyer {
    pub(crate) buffer: GpuBuffer,
}

impl BufferDestroyer {
    fn new(buffer: GpuBuffer) -> Self {
        BufferDestroyer { buffer }
    }
}

impl Drop for BufferDestroyer {
    fn drop(&mut self) {
        self.buffer.destroy();
    }
}

pub struct Buffer<T, U>
where
    T: ?Sized,
{
    pub(crate) inner: Arc<BufferDestroyer>,
    id: usize,
    len: usize,
    map_context: Mutex<MapContext>,
    _marker: marker::PhantomData<T>,
    _usages: marker::PhantomData<U>,
}

impl<T, U> Buffer<T, U>
where
    T: ?Sized,
{
    pub fn unmap(&self) {
        self.map_context.lock().unwrap().reset();
        self.as_web_sys().unmap();
    }

    pub(crate) fn id(&self) -> usize {
        self.id
    }

    pub(crate) fn as_web_sys(&self) -> &GpuBuffer {
        &self.inner.buffer
    }

    fn map_async_internal(
        &self,
        mode: u32,
        start: u32,
        size: u32,
    ) -> impl Future<Output = Result<(), MapError>> {
        let end = start + size;

        let mut mc = self.map_context.lock().unwrap();

        assert_eq!(
            mc.initial_range,
            0..0,
            "Buffer {:?} is already mapped",
            self.as_web_sys()
        );

        mc.initial_range = start..end;

        let promise = self
            .as_web_sys()
            .map_async_with_u32_and_u32(mode, start, size);

        JsFuture::from(promise).map_ok(|_| ()).map_err(|_| MapError)
    }
}

impl<T, U> Buffer<MaybeUninit<T>, U>
where
    U: ValidUsageFlags,
{
    pub(crate) fn create_uninit(device: &Device, mapped_at_creation: bool, _usage: U) -> Self {
        let id = ID_GEN.get();
        let size_in_bytes = mem::size_of::<T>();
        let mut desc = GpuBufferDescriptor::new(size_in_bytes as f64, U::BITS);

        desc.mapped_at_creation(mapped_at_creation);

        let buffer = device.inner.create_buffer(&desc);

        let mut map_context = MapContext::new();

        if mapped_at_creation {
            map_context.initial_range = 0..size_in_bytes as u32;
        }

        Buffer {
            inner: Arc::new(BufferDestroyer::new(buffer)),
            id,
            len: 1,
            map_context: Mutex::new(map_context),
            _marker: Default::default(),
            _usages: Default::default(),
        }
    }
}

impl<T, U> Buffer<MaybeUninit<T>, U> {
    /// Converts to `Buffer<T>`.
    ///
    /// # Safety
    ///
    /// Any tasks that read from the buffer after `assume_init` was called, must only be executed
    /// after the buffer was initialized.
    pub unsafe fn assume_init(self) -> Buffer<T, U> {
        Buffer {
            inner: self.inner,
            id: self.id,
            len: 1,
            map_context: self.map_context,
            _marker: Default::default(),
            _usages: Default::default(),
        }
    }
}

impl<T, U> Buffer<[MaybeUninit<T>], U>
where
    U: ValidUsageFlags,
{
    pub(crate) fn create_slice_uninit(
        device: &Device,
        len: usize,
        mapped_at_creation: bool,
        _usage: U,
    ) -> Self {
        let id = ID_GEN.get();
        let size_in_bytes = mem::size_of::<T>() * len;
        let mut desc = GpuBufferDescriptor::new(size_in_bytes as f64, U::BITS);

        desc.mapped_at_creation(mapped_at_creation);

        let buffer = device.inner.create_buffer(&desc);

        let mut map_context = MapContext::new();

        if mapped_at_creation {
            map_context.initial_range = 0..size_in_bytes as u32;
        }

        Buffer {
            inner: Arc::new(BufferDestroyer::new(buffer)),
            id,
            len,
            map_context: Mutex::new(map_context),
            _marker: Default::default(),
            _usages: Default::default(),
        }
    }
}

impl<T, U> Buffer<[MaybeUninit<T>], U> {
    /// Converts to `Buffer<T>`.
    ///
    /// # Safety
    ///
    /// Any tasks that read from the buffer after `assume_init` was called, must only be executed
    /// after the buffer was initialized.
    pub unsafe fn assume_init(self) -> Buffer<[T], U> {
        Buffer {
            inner: self.inner,
            id: self.id,
            len: self.len,
            map_context: self.map_context,
            _marker: Default::default(),
            _usages: Default::default(),
        }
    }
}

impl<T, U> Buffer<T, U> {
    pub fn map_read(&self) -> impl Future<Output = Result<(), MapError>>
    where
        U: MapRead,
    {
        View::from(self).map_read()
    }

    pub fn map_write(&self) -> impl Future<Output = Result<(), MapError>>
    where
        U: MapWrite,
    {
        View::from(self).map_write()
    }

    pub fn mapped(&self) -> Mapped<T>
    where
        U: MapRead,
    {
        View::from(self).mapped()
    }

    pub fn mapped_mut(&self) -> MappedMut<T>
    where
        U: MapWrite,
    {
        View::from(self).mapped_mut()
    }

    pub fn view(&self) -> View<T, U> {
        self.into()
    }

    pub fn uniform(&self) -> Uniform<T>
    where
        T: abi::Sized,
        U: UniformBinding,
    {
        Uniform {
            inner: self.inner.clone(),
            offset: 0,
            size: self.size_in_bytes(),
            _marker: Default::default(),
        }
    }

    pub fn storage(&self) -> Storage<T>
    where
        T: abi::Unsized,
        U: StorageBinding,
    {
        Storage {
            inner: self.inner.clone(),
            offset: 0,
            size: self.size_in_bytes(),
            _marker: Default::default(),
        }
    }

    pub fn read_only_storage(&self) -> ReadOnlyStorage<T>
    where
        T: abi::Unsized,
        U: StorageBinding,
    {
        ReadOnlyStorage {
            inner: self.inner.clone(),
            offset: 0,
            size: self.size_in_bytes(),
            _marker: Default::default(),
        }
    }

    pub(crate) fn size_in_bytes(&self) -> usize {
        mem::size_of::<T>()
    }
}

impl<T, U> Buffer<[T], U> {
    /// Returns the number of elements contained in this [Buffer].
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns a [View] on an element or a slice of the elements this [Buffer], depending on the
    /// type of `index`.
    ///
    /// - If given a position, returns a view on the element at that position or `None` if out of
    ///   bounds.
    /// - If given a range, returns a view on the slice of elements corresponding to that range, or
    ///   `None` if out of bounds.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let buffer = device.create_buffer([1.0, 2.0, 3.0, 4.0], buffer_descriptor);
    ///
    /// buffer.get(1); // Some View<f32> containing `2.0`
    /// buffer.get(1..3); // Some View<[f32]> containing `[2.0, 3.0]`
    /// buffer.get(..2); // Some View<[f32]> containing `[1.0 2.0]`
    /// buffer.get(4); // None (index out of bounds)
    /// ```
    pub fn get<I>(&self, index: I) -> Option<View<I::Output, U>>
    where
        I: SliceIndex<T>,
    {
        index.get(self.into())
    }

    /// Returns a [View] on an element or a slice of the elements this [Buffer], depending on the
    /// type of `index`, without doing bounds checking.
    ///
    /// - If given a position, returns a view on the element at that position, without doing bounds
    ///   checking.
    /// - If given a range, returns a view on the slice of elements corresponding to that range,
    ///   without doing bounds checking.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let buffer = device.create_buffer([1.0, 2.0, 3.0, 4.0], buffer_descriptor);
    ///
    /// unsafe { buffer.get_unchecked(1) }; // BufferView<f32> containing `2.0`
    /// ```
    ///
    /// # Unsafe
    ///
    /// Only safe if `index` is in bounds. See [get] for a safe alternative.
    pub unsafe fn get_unchecked<I>(&self, index: I) -> View<I::Output, U>
    where
        I: SliceIndex<T>,
    {
        index.get_unchecked(self.into())
    }

    pub fn map_read(&self) -> impl Future<Output = Result<(), MapError>>
    where
        U: MapRead,
    {
        View::from(self).map_read()
    }

    pub fn map_write(&self) -> impl Future<Output = Result<(), MapError>>
    where
        U: MapWrite,
    {
        View::from(self).map_write()
    }

    pub fn mapped(&self) -> MappedSlice<T>
    where
        U: MapRead,
    {
        View::from(self).mapped()
    }

    pub fn mapped_mut(&self) -> MappedSliceMut<T>
    where
        U: MapWrite,
    {
        View::from(self).mapped_mut()
    }

    pub fn view(&self) -> View<[T], U> {
        self.into()
    }

    pub fn storage(&self) -> Storage<[T]>
    where
        T: abi::Sized,
        U: StorageBinding,
    {
        Storage {
            inner: self.inner.clone(),
            offset: 0,
            size: self.size_in_bytes(),
            _marker: Default::default(),
        }
    }

    pub fn read_only_storage(&self) -> ReadOnlyStorage<[T]>
    where
        T: abi::Sized,
        U: StorageBinding,
    {
        ReadOnlyStorage {
            inner: self.inner.clone(),
            offset: 0,
            size: self.size_in_bytes(),
            _marker: Default::default(),
        }
    }

    pub fn image_copy_src(&self, layout: ImageDataLayout) -> ImageCopySrc<T>
    where
        U: CopySrc,
    {
        let ImageDataLayout {
            blocks_per_row,
            rows_per_image,
        } = layout;

        let bytes_per_block = mem::size_of::<T>() as u32;
        let bytes_per_row = blocks_per_row * bytes_per_block;

        assert!(bytes_per_row.rem(256) == 0, "bytes per block row `block_size * block_per_row` (`{} * {}`) must be a multiple of `256`", bytes_per_block, bytes_per_row);

        ImageCopySrc {
            inner: ImageCopyBuffer {
                buffer: self.inner.clone(),
                offset: 0,
                size: self.size_in_bytes(),
                bytes_per_block,
                blocks_per_row,
                rows_per_image,
            },
            _marker: Default::default(),
        }
    }

    pub fn image_copy_dst(&self, layout: ImageDataLayout) -> ImageCopyDst<T>
    where
        U: CopyDst,
    {
        let ImageDataLayout {
            blocks_per_row,
            rows_per_image,
        } = layout;

        let bytes_per_block = mem::size_of::<T>() as u32;
        let bytes_per_row = blocks_per_row * bytes_per_block;

        assert!(bytes_per_row.rem(256) == 0, "bytes per block row `block_size * block_per_row` (`{} * {}`) must be a multiple of `256`", bytes_per_block, bytes_per_row);

        ImageCopyDst {
            inner: ImageCopyBuffer {
                buffer: self.inner.clone(),
                offset: 0,
                size: self.size_in_bytes(),
                bytes_per_block,
                blocks_per_row,
                rows_per_image,
            },
            _marker: Default::default(),
        }
    }

    pub(crate) fn size_in_bytes(&self) -> usize {
        mem::size_of::<T>() * self.len
    }
}

impl<U> Buffer<[u8], U> {
    pub fn image_copy_src_raw(&self, layout: ImageDataByteLayout) -> ImageCopySrcRaw
    where
        U: CopySrc,
    {
        let ImageDataByteLayout {
            bytes_per_block,
            blocks_per_row,
            rows_per_image,
        } = layout;

        let bytes_per_row = blocks_per_row * bytes_per_block;

        assert!(bytes_per_row.rem(256) == 0, "bytes per block row `block_size * block_per_row` (`{} * {}`) must be a multiple of `256`", bytes_per_block, bytes_per_row);

        ImageCopySrcRaw {
            inner: ImageCopyBuffer {
                buffer: self.inner.clone(),
                offset: 0,
                size: self.size_in_bytes(),
                bytes_per_block,
                blocks_per_row,
                rows_per_image,
            },
        }
    }

    pub fn image_copy_dst_raw(&self, layout: ImageDataByteLayout) -> ImageCopyDstRaw
    where
        U: CopyDst,
    {
        let ImageDataByteLayout {
            bytes_per_block,
            blocks_per_row,
            rows_per_image,
        } = layout;

        let bytes_per_row = blocks_per_row * bytes_per_block;

        assert!(bytes_per_row.rem(256) == 0, "bytes per block row `block_size * block_per_row` (`{} * {}`) must be a multiple of `256`", bytes_per_block, bytes_per_row);

        ImageCopyDstRaw {
            inner: ImageCopyBuffer {
                buffer: self.inner.clone(),
                offset: 0,
                size: self.size_in_bytes(),
                bytes_per_block,
                blocks_per_row,
                rows_per_image,
            },
        }
    }
}

// TODO: it's a bit unfortunate that wgpu opted to name it's mapped memory types `BufferView` and
// `BufferViewMut`, may want to consider different naming here. On the other hand... on an intuitive
// level I quite like the names as they currently are

/// View on a [Buffer] region.
pub struct View<'a, T, U>
where
    T: ?Sized,
{
    pub(crate) buffer: &'a Buffer<T, U>,
    // Note: using separate offset/len here rather than `Range` because `Range` is not `Copy`
    offset: usize,
    len: usize,
}

impl<'a, T, U> View<'a, T, U>
where
    T: ?Sized,
{
    pub(crate) fn id(&self) -> usize {
        self.buffer.id
    }

    pub(crate) fn as_web_sys(self) -> &'a GpuBuffer {
        self.buffer.as_web_sys()
    }
}

impl<'a, T, U> View<'a, T, U> {
    fn map_internal(&self, mode: u32) -> impl Future<Output = Result<(), MapError>> {
        let size_in_bytes = mem::size_of::<T>() as u32;
        let start = size_in_bytes * self.offset as u32;

        self.buffer.map_async_internal(mode, start, size_in_bytes)
    }

    pub fn map_read(&self) -> impl Future<Output = Result<(), MapError>>
    where
        U: MapRead,
    {
        self.map_internal(1)
    }

    pub fn map_write(&self) -> impl Future<Output = Result<(), MapError>>
    where
        U: MapWrite,
    {
        self.map_internal(2)
    }

    pub fn mapped(self) -> Mapped<'a, T>
    where
        U: MapRead,
    {
        let size_in_bytes = mem::size_of::<T>() as u32;
        let start = size_in_bytes * self.offset as u32;
        let end = start + size_in_bytes;

        self.buffer.map_context.lock().unwrap().add(start..end);

        let mapped_bytes = Uint8Array::new(
            &self
                .as_web_sys()
                .get_mapped_range_with_u32_and_u32(start, size_in_bytes),
        );
        let mut buffered = MaybeUninit::<T>::uninit();
        let ptr = buffered.as_mut_ptr() as *mut ();

        copy_buffer_to_memory(
            &mapped_bytes,
            0,
            size_in_bytes,
            &wasm_bindgen::memory(),
            ptr,
        );

        let buffered = unsafe { buffered.assume_init() };

        Mapped {
            buffered,
            range: start..end,
            map_context: &self.buffer.map_context,
        }
    }

    pub fn mapped_mut(self) -> MappedMut<'a, T>
    where
        U: MapWrite,
    {
        let size_in_bytes = mem::size_of::<T>() as u32;
        let start = size_in_bytes * self.offset as u32;
        let end = start + size_in_bytes;

        self.buffer.map_context.lock().unwrap().add(start..end);

        let mapped_bytes = Uint8Array::new(
            &self
                .as_web_sys()
                .get_mapped_range_with_u32_and_u32(start, size_in_bytes),
        );
        let mut buffered = MaybeUninit::<T>::uninit();
        let ptr = buffered.as_mut_ptr() as *mut ();

        copy_buffer_to_memory(
            &mapped_bytes,
            0,
            size_in_bytes,
            &wasm_bindgen::memory(),
            ptr,
        );

        let buffered = unsafe { buffered.assume_init() };

        MappedMut {
            buffered,
            mapped_bytes,
            range: start..end,
            map_context: &self.buffer.map_context,
        }
    }

    pub fn uniform(&self) -> Uniform<T>
    where
        T: abi::Sized,
        U: UniformBinding,
    {
        Uniform {
            inner: self.buffer.inner.clone(),
            offset: self.offset_in_bytes(),
            size: self.size_in_bytes(),
            _marker: Default::default(),
        }
    }

    pub fn storage(&self) -> Storage<T>
    where
        T: abi::Unsized,
        U: StorageBinding,
    {
        Storage {
            inner: self.buffer.inner.clone(),
            offset: self.offset_in_bytes(),
            size: self.size_in_bytes(),
            _marker: Default::default(),
        }
    }

    pub fn read_only_storage(&self) -> ReadOnlyStorage<T>
    where
        T: abi::Unsized,
        U: StorageBinding,
    {
        ReadOnlyStorage {
            inner: self.buffer.inner.clone(),
            offset: self.offset_in_bytes(),
            size: self.size_in_bytes(),
            _marker: Default::default(),
        }
    }

    pub(crate) fn offset_in_bytes(&self) -> usize {
        mem::size_of::<T>() * self.offset
    }

    pub(crate) fn size_in_bytes(&self) -> usize {
        mem::size_of::<T>()
    }
}

impl<'a, T, U> View<'a, [T], U> {
    /// Returns the number of elements in this view.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns a [View] on an element or a sub-slice of the elements this [View], depending on the
    /// type of `index`.
    ///
    /// - If given a position, returns a view on the element at that position or `None` if out of
    ///   bounds.
    /// - If given a range, returns a view on the sub-slice of elements corresponding to that range,
    ///   or `None` if out of bounds.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let buffer: Buffer<[f32]> = device.create_buffer([1.0, 2.0, 3.0, 4.0], descriptor);
    /// let view = View::from(&buffer);
    ///
    /// view.get(1); // Some View<f32> containing `2.0`
    /// view.get(1..3); // Some View<[f32]> containing `[2.0, 3.0]`
    /// view.get(..2); // Some View<[f32]> containing `[1.0 2.0]`
    /// view.get(4); // None (index out of bounds)
    /// # }
    /// ```
    pub fn get<I>(self, index: I) -> Option<View<'a, I::Output, U>>
    where
        I: SliceIndex<T>,
    {
        index.get(self)
    }

    /// Returns a [View] on an element or a sub-slice of the elements this [View], depending on the
    /// type of `index`, without doing bounds checking.
    ///
    /// - If given a position, returns a view on the element at that position, without doing bounds
    ///   checking.
    /// - If given a range, returns a view on the slice of elements corresponding to that range,
    ///   without doing bounds checking.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let buffer: Buffer<[f32]> = device.create_buffer([1.0, 2.0, 3.0, 4.0], descriptor);
    /// let view = View::from(&buffer);
    ///
    /// unsafe { view.get_unchecked(1) }; // View<f32> containing `2.0`
    /// # }
    /// ```
    ///
    /// # Unsafe
    ///
    /// Only safe if `index` is in bounds. See [get] for a safe alternative.
    pub unsafe fn get_unchecked<I>(self, index: I) -> View<'a, I::Output, U>
    where
        I: SliceIndex<T>,
    {
        index.get_unchecked(self)
    }

    fn map_internal(&self, mode: u32) -> impl Future<Output = Result<(), MapError>> {
        let size_in_bytes = (mem::size_of::<T>() * self.len) as u32;
        let start = size_in_bytes * self.offset as u32;

        self.buffer.map_async_internal(mode, start, size_in_bytes)
    }

    pub fn map_read(&self) -> impl Future<Output = Result<(), MapError>>
    where
        U: MapRead,
    {
        self.map_internal(1)
    }

    pub fn map_write(&self) -> impl Future<Output = Result<(), MapError>>
    where
        U: MapWrite,
    {
        self.map_internal(2)
    }

    pub fn mapped(self) -> MappedSlice<'a, T>
    where
        U: MapRead,
    {
        let size_in_bytes = (mem::size_of::<T>() * self.len) as u32;
        let start = (mem::size_of::<T>() * self.offset) as u32;
        let end = start + size_in_bytes;

        self.buffer.map_context.lock().unwrap().add(start..end);

        let mapped_bytes = Uint8Array::new(
            self.as_web_sys()
                .get_mapped_range_with_u32_and_u32(start, size_in_bytes)
                .as_ref(),
        );
        let mut buffered = Box::<[T]>::new_uninit_slice(self.len);
        let ptr = buffered.as_mut_ptr() as *mut ();

        copy_buffer_to_memory(
            &mapped_bytes,
            0,
            size_in_bytes,
            &wasm_bindgen::memory(),
            ptr,
        );

        let buffered = unsafe { buffered.assume_init() };

        MappedSlice {
            buffered,
            range: start..end,
            map_context: &self.buffer.map_context,
        }
    }

    pub fn mapped_mut(self) -> MappedSliceMut<'a, T>
    where
        U: MapWrite,
    {
        let size_in_bytes = (mem::size_of::<T>() * self.len) as u32;
        let start = (mem::size_of::<T>() * self.offset) as u32;
        let end = start + size_in_bytes;

        self.buffer.map_context.lock().unwrap().add(start..end);

        let mapped_bytes = Uint8Array::new(
            &self
                .as_web_sys()
                .get_mapped_range_with_u32_and_u32(start, size_in_bytes),
        );
        let mut buffered = Box::<[T]>::new_uninit_slice(self.len);
        let ptr = buffered.as_mut_ptr() as *mut ();

        copy_buffer_to_memory(
            &mapped_bytes,
            0,
            size_in_bytes,
            &wasm_bindgen::memory(),
            ptr,
        );

        let buffered = unsafe { buffered.assume_init() };

        MappedSliceMut {
            buffered,
            mapped_bytes,
            range: start..end,
            map_context: &self.buffer.map_context,
        }
    }

    pub fn storage(&self) -> Storage<[T]>
    where
        T: abi::Sized,
        U: StorageBinding,
    {
        Storage {
            inner: self.buffer.inner.clone(),
            offset: self.offset_in_bytes(),
            size: self.size_in_bytes(),
            _marker: Default::default(),
        }
    }

    pub fn read_only_storage(&self) -> ReadOnlyStorage<[T]>
    where
        T: abi::Sized,
        U: StorageBinding,
    {
        ReadOnlyStorage {
            inner: self.buffer.inner.clone(),
            offset: self.offset_in_bytes(),
            size: self.size_in_bytes(),
            _marker: Default::default(),
        }
    }

    pub fn image_copy_src(&self, layout: ImageDataLayout) -> ImageCopySrc<T>
    where
        U: CopySrc,
    {
        let ImageDataLayout {
            blocks_per_row,
            rows_per_image,
        } = layout;

        let bytes_per_block = mem::size_of::<T>() as u32;
        let bytes_per_row = blocks_per_row * bytes_per_block;

        assert!(bytes_per_row.rem(256) == 0, "bytes per block row `block_size * block_per_row` (`{} * {}`) must be a multiple of `256`", bytes_per_block, bytes_per_row);

        ImageCopySrc {
            inner: ImageCopyBuffer {
                buffer: self.buffer.inner.clone(),
                offset: self.offset_in_bytes(),
                size: self.size_in_bytes(),
                bytes_per_block,
                blocks_per_row,
                rows_per_image,
            },
            _marker: Default::default(),
        }
    }

    pub fn image_copy_dst(&self, layout: ImageDataLayout) -> ImageCopyDst<T>
    where
        U: CopyDst,
    {
        let ImageDataLayout {
            blocks_per_row,
            rows_per_image,
        } = layout;

        let bytes_per_block = mem::size_of::<T>() as u32;
        let bytes_per_row = blocks_per_row * bytes_per_block;

        assert!(bytes_per_row.rem(256) == 0, "bytes per block row `block_size * block_per_row` (`{} * {}`) must be a multiple of `256`", bytes_per_block, bytes_per_row);

        ImageCopyDst {
            inner: ImageCopyBuffer {
                buffer: self.buffer.inner.clone(),
                offset: self.offset_in_bytes(),
                size: self.size_in_bytes(),
                bytes_per_block,
                blocks_per_row,
                rows_per_image,
            },
            _marker: Default::default(),
        }
    }

    pub(crate) fn offset_in_bytes(&self) -> usize {
        mem::size_of::<T>() * self.offset
    }

    pub(crate) fn size_in_bytes(&self) -> usize {
        mem::size_of::<T>() * self.len
    }
}

impl<'a, U> View<'a, [u8], U> {
    pub fn image_copy_src_raw(&self, layout: ImageDataByteLayout) -> ImageCopySrcRaw
    where
        U: CopySrc,
    {
        let ImageDataByteLayout {
            bytes_per_block,
            blocks_per_row,
            rows_per_image,
        } = layout;

        let bytes_per_row = blocks_per_row * bytes_per_block;

        assert!(bytes_per_row.rem(256) == 0, "bytes per block row `block_size * block_per_row` (`{} * {}`) must be a multiple of `256`", bytes_per_block, bytes_per_row);

        ImageCopySrcRaw {
            inner: ImageCopyBuffer {
                buffer: self.buffer.inner.clone(),
                offset: self.offset_in_bytes(),
                size: self.size_in_bytes(),
                bytes_per_block,
                blocks_per_row,
                rows_per_image,
            },
        }
    }

    pub fn image_copy_dst_raw(&self, layout: ImageDataByteLayout) -> ImageCopyDstRaw
    where
        U: CopyDst,
    {
        let ImageDataByteLayout {
            bytes_per_block,
            blocks_per_row,
            rows_per_image,
        } = layout;

        let bytes_per_row = blocks_per_row * bytes_per_block;

        assert!(bytes_per_row.rem(256) == 0, "bytes per block row `block_size * block_per_row` (`{} * {}`) must be a multiple of `256`", bytes_per_block, bytes_per_row);

        ImageCopyDstRaw {
            inner: ImageCopyBuffer {
                buffer: self.buffer.inner.clone(),
                offset: self.offset_in_bytes(),
                size: self.size_in_bytes(),
                bytes_per_block,
                blocks_per_row,
                rows_per_image,
            },
        }
    }
}

impl<'a, T, U> Clone for View<'a, T, U>
where
    T: ?Sized,
{
    fn clone(&self) -> Self {
        View {
            buffer: self.buffer,
            offset: self.offset,
            len: self.len,
        }
    }
}

impl<'a, T, U> Copy for View<'a, T, U> where T: ?Sized {}

impl<'a, T, U> From<&'a Buffer<T, U>> for View<'a, T, U>
where
    T: ?Sized,
{
    fn from(buffer: &'a Buffer<T, U>) -> Self {
        View {
            buffer,
            offset: 0,
            len: buffer.len,
        }
    }
}

// Note: we don't wrapped the buffered values in `ManuallyDrop` here, because in the current
// implementation, all data that can be (safely) put in a Buffer (including all GPU generated data)
// is `Copy`, hence there should be no drop-related concerns (`Copy` and `Drop` are mutually
// exclusive; a type cannot be both).

pub struct Mapped<'a, T> {
    buffered: T,
    range: Range<u32>,
    map_context: &'a Mutex<MapContext>,
}

impl<'a, T> Deref for Mapped<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.buffered
    }
}

impl<'a, T> Drop for Mapped<'a, T> {
    fn drop(&mut self) {
        self.map_context.lock().unwrap().remove(self.range.clone());
    }
}

pub struct MappedSlice<'a, T> {
    buffered: Box<[T]>,
    range: Range<u32>,
    map_context: &'a Mutex<MapContext>,
}

impl<'a, T> Deref for MappedSlice<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.buffered
    }
}

impl<'a, T> Drop for MappedSlice<'a, T> {
    fn drop(&mut self) {
        self.map_context.lock().unwrap().remove(self.range.clone());
    }
}

pub struct MappedMut<'a, T> {
    buffered: T,
    mapped_bytes: Uint8Array,
    range: Range<u32>,
    map_context: &'a Mutex<MapContext>,
}

impl<'a, T> Deref for MappedMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.buffered
    }
}

impl<'a, T> DerefMut for MappedMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffered
    }
}

impl<'a, T> Drop for MappedMut<'a, T> {
    fn drop(&mut self) {
        let data_bytes = unsafe { value_to_bytes(&self.buffered) };

        self.mapped_bytes.copy_from(data_bytes);

        self.map_context.lock().unwrap().remove(self.range.clone());
    }
}

pub struct MappedSliceMut<'a, T> {
    buffered: Box<[T]>,
    mapped_bytes: Uint8Array,
    range: Range<u32>,
    map_context: &'a Mutex<MapContext>,
}

impl<'a, T> Deref for MappedSliceMut<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.buffered
    }
}
impl<'a, T> DerefMut for MappedSliceMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffered
    }
}

impl<'a, T> Drop for MappedSliceMut<'a, T> {
    fn drop(&mut self) {
        let data_bytes = unsafe { slice_to_bytes(&self.buffered) };

        self.mapped_bytes.copy_from(data_bytes);

        self.map_context.lock().unwrap().remove(self.range.clone());
    }
}

mod slice_index_seal {
    use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

    pub trait Seal {}

    impl Seal for usize {}
    impl Seal for RangeFull {}
    impl Seal for Range<usize> {}
    impl Seal for RangeInclusive<usize> {}
    impl Seal for RangeFrom<usize> {}
    impl Seal for RangeTo<usize> {}
    impl Seal for RangeToInclusive<usize> {}
}

/// A helper trait type for indexing operations on a [Buffer] that contains a slice.
pub trait SliceIndex<T>: slice_index_seal::Seal + Sized {
    /// The output type returned by the indexing operations.
    type Output: ?Sized;

    /// Returns a view on the output for this operation if in bounds, or `None` otherwise.
    fn get<U>(self, view: View<[T], U>) -> Option<View<Self::Output, U>>;

    /// Returns a view on the output for this operation, without performing any bounds checking.
    unsafe fn get_unchecked<U>(self, view: View<[T], U>) -> View<Self::Output, U>;
}

impl<T> SliceIndex<T> for usize {
    type Output = T;

    fn get<U>(self, view: View<[T], U>) -> Option<View<Self::Output, U>> {
        if self < view.len() {
            Some(View {
                buffer: unsafe { mem::transmute(view.buffer) },
                offset: view.offset + self,
                len: 1,
            })
        } else {
            None
        }
    }

    unsafe fn get_unchecked<U>(self, view: View<[T], U>) -> View<Self::Output, U> {
        View {
            buffer: mem::transmute(view.buffer),
            offset: view.offset + self,
            len: 1,
        }
    }
}

impl<T> SliceIndex<T> for RangeFull {
    type Output = [T];

    fn get<U>(self, view: View<[T], U>) -> Option<View<Self::Output, U>> {
        Some(view)
    }

    unsafe fn get_unchecked<U>(self, view: View<[T], U>) -> View<Self::Output, U> {
        view
    }
}

impl<T> SliceIndex<T> for Range<usize> {
    type Output = [T];

    fn get<U>(self, view: View<[T], U>) -> Option<View<Self::Output, U>> {
        let Range { start, end } = self;

        if start > end || end > view.len() {
            None
        } else {
            Some(View {
                buffer: view.buffer,
                offset: view.offset + start,
                len: end - start,
            })
        }
    }

    unsafe fn get_unchecked<U>(self, view: View<[T], U>) -> View<Self::Output, U> {
        let Range { start, end } = self;

        View {
            buffer: view.buffer,
            offset: view.offset + start,
            len: end - start,
        }
    }
}

impl<T> SliceIndex<T> for RangeInclusive<usize> {
    type Output = [T];

    fn get<U>(self, view: View<[T], U>) -> Option<View<Self::Output, U>> {
        if *self.end() == usize::MAX {
            None
        } else {
            view.get(*self.start()..self.end() + 1)
        }
    }

    unsafe fn get_unchecked<U>(self, view: View<[T], U>) -> View<Self::Output, U> {
        view.get_unchecked(*self.start()..self.end() + 1)
    }
}

impl<T> SliceIndex<T> for RangeFrom<usize> {
    type Output = [T];

    fn get<U>(self, view: View<[T], U>) -> Option<View<Self::Output, U>> {
        view.get(self.start..view.len())
    }

    unsafe fn get_unchecked<U>(self, view: View<[T], U>) -> View<Self::Output, U> {
        view.get_unchecked(self.start..view.len())
    }
}

impl<T> SliceIndex<T> for RangeTo<usize> {
    type Output = [T];

    fn get<U>(self, view: View<[T], U>) -> Option<View<Self::Output, U>> {
        view.get(0..self.end)
    }

    unsafe fn get_unchecked<U>(self, view: View<[T], U>) -> View<Self::Output, U> {
        view.get_unchecked(0..self.end)
    }
}

impl<T> SliceIndex<T> for RangeToInclusive<usize> {
    type Output = [T];

    fn get<U>(self, view: View<[T], U>) -> Option<View<Self::Output, U>> {
        view.get(0..=self.end)
    }

    unsafe fn get_unchecked<U>(self, view: View<[T], U>) -> View<Self::Output, U> {
        view.get_unchecked(0..=self.end)
    }
}

unsafe fn value_to_bytes<T>(value: &T) -> &[u8] {
    let size_in_bytes = mem::size_of::<T>();

    slice::from_raw_parts(value as *const T as *const u8, size_in_bytes)
}

unsafe fn slice_to_bytes<T>(slice: &[T]) -> &[u8] {
    let size_in_bytes = mem::size_of::<T>() * slice.len();

    slice::from_raw_parts(slice as *const [T] as *const u8, size_in_bytes)
}

pub struct Uniform<T>
where
    T: ?Sized,
{
    pub(crate) inner: Arc<BufferDestroyer>,
    pub(crate) offset: usize,
    pub(crate) size: usize,
    _marker: marker::PhantomData<*const T>,
}

pub struct Storage<T>
where
    T: ?Sized,
{
    pub(crate) inner: Arc<BufferDestroyer>,
    pub(crate) offset: usize,
    pub(crate) size: usize,
    _marker: marker::PhantomData<*const T>,
}

pub struct ReadOnlyStorage<T>
where
    T: ?Sized,
{
    pub(crate) inner: Arc<BufferDestroyer>,
    pub(crate) offset: usize,
    pub(crate) size: usize,
    _marker: marker::PhantomData<*const T>,
}

pub(crate) struct ImageCopyBuffer {
    pub(crate) buffer: Arc<BufferDestroyer>,
    pub(crate) offset: usize,
    pub(crate) size: usize,
    pub(crate) bytes_per_block: u32,
    pub(crate) blocks_per_row: u32,
    pub(crate) rows_per_image: u32,
}

impl ImageCopyBuffer {
    pub(crate) fn validate_with_size_and_block_size(
        &self,
        size: ImageCopySize3D,
        block_size: [u32; 2],
    ) {
        let ImageCopySize3D {
            width,
            height,
            depth_or_layers,
        } = size;

        let [block_width, block_height] = block_size;

        let width_in_blocks = width / block_width;

        assert!(
            self.blocks_per_row >= width_in_blocks,
            "blocks per row must be at least the copy width in blocks (`{}`)",
            width_in_blocks
        );

        let height_in_blocks = height / block_height;

        assert!(
            self.rows_per_image >= height_in_blocks,
            "rows per image must be at least the copy height in blocks (`{}`)",
            height_in_blocks
        );

        let min_size = self.blocks_per_row * self.rows_per_image * depth_or_layers;

        assert!(
            self.size >= min_size as usize,
            "buffer view must contains enough elements for the copy size (`{}` blocks)",
            min_size
        );
    }

    pub(crate) fn to_web_sys(&self) -> GpuImageCopyBuffer {
        let mut copy_buffer = GpuImageCopyBuffer::new(&self.buffer.buffer);

        copy_buffer.offset(self.offset as f64);
        copy_buffer.bytes_per_row(self.bytes_per_block * self.blocks_per_row);
        copy_buffer.rows_per_image(self.rows_per_image);

        copy_buffer
    }
}

pub struct ImageCopySrc<T> {
    pub(crate) inner: ImageCopyBuffer,
    _marker: marker::PhantomData<*const T>,
}

pub struct ImageCopySrcRaw {
    pub(crate) inner: ImageCopyBuffer,
}

pub struct ImageCopyDst<T> {
    pub(crate) inner: ImageCopyBuffer,
    _marker: marker::PhantomData<*const T>,
}

pub struct ImageCopyDstRaw {
    pub(crate) inner: ImageCopyBuffer,
}

// Struct modified from https://github.com/gfx-rs/wgpu

#[derive(Debug)]
struct MapContext {
    initial_range: Range<u32>,
    sub_ranges: Vec<Range<u32>>,
}

impl MapContext {
    fn new() -> Self {
        Self {
            initial_range: 0..0,
            sub_ranges: Vec::new(),
        }
    }

    fn reset(&mut self) {
        self.initial_range = 0..0;

        assert!(
            self.sub_ranges.is_empty(),
            "You cannot unmap a buffer that still has accessible mapped views"
        );
    }

    fn add(&mut self, range: Range<u32>) {
        assert!(self.initial_range.start <= range.start && range.end <= self.initial_range.end);

        for sub in self.sub_ranges.iter() {
            assert!(
                range.end <= sub.start || range.start >= sub.end,
                "Intersecting map range with {:?}",
                sub
            );
        }

        self.sub_ranges.push(range);
    }

    fn remove(&mut self, range: Range<u32>) {
        let index = self
            .sub_ranges
            .iter()
            .position(|r| *r == range.clone())
            .expect("unable to remove range from map context");

        self.sub_ranges.swap_remove(index);
    }
}

#[wasm_bindgen(module = "/src/js_support.js")]
extern "C" {
    #[wasm_bindgen(js_name = __empa_js_copy_buffer_to_memory)]
    fn copy_buffer_to_memory(
        buffer: &Uint8Array,
        offset: u32,
        size: u32,
        wasm_memory: &JsValue,
        pointer: *mut (),
    );
}
