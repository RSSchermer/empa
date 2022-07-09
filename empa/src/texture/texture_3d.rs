use std::cmp::max;
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
    SignedIntegerSamplable, Storable, SubImageCopyFormat, Texture3DFormat, TextureFormatId,
    UnfilteredFloatSamplable, UnsignedIntegerSamplable, ViewFormat, ViewFormats,
};
use crate::texture::{
    CopyDst, CopySrc, FormatKind, ImageCopyDst, ImageCopyFromTextureDst, ImageCopySrc,
    ImageCopyTexture, ImageCopyToTextureSrc, MipmapLevels, StorageBinding, SubImageCopyDst,
    SubImageCopyFromTextureDst, SubImageCopySrc, SubImageCopyToTextureSrc, TextureBinding,
    TextureDestroyer, UnsupportedViewFormat, UsageFlags,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Texture3DDescriptor<F, U, V>
where
    F: Texture3DFormat,
    U: UsageFlags,
    V: ViewFormats<F>,
{
    pub format: F,
    pub usage: U,
    pub view_formats: V,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub mipmap_levels: MipmapLevels,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct View3DDescriptor {
    pub base_mipmap_level: u8,
    pub mipmap_level_count: Option<u8>,
}

impl Default for View3DDescriptor {
    fn default() -> Self {
        View3DDescriptor {
            base_mipmap_level: 0,
            mipmap_level_count: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SubImageCopy3DDescriptor {
    pub mipmap_level: u8,
    pub origin_x: u32,
    pub origin_y: u32,
    pub origin_z: u32,
}

pub struct Texture3D<F, Usage> {
    inner: Arc<TextureDestroyer>,
    mip_level_count: u8,
    format: FormatKind<F>,
    width: u32,
    height: u32,
    depth: u32,
    view_formats: StaticVec<TextureFormatId, 8>,
    _usage: marker::PhantomData<Usage>,
}

impl<F, U> Texture3D<F, U>
    where
        F: Texture3DFormat,
        U: UsageFlags,
{
    pub(crate) fn new<V: ViewFormats<F>>(
        device: &Device,
        descriptor: &Texture3DDescriptor<F, U, V>,
    ) -> Self {
        let Texture3DDescriptor {
            view_formats,
            width,
            height,
            depth,
            mipmap_levels,
            ..
        } = descriptor;

        assert!(*width > 0, "width must be greater than `0`");
        assert!(*height > 0, "height must be greater than `0`");
        assert!(*depth > 0, "depth must be greater than `0`");

        let mip_level_count = mipmap_levels.to_u32(max(max(*width, *height), *depth));
        let mut size = GpuExtent3dDict::new(*width);

        size.height(*height);
        size.depth_or_array_layers(*depth);

        let mut desc = GpuTextureDescriptor::new(F::FORMAT_ID.to_web_sys(), &size.into(), U::BITS);

        desc.dimension(GpuTextureDimension::N3d);
        desc.mip_level_count(mip_level_count);

        let inner = device.inner.create_texture(&desc);
        let view_formats = view_formats.formats().collect();

        Texture3D {
            inner: Arc::new(TextureDestroyer::new(inner, false)),
            format: FormatKind::Typed(Default::default()),
            width: *width,
            height: *height,
            depth: *depth,
            mip_level_count: mip_level_count as u8,
            view_formats,
            _usage: Default::default(),
        }
    }
}

impl<F, U> Texture3D<F, U> {
    fn as_web_sys(&self) -> &GpuTexture {
        &self.inner.texture
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn depth(&self) -> u32 {
        self.depth
    }


    fn view_internal(
        &self,
        format: GpuTextureFormat,
        descriptor: &View3DDescriptor,
    ) -> GpuTextureView {
        let View3DDescriptor {
            base_mipmap_level,
            mipmap_level_count,
        } = *descriptor;

        assert!(
            base_mipmap_level < self.mip_level_count,
            "`base_mipmap_level` must not exceed the texture's mipmap level count"
        );

        let mipmap_level_count = if let Some(mipmap_level_count) = mipmap_level_count {
            assert!(
                base_mipmap_level + mipmap_level_count < self.mip_level_count,
                "`base_mipmap_level + mip_level_count` must not exceed the texture's mipmap \
                    level count"
            );

            mipmap_level_count
        } else {
            self.mip_level_count - base_mipmap_level
        };

        let mut desc = GpuTextureViewDescriptor::new();

        desc.dimension(GpuTextureViewDimension::N3d);
        desc.format(format);
        desc.base_mip_level(base_mipmap_level as u32);
        desc.mip_level_count(mipmap_level_count as u32);

        self.as_web_sys().create_view_with_descriptor(&desc)
    }

    pub fn sampled_float(&self, descriptor: &View3DDescriptor) -> Sampled3DFloat
        where
            F: FloatSamplable,
            U: TextureBinding,
    {
        Sampled3DFloat {
            inner: self.view_internal(F::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
        }
    }

    pub fn try_as_sampled_float<ViewedFormat>(
        &self,
        descriptor: &View3DDescriptor,
    ) -> Result<Sampled3DFloat, UnsupportedViewFormat>
        where
            ViewedFormat: ViewFormat<F> + FloatSamplable,
            U: TextureBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(Sampled3DFloat {
                inner: self.view_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
                texture_destroyer: self.inner.clone(),
            })
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    pub fn sampled_unfiltered_float(
        &self,
        descriptor: &View3DDescriptor,
    ) -> Sampled3DUnfilteredFloat
        where
            F: UnfilteredFloatSamplable,
            U: TextureBinding,
    {
        Sampled3DUnfilteredFloat {
            inner: self.view_internal(F::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
        }
    }

    pub fn try_as_sampled_unfiltered_float<ViewedFormat>(
        &self,
        descriptor: &View3DDescriptor,
    ) -> Result<Sampled3DUnfilteredFloat, UnsupportedViewFormat>
        where
            ViewedFormat: ViewFormat<F> + UnfilteredFloatSamplable,
            U: TextureBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(Sampled3DUnfilteredFloat {
                inner: self.view_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
                texture_destroyer: self.inner.clone(),
            })
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    pub fn sampled_signed_integer(&self, descriptor: &View3DDescriptor) -> Sampled3DSignedInteger
        where
            F: SignedIntegerSamplable,
            U: TextureBinding,
    {
        Sampled3DSignedInteger {
            inner: self.view_internal(F::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
        }
    }

    pub fn try_as_sampled_signed_integer<ViewedFormat>(
        &self,
        descriptor: &View3DDescriptor,
    ) -> Result<Sampled3DSignedInteger, UnsupportedViewFormat>
        where
            ViewedFormat: ViewFormat<F> + SignedIntegerSamplable,
            U: TextureBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(Sampled3DSignedInteger {
                inner: self.view_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
                texture_destroyer: self.inner.clone(),
            })
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    pub fn sampled_unsigned_integer(
        &self,
        descriptor: &View3DDescriptor,
    ) -> Sampled3DUnsignedInteger
        where
            F: UnsignedIntegerSamplable,
            U: TextureBinding,
    {
        Sampled3DUnsignedInteger {
            inner: self.view_internal(F::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
        }
    }

    pub fn try_as_sampled_unsigned_integer<ViewedFormat>(
        &self,
        descriptor: &View3DDescriptor,
    ) -> Result<Sampled3DUnsignedInteger, UnsupportedViewFormat>
        where
            ViewedFormat: ViewFormat<F> + UnsignedIntegerSamplable,
            U: TextureBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(Sampled3DUnsignedInteger {
                inner: self.view_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
                texture_destroyer: self.inner.clone(),
            })
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    pub fn storage(&self, descriptor: &View3DDescriptor) -> Storage3D<F>
        where
            F: Storable,
            U: StorageBinding,
    {
        Storage3D {
            inner: self.view_internal(F::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
            _marker: Default::default(),
        }
    }

    pub fn try_as_storage<ViewedFormat>(
        &self,
        descriptor: &View3DDescriptor,
    ) -> Result<Storage3D<ViewedFormat>, UnsupportedViewFormat>
        where
            ViewedFormat: ViewFormat<F> + Storable,
            U: StorageBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(Storage3D {
                inner: self.view_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
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
        mipmap_level: u8,
        bytes_per_block: u32,
        block_size: [u32; 2],
    ) -> ImageCopyTexture<F> {
        assert!(
            mipmap_level < self.mip_level_count,
            "mipmap level out of bounds"
        );

        ImageCopyTexture {
            texture: self.inner.clone(),
            aspect: GpuTextureAspect::All,
            mipmap_level,
            width: self.width,
            height: self.height,
            depth_or_layers: self.depth,
            bytes_per_block,
            block_size,
            origin_x: 0,
            origin_y: 0,
            origin_z: 0,
            _marker: Default::default(),
        }
    }

    fn sub_image_copy_internal(
        &self,
        descriptor: SubImageCopy3DDescriptor,
        bytes_per_block: u32,
        block_size: [u32; 2],
    ) -> ImageCopyTexture<F> {
        let SubImageCopy3DDescriptor {
            mipmap_level,
            origin_x,
            origin_y,
            origin_z,
        } = descriptor;

        assert!(
            mipmap_level < self.mip_level_count,
            "mipmap level out of bounds"
        );
        assert!(origin_x < self.width, "`x` origin out of bounds");
        assert!(origin_y < self.height, "`y` origin out of bounds");
        assert!(origin_z < self.depth, "layer origin out of bounds");

        ImageCopyTexture {
            texture: self.inner.clone(),
            aspect: GpuTextureAspect::All,
            mipmap_level,
            width: self.width,
            height: self.height,
            depth_or_layers: self.depth,
            bytes_per_block,
            block_size,
            origin_x,
            origin_y,
            origin_z,
            _marker: Default::default(),
        }
    }

    pub fn image_copy_to_buffer_src(&self, mipmap_level: u8) -> ImageCopySrc<F>
        where
            F: ImageCopyToBufferFormat,
            U: CopySrc,
    {
        ImageCopySrc {
            inner: self.image_copy_internal(mipmap_level, F::BYTES_PER_BLOCK, F::BLOCK_SIZE),
        }
    }

    pub fn image_copy_from_buffer_dst(&self, mipmap_level: u8) -> ImageCopyDst<F>
        where
            F: ImageCopyFromBufferFormat,
            U: CopyDst,
    {
        ImageCopyDst {
            inner: self.image_copy_internal(mipmap_level, F::BYTES_PER_BLOCK, F::BLOCK_SIZE),
        }
    }

    pub fn image_copy_to_texture_src(&self, mipmap_level: u8) -> ImageCopyToTextureSrc<F>
        where
            F: ImageCopyTextureFormat,
            U: CopySrc,
    {
        ImageCopyToTextureSrc {
            inner: self.image_copy_internal(mipmap_level, 0, F::BLOCK_SIZE),
        }
    }

    pub fn image_copy_from_texture_dst(&self, mipmap_level: u8) -> ImageCopyFromTextureDst<F>
        where
            F: ImageCopyTextureFormat,
            U: CopyDst,
    {
        ImageCopyFromTextureDst {
            inner: self.image_copy_internal(mipmap_level, 0, F::BLOCK_SIZE),
        }
    }

    pub fn sub_image_copy_to_buffer_src(
        &self,
        descriptor: SubImageCopy3DDescriptor,
    ) -> SubImageCopySrc<F>
        where
            F: ImageCopyToBufferFormat + SubImageCopyFormat,
            U: CopySrc,
    {
        SubImageCopySrc {
            inner: self.sub_image_copy_internal(descriptor, F::BYTES_PER_BLOCK, F::BLOCK_SIZE),
        }
    }

    pub fn sub_image_copy_from_buffer_dst(
        &self,
        descriptor: SubImageCopy3DDescriptor,
    ) -> SubImageCopyDst<F>
        where
            F: ImageCopyFromBufferFormat + SubImageCopyFormat,
            U: CopyDst,
    {
        SubImageCopyDst {
            inner: self.sub_image_copy_internal(descriptor, F::BYTES_PER_BLOCK, F::BLOCK_SIZE),
        }
    }

    pub fn sub_image_copy_to_texture_src(
        &self,
        descriptor: SubImageCopy3DDescriptor,
    ) -> SubImageCopyToTextureSrc<F>
        where
            F: ImageCopyTextureFormat + SubImageCopyFormat,
            U: CopySrc,
    {
        SubImageCopyToTextureSrc {
            inner: self.sub_image_copy_internal(descriptor, 0, F::BLOCK_SIZE),
        }
    }

    pub fn sub_image_copy_from_texture_dst(
        &self,
        descriptor: SubImageCopy3DDescriptor,
    ) -> SubImageCopyFromTextureDst<F>
        where
            F: ImageCopyTextureFormat + SubImageCopyFormat,
            U: CopyDst,
    {
        SubImageCopyFromTextureDst {
            inner: self.sub_image_copy_internal(descriptor, 0, F::BLOCK_SIZE),
        }
    }
}

/// View on a 3D texture that can be bound to a pipeline as a float sampled texture resource.
pub struct Sampled3DFloat {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
}

/// View on a 3D texture that can be bound to a pipeline as a unfiltered float sampled texture
/// resource.
pub struct Sampled3DUnfilteredFloat {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
}

/// View on a 3D texture that can be bound to a pipeline as a signed integer sampled texture
/// resource.
pub struct Sampled3DSignedInteger {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
}

/// View on a 3D texture that can be bound to a pipeline as a unsigned integer sampled texture
/// resource.
pub struct Sampled3DUnsignedInteger {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
}

/// View on a 3D texture that can be bound to a pipeline as a texture storage resource.
pub struct Storage3D<F> {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
    _marker: marker::PhantomData<*const F>,
}
