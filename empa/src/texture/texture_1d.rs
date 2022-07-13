use std::marker;
use std::sync::Arc;

use staticvec::StaticVec;
use web_sys::{
    GpuExtent3dDict, GpuTexture, GpuTextureAspect, GpuTextureDescriptor, GpuTextureDimension,
    GpuTextureFormat, GpuTextureView, GpuTextureViewDescriptor, GpuTextureViewDimension,
};

use crate::device::Device;
use crate::texture::format::{
    FloatSamplable, ImageCopyFromBufferFormat, ImageCopyTextureFormat, ImageCopyToBufferFormat,
    SignedIntegerSamplable, Storable, SubImageCopyFormat, Texture1DFormat, TextureFormatId,
    UnfilteredFloatSamplable, UnsignedIntegerSamplable, ViewFormat, ViewFormats,
};
use crate::texture::{
    CopyDst, CopySrc, FormatKind, ImageCopyDst, ImageCopyFromTextureDst, ImageCopySrc,
    ImageCopyTexture, ImageCopyToTextureSrc, StorageBinding, SubImageCopyDst,
    SubImageCopyFromTextureDst, SubImageCopySrc, SubImageCopyToTextureSrc, TextureBinding,
    TextureDestroyer, UnsupportedViewFormat, UsageFlags,
};

pub struct Texture1DDescriptor<F, U, V>
where
    F: Texture1DFormat,
    U: UsageFlags,
    V: ViewFormats<F>,
{
    pub format: F,
    pub usage: U,
    pub view_formats: V,
    pub size: u32,
}

pub struct Texture1D<F, Usage> {
    inner: Arc<TextureDestroyer>,
    format: FormatKind<F>,
    size: u32,
    view_formats: StaticVec<TextureFormatId, 8>,
    usage: Usage,
}

impl<F, U> Texture1D<F, U>
where
    F: Texture1DFormat,
    U: UsageFlags,
{
    pub(crate) fn new<V: ViewFormats<F>>(
        device: &Device,
        descriptor: &Texture1DDescriptor<F, U, V>,
    ) -> Self {
        let Texture1DDescriptor {
            view_formats,
            size,
            usage,
            ..
        } = descriptor;

        assert!(*size > 0, "size must be greater than `0`");

        let extent = GpuExtent3dDict::new(*size);
        let mut desc =
            GpuTextureDescriptor::new(F::FORMAT_ID.to_web_sys(), &extent.into(), U::BITS);

        desc.dimension(GpuTextureDimension::N1d);

        let inner = device.inner.create_texture(&desc);
        let view_formats = view_formats.formats().collect();

        Texture1D {
            inner: Arc::new(TextureDestroyer::new(inner, false)),
            format: FormatKind::Typed(Default::default()),
            size: *size,
            view_formats,
            usage: *usage,
        }
    }
}

impl<F, U> Texture1D<F, U>
where
    U: UsageFlags,
{
    pub fn usage(&self) -> U {
        self.usage
    }
}

