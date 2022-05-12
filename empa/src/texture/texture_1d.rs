use crate::device::Device;
use crate::texture::format::{
    FloatSamplable, ImageCopyFromBufferFormat, ImageCopyTextureFormat, ImageCopyToBufferFormat,
    SignedIntegerSamplable, Storable, SubImageCopyFormat, Texture1DFormat, Texture2DFormat,
    UnfilteredFloatSamplable, UnsignedIntegerSamplable, ViewFormat, ViewFormats,
};
use crate::texture::usage::UsageFlags;
use crate::texture::{
    CopyDst, CopySrc, FormatKind, ImageCopyFromBufferDst, ImageCopyFromTextureDst,
    ImageCopyTexture, ImageCopyToBufferSrc, ImageCopyToTextureSrc, StorageBinding,
    SubImageCopyFromBufferDst, SubImageCopyFromTextureDst, SubImageCopyToBufferSrc,
    SubImageCopyToTextureSrc, TextureBinding, TextureDestroyer,
};
use std::marker;
use std::sync::Arc;
use web_sys::{
    GpuExtent3dDict, GpuTexture, GpuTextureAspect, GpuTextureDescriptor, GpuTextureDimension,
    GpuTextureFormat, GpuTextureView, GpuTextureViewDescriptor, GpuTextureViewDimension,
};

pub struct Texture1D<F, Usage, ViewFormats = F> {
    inner: Arc<TextureDestroyer>,
    format: FormatKind<F>,
    size: u32,
    _usage: marker::PhantomData<Usage>,
    _view_formats: marker::PhantomData<ViewFormats>,
}

impl<F, U, V> Texture1D<F, U, V> {
    fn as_web_sys(&self) -> &GpuTexture {
        &self.inner.texture
    }
}

