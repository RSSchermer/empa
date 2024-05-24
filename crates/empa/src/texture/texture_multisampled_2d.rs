use std::marker;

use crate::device::Device;
use crate::driver;
use crate::driver::{
    Device as _, Driver, Dvr, Texture, TextureAspect, TextureDescriptor, TextureDimensions,
    TextureViewDescriptor, TextureViewDimension,
};
use crate::texture::format::MultisampleFormat;
use crate::texture::{
    CopyDst, CopySrc, FormatKind, ImageCopyTexture, ImageCopyToTextureDstMultisample,
    ImageCopyToTextureSrcMultisample, RenderAttachment, UsageFlags,
};

pub struct TextureMultisampled2DDescriptor {
    pub width: u32,
    pub height: u32,
}

pub struct TextureMultisampled2D<F, Usage, const SAMPLES: u8> {
    handle: <Dvr as Driver>::TextureHandle,
    width: u32,
    height: u32,
    _format: FormatKind<F>,
    _usage: marker::PhantomData<Usage>,
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

        let handle = device.handle.create_texture(&TextureDescriptor {
            size: (width, height, 1),
            mipmap_levels: 1,
            sample_count: SAMPLES as u32,
            dimensions: TextureDimensions::Two,
            format: F::FORMAT_ID,
            usage_flags: U::FLAG_SET,
            view_formats: &[],
        });

        TextureMultisampled2D {
            handle,
            width,
            height,
            _format: FormatKind::Typed(Default::default()),
            _usage: Default::default(),
        }
    }

    pub fn attachable_image(&self) -> AttachableMultisampledImage<F, SAMPLES> {
        let inner = self.handle.texture_view(&TextureViewDescriptor {
            format: F::FORMAT_ID,
            dimensions: TextureViewDimension::Two,
            aspect: TextureAspect::All,
            mip_levels: 0..1,
            layers: 0..1,
        });

        AttachableMultisampledImage {
            inner,
            width: self.width,
            height: self.height,
            _marker: Default::default(),
        }
    }

    fn image_copy_internal(&self) -> ImageCopyTexture<F> {
        let inner = driver::ImageCopyTexture {
            texture_handle: &self.handle,
            mip_level: 0,
            origin: (0, 0, 0),
            aspect: TextureAspect::All,
        };

        ImageCopyTexture {
            inner,
            width: self.width,
            height: self.height,
            depth_or_layers: 1,
            bytes_per_block: 0,
            block_size: [1, 1],
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

pub struct AttachableMultisampledImage<'a, F, const SAMPLES: u8> {
    pub(crate) inner: <Dvr as Driver>::TextureView,
    pub(crate) width: u32,
    pub(crate) height: u32,
    _marker: marker::PhantomData<&'a F>,
}