impl<F, U> Texture1D<F, U> {
    fn as_web_sys(&self) -> &GpuTexture {
        &self.inner.texture
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    fn view_internal(&self, format: GpuTextureFormat) -> GpuTextureView {
        let mut desc = GpuTextureViewDescriptor::new();

        desc.dimension(GpuTextureViewDimension::N1d);
        desc.format(format);

        self.as_web_sys().create_view_with_descriptor(&desc)
    }

    pub fn sampled_float(&self) -> Sampled1DFloat
    where
        F: FloatSamplable,
        U: TextureBinding,
    {
        Sampled1DFloat {
            inner: self.view_internal(F::FORMAT_ID.to_web_sys()),
            texture_destroyer: self.inner.clone(),
        }
    }

    pub fn try_as_sampled_float<ViewedFormat>(
        &self,
    ) -> Result<Sampled1DFloat, UnsupportedViewFormat>
    where
        ViewedFormat: ViewFormat<F> + FloatSamplable,
        U: TextureBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(Sampled1DFloat {
                inner: self.view_internal(ViewedFormat::FORMAT_ID.to_web_sys()),
                texture_destroyer: self.inner.clone(),
            })
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    pub fn sampled_unfiltered_float(&self) -> Sampled1DUnfilteredFloat
    where
        F: UnfilteredFloatSamplable,
        U: TextureBinding,
    {
        Sampled1DUnfilteredFloat {
            inner: self.view_internal(F::FORMAT_ID.to_web_sys()),
            texture_destroyer: self.inner.clone(),
        }
    }

    pub fn try_as_sampled_unfiltered_float<ViewedFormat>(
        &self,
    ) -> Result<Sampled1DUnfilteredFloat, UnsupportedViewFormat>
    where
        ViewedFormat: ViewFormat<F> + UnfilteredFloatSamplable,
        U: TextureBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(Sampled1DUnfilteredFloat {
                inner: self.view_internal(ViewedFormat::FORMAT_ID.to_web_sys()),
                texture_destroyer: self.inner.clone(),
            })
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    pub fn sampled_signed_integer(&self) -> Sampled1DSignedInteger
    where
        F: SignedIntegerSamplable,
        U: TextureBinding,
    {
        Sampled1DSignedInteger {
            inner: self.view_internal(F::FORMAT_ID.to_web_sys()),
            texture_destroyer: self.inner.clone(),
        }
    }

    pub fn try_as_sampled_signed_integer<ViewedFormat>(
        &self,
    ) -> Result<Sampled1DSignedInteger, UnsupportedViewFormat>
    where
        ViewedFormat: ViewFormat<F> + SignedIntegerSamplable,
        U: TextureBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(Sampled1DSignedInteger {
                inner: self.view_internal(ViewedFormat::FORMAT_ID.to_web_sys()),
                texture_destroyer: self.inner.clone(),
            })
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    pub fn sampled_unsigned_integer(&self) -> Sampled1DUnsignedInteger
    where
        F: UnsignedIntegerSamplable,
        U: TextureBinding,
    {
        Sampled1DUnsignedInteger {
            inner: self.view_internal(F::FORMAT_ID.to_web_sys()),
            texture_destroyer: self.inner.clone(),
        }
    }

    pub fn try_as_sampled_unsigned_integer<ViewedFormat>(
        &self,
    ) -> Result<Sampled1DUnsignedInteger, UnsupportedViewFormat>
    where
        ViewedFormat: ViewFormat<F> + UnsignedIntegerSamplable,
        U: TextureBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(Sampled1DUnsignedInteger {
                inner: self.view_internal(ViewedFormat::FORMAT_ID.to_web_sys()),
                texture_destroyer: self.inner.clone(),
            })
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    pub fn storage(&self) -> Storage1D<F>
    where
        F: Storable,
        U: StorageBinding,
    {
        Storage1D {
            inner: self.view_internal(F::FORMAT_ID.to_web_sys()),
            texture_destroyer: self.inner.clone(),
            _marker: Default::default(),
        }
    }

    pub fn try_as_storage<ViewedFormat>(
        &self,
    ) -> Result<Storage1D<ViewedFormat>, UnsupportedViewFormat>
    where
        ViewedFormat: ViewFormat<F> + Storable,
        U: StorageBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(Storage1D {
                inner: self.view_internal(ViewedFormat::FORMAT_ID.to_web_sys()),
                texture_destroyer: self.inner.clone(),
                _marker: Default::default(),
            })
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
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

    pub fn image_copy_to_buffer_src(&self) -> ImageCopySrc<F>
    where
        F: ImageCopyToBufferFormat,
        U: CopySrc,
    {
        ImageCopySrc {
            inner: self.image_copy_internal(0, F::BYTES_PER_BLOCK, F::BLOCK_SIZE),
        }
    }

    pub fn image_copy_from_buffer_dst(&self) -> ImageCopyDst<F>
    where
        F: ImageCopyFromBufferFormat,
        U: CopyDst,
    {
        ImageCopyDst {
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

    pub fn sub_image_copy_to_buffer_src(&self, origin: u32) -> SubImageCopySrc<F>
    where
        F: ImageCopyToBufferFormat + SubImageCopyFormat,
        U: CopySrc,
    {
        SubImageCopySrc {
            inner: self.image_copy_internal(origin, F::BYTES_PER_BLOCK, F::BLOCK_SIZE),
        }
    }

    pub fn sub_image_copy_from_buffer_dst(&self, origin: u32) -> SubImageCopyDst<F>
    where
        F: ImageCopyFromBufferFormat + SubImageCopyFormat,
        U: CopyDst,
    {
        SubImageCopyDst {
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
