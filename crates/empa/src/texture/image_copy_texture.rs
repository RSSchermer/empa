use std::marker;

use crate::driver::{Dvr, ImageCopyTexture as ImageCopyTextureInternal};
use crate::texture::ImageCopySize3D;

pub(crate) struct ImageCopyTexture<'a, F> {
    pub(crate) inner: ImageCopyTextureInternal<'a, Dvr>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) depth_or_layers: u32,
    pub(crate) bytes_per_block: u32,
    pub(crate) block_size: [u32; 2],
    pub(crate) _marker: marker::PhantomData<F>,
}

impl<'a, F> ImageCopyTexture<'a, F> {
    pub(crate) fn validate_src_with_size(&self, copy_size: ImageCopySize3D) {
        let ImageCopySize3D {
            width,
            height,
            depth_or_layers,
        } = copy_size;

        assert!(
            self.inner.origin.0 + width <= self.width,
            "copy width outside of `src`'s bounds"
        );
        assert!(
            self.inner.origin.1 + height <= self.height,
            "copy height outside of `src`'s bounds"
        );
        assert!(
            self.inner.origin.2 + depth_or_layers <= self.depth_or_layers,
            "copy depth/layers outside of `src`'s bounds"
        );
    }

    pub(crate) fn validate_dst_with_size(&self, copy_size: ImageCopySize3D) {
        let ImageCopySize3D {
            width,
            height,
            depth_or_layers,
        } = copy_size;

        assert!(
            self.inner.origin.0 + width <= self.width,
            "copy width outside of `dst`'s bounds"
        );
        assert!(
            self.inner.origin.1 + height <= self.height,
            "copy height outside of `dst`'s bounds"
        );
        assert!(
            self.inner.origin.2 + depth_or_layers <= self.depth_or_layers,
            "copy depth/layers outside of `dst`'s bounds"
        );
    }
}

pub struct ImageCopyDst<'a, F> {
    pub(crate) inner: ImageCopyTexture<'a, F>,
}

pub struct SubImageCopyDst<'a, F> {
    pub(crate) inner: ImageCopyTexture<'a, F>,
}

pub struct ImageCopySrc<'a, F> {
    pub(crate) inner: ImageCopyTexture<'a, F>,
}

pub struct SubImageCopySrc<'a, F> {
    pub(crate) inner: ImageCopyTexture<'a, F>,
}

pub struct ImageCopyFromTextureDst<'a, F> {
    pub(crate) inner: ImageCopyTexture<'a, F>,
}

pub struct SubImageCopyFromTextureDst<'a, F> {
    pub(crate) inner: ImageCopyTexture<'a, F>,
}

pub struct ImageCopyToTextureSrc<'a, F> {
    pub(crate) inner: ImageCopyTexture<'a, F>,
}

pub struct SubImageCopyToTextureSrc<'a, F> {
    pub(crate) inner: ImageCopyTexture<'a, F>,
}

pub struct ImageCopyToTextureDstMultisample<'a, F, const SAMPLES: u8> {
    pub(crate) inner: ImageCopyTexture<'a, F>,
}

pub struct ImageCopyToTextureSrcMultisample<'a, F, const SAMPLES: u8> {
    pub(crate) inner: ImageCopyTexture<'a, F>,
}
