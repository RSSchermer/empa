use std::marker;
use std::sync::Arc;

use web_sys::{
    GpuExtent3dDict, GpuTexture, GpuTextureAspect, GpuTextureDescriptor, GpuTextureDimension,
    GpuTextureView, GpuTextureViewDescriptor, GpuTextureViewDimension,
};

use crate::device::Device;
use crate::texture::format::MultisampleFormat;
use crate::texture::{
    CopyDst, CopySrc, FormatKind, ImageCopyTexture, ImageCopyToTextureDstMultisample,
    ImageCopyToTextureSrcMultisample, RenderAttachment, TextureHandle, UsageFlags,
};

pub struct TextureMultisampled2DDescriptor {
    pub width: u32,
    pub height: u32,
}

pub struct TextureMultisampled2D<F, Usage, const SAMPLES: u8> {
    inner: Arc<TextureHandle>,
    width: u32,
    height: u32,
    format: FormatKind<F>,
    _usage: marker::PhantomData<Usage>,
}

impl<F, U, const SAMPLES: u8> TextureMultisampled2D<F, U, SAMPLES> {
    fn as_web_sys(&self) -> &GpuTexture {
        &self.inner.texture
    }
}

impl<F, U, const SAMPLES: u8> TextureMultisampled2D<F, U, SAMPLES>
where
    F: MultisampleFormat,
    U: UsageFlags + RenderAttachment,
{
    pub(crate) fn new(device: &Device, descriptor: &TextureMultisampled2DDescriptor) -> Self {
        assert!(
            SAMPLES == 4,
            "only a sample count of 4 is currently supported"
        );

        let TextureMultisampled2DDescriptor { width, height } = *descriptor;

        assert!(width > 0, "width must be greater than `0`");
        assert!(height > 0, "height must be greater than `0`");

        let mut size = GpuExtent3dDict::new(width);

        size.height(height);

        let mut desc = GpuTextureDescriptor::new(F::FORMAT_ID.to_web_sys(), &size.into(), U::BITS);

        desc.sample_count(SAMPLES as u32);
        desc.dimension(GpuTextureDimension::N2d);

        let inner = device.inner.create_texture(&desc);

        TextureMultisampled2D {
            inner: Arc::new(TextureHandle::new(inner, false)),
            width,
            height,
            format: FormatKind::Typed(Default::default()),
            _usage: Default::default(),
        }
    }

    pub fn attachable_image(&self) -> AttachableMultisampledImage<F, SAMPLES> {
        let mut desc = GpuTextureViewDescriptor::new();

        desc.dimension(GpuTextureViewDimension::N2d);
        desc.format(F::FORMAT_ID.to_web_sys());

        let inner = self.as_web_sys().create_view_with_descriptor(&desc);

        AttachableMultisampledImage {
            inner,
            width: self.width,
            height: self.height,
            _texture_handle: self.inner.clone(),
            _marker: Default::default(),
        }
    }

    fn image_copy_internal(&self) -> ImageCopyTexture<F> {
        ImageCopyTexture {
            texture: self.inner.clone(),
            aspect: GpuTextureAspect::All,
            mipmap_level: 0,
            width: self.width,
            height: self.height,
            depth_or_layers: 1,
            bytes_per_block: 0,
            block_size: [1, 1],
            origin_x: 0,
            origin_y: 0,
            origin_z: 0,
            _marker: Default::default(),
        }
    }

    pub fn image_copy_to_texture_src(&self) -> ImageCopyToTextureSrcMultisample<F, SAMPLES>
    where
        U: CopySrc,
    {
        ImageCopyToTextureSrcMultisample {
            inner: self.image_copy_internal(),
        }
    }

    pub fn image_copy_from_texture_dst(&self) -> ImageCopyToTextureDstMultisample<F, SAMPLES>
    where
        U: CopyDst,
    {
        ImageCopyToTextureDstMultisample {
            inner: self.image_copy_internal(),
        }
    }
}

pub struct AttachableMultisampledImage<F, const SAMPLES: u8> {
    pub(crate) inner: GpuTextureView,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) _texture_handle: Arc<TextureHandle>,
    _marker: marker::PhantomData<*const F>,
}