impl<F, U, V> Texture1D<F, U, V>
where
    F: Texture1DFormat,
    U: UsageFlags,
    V: ViewFormats<F>,
{
    pub(crate) fn new(device: &Device, size: u32) -> Self {
        assert!(size > 0, "size must be greater than `0`");

        let extent = GpuExtent3dDict::new(size);
        let mut desc =
            GpuTextureDescriptor::new(F::FORMAT_ID.to_web_sys(), &extent.into(), U::BITS);

        desc.dimension(GpuTextureDimension::N1d);

        let inner = device.inner.create_texture(&desc);

        Texture1D {
            inner: Arc::new(TextureDestroyer::new(inner)),
            format: FormatKind::Typed(Default::default()),
            size,
            _usage: Default::default(),
            _view_formats: Default::default(),
        }
    }

    fn view_internal(&self, format: GpuTextureFormat) -> GpuTextureView {
        let mut desc = GpuTextureViewDescriptor::new();

        desc.dimension(GpuTextureViewDimension::N1d);
        desc.format(format);

        self.as_web_sys().create_view_with_descriptor(&desc)
    }

    pub fn sampled_float<ViewedFormat>(&self) -> Sampled1DFloat
    where
        ViewedFormat: ViewFormat<V> + FloatSamplable,
        U: TextureBinding,
    {
        Sampled1DFloat {
            inner: self.view_internal(ViewedFormat::FORMAT_ID.to_web_sys()),
            texture_destroyer: self.inner.clone(),
        }
    }

    pub fn sampled_unfiltered_float<ViewedFormat>(&self) -> Sampled1DUnfilteredFloat
    where
        ViewedFormat: ViewFormat<V> + UnfilteredFloatSamplable,
        U: TextureBinding,
    {
        Sampled1DUnfilteredFloat {
            inner: self.view_internal(ViewedFormat::FORMAT_ID.to_web_sys()),
            texture_destroyer: self.inner.clone(),
        }
    }

    pub fn sampled_signed_integer<ViewedFormat>(&self) -> Sampled1DSignedInteger
    where
        ViewedFormat: ViewFormat<V> + SignedIntegerSamplable,
        U: TextureBinding,
    {
        Sampled1DSignedInteger {
            inner: self.view_internal(ViewedFormat::FORMAT_ID.to_web_sys()),
            texture_destroyer: self.inner.clone(),
        }
    }

    pub fn sampled_unsigned_integer<ViewedFormat>(&self) -> Sampled1DUnsignedInteger
    where
        ViewedFormat: ViewFormat<V> + UnsignedIntegerSamplable,
        U: TextureBinding,
    {
        Sampled1DUnsignedInteger {
            inner: self.view_internal(ViewedFormat::FORMAT_ID.to_web_sys()),
            texture_destroyer: self.inner.clone(),
        }
    }

    pub fn storage<ViewedFormat>(&self) -> Storage1D<ViewedFormat>
    where
        ViewedFormat: ViewFormat<V> + Storable,
        U: StorageBinding,
    {
        Storage1D {
            inner: self.view_internal(ViewedFormat::FORMAT_ID.to_web_sys()),
            texture_destroyer: self.inner.clone(),
            _marker: Default::default(),
        }
    }

    fn image_copy_internal(
        &self,
        origin: u32,
        bytes_per_block: u32,
        block_size: [u32; 2],
    ) -> ImageCopyTexture<F> {
        assert!(origin < self.size, "origin out of bounds");

        ImageCopyTexture {
            texture: self.inner.clone(),
            aspect: GpuTextureAspect::All,
            mipmap_level: 0,
            width: self.size,
            height: 1,
            depth_or_layers: 1,
            bytes_per_block,
            block_size,
            origin_x: origin,
            origin_y: 0,
            origin_z: 0,
            _marker: Default::default(),
        }
    }

    pub fn image_copy_to_buffer_src(&self) -> ImageCopyToBufferSrc<F>
    where
        F: ImageCopyToBufferFormat,
        U: CopySrc,
    {
        ImageCopyToBufferSrc {
            inner: self.image_copy_internal(0, F::BYTES_PER_BLOCK, F::BLOCK_SIZE),
        }
    }

    pub fn image_copy_from_buffer_dst(&self) -> ImageCopyFromBufferDst<F>
    where
        F: ImageCopyFromBufferFormat,
        U: CopyDst,
    {
        ImageCopyFromBufferDst {
            inner: self.image_copy_internal(0, F::BYTES_PER_BLOCK, F::BLOCK_SIZE),
        }
    }

    pub fn image_copy_to_texture_src(&self) -> ImageCopyToTextureSrc<F>
    where
        F: ImageCopyTextureFormat,
        U: CopySrc,
    {
        ImageCopyToTextureSrc {
            inner: self.image_copy_internal(0, 0, F::BLOCK_SIZE),
        }
    }

    pub fn image_copy_from_texture_dst(&self) -> ImageCopyFromTextureDst<F>
    where
        F: ImageCopyTextureFormat,
        U: CopyDst,
    {
        ImageCopyFromTextureDst {
            inner: self.image_copy_internal(0, 0, F::BLOCK_SIZE),
        }
    }

    pub fn sub_image_copy_to_buffer_src(&self, origin: u32) -> SubImageCopyToBufferSrc<F>
    where
        F: ImageCopyToBufferFormat + SubImageCopyFormat,
        U: CopySrc,
    {
        SubImageCopyToBufferSrc {
            inner: self.image_copy_internal(origin, F::BYTES_PER_BLOCK, F::BLOCK_SIZE),
        }
    }

    pub fn sub_image_copy_from_buffer_dst(&self, origin: u32) -> SubImageCopyFromBufferDst<F>
    where
        F: ImageCopyFromBufferFormat + SubImageCopyFormat,
        U: CopyDst,
    {
        SubImageCopyFromBufferDst {
            inner: self.image_copy_internal(origin, F::BYTES_PER_BLOCK, F::BLOCK_SIZE),
        }
    }

    pub fn sub_image_copy_to_texture_src(&self, origin: u32) -> SubImageCopyToTextureSrc<F>
    where
        F: ImageCopyTextureFormat + SubImageCopyFormat,
        U: CopySrc,
    {
        SubImageCopyToTextureSrc {
            inner: self.image_copy_internal(origin, 0, F::BLOCK_SIZE),
        }
    }

    pub fn sub_image_copy_from_texture_dst(&self, origin: u32) -> SubImageCopyFromTextureDst<F>
    where
        F: ImageCopyTextureFormat + SubImageCopyFormat,
        U: CopyDst,
    {
        SubImageCopyFromTextureDst {
            inner: self.image_copy_internal(origin, 0, F::BLOCK_SIZE),
        }
    }
}

/// View on a 1D texture that can be bound to a pipeline as a float sampled texture resource.
pub struct Sampled1DFloat {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
}

/// View on a 1D texture that can be bound to a pipeline as a unfiltered float sampled texture
/// resource.
pub struct Sampled1DUnfilteredFloat {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
}

/// View on a 1D texture that can be bound to a pipeline as a signed integer sampled texture
/// resource.
pub struct Sampled1DSignedInteger {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
}

/// View on a 1D texture that can be bound to a pipeline as a unsigned integer sampled texture
/// resource.
pub struct Sampled1DUnsignedInteger {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
}

/// View on a 1D texture that can be bound to a pipeline as a texture storage resource.
pub struct Storage1D<F> {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
    _marker: marker::PhantomData<*const F>,
}
