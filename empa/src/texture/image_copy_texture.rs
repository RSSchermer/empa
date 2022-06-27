use std::marker;
use std::sync::Arc;

use web_sys::{GpuImageCopyTexture, GpuOrigin3dDict, GpuTextureAspect};

use crate::texture::{TextureDestroyer, ImageCopySize3D};

pub(crate) struct ImageCopyTexture<F> {
    pub(crate) texture: Arc<TextureDestroyer>,
    pub(crate) aspect: GpuTextureAspect,
    pub(crate) mipmap_level: u8,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) depth_or_layers: u32,
    pub(crate) bytes_per_block: u32,
    pub(crate) block_size: [u32; 2],
    pub(crate) origin_x: u32,
    pub(crate) origin_y: u32,
    pub(crate) origin_z: u32,
    pub(crate) _marker: marker::PhantomData<F>,
}

impl<F> ImageCopyTexture<F> {
    pub(crate) fn validate_src_with_size(&self, copy_size: ImageCopySize3D) {
        let ImageCopySize3D {
            width,
            height,
            depth_or_layers,
        } = copy_size;

        assert!(
            self.origin_x + width <= self.width,
            "copy width outside of `src`'s bounds"
        );
        assert!(
            self.origin_y + height <= self.height,
            "copy height outside of `src`'s bounds"
        );
        assert!(
            self.origin_z + depth_or_layers <= self.depth_or_layers,
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
            self.origin_x + width <= self.width,
            "copy width outside of `dst`'s bounds"
        );
        assert!(
            self.origin_y + height <= self.height,
            "copy height outside of `dst`'s bounds"
        );
        assert!(
            self.origin_z + depth_or_layers <= self.depth_or_layers,
            "copy depth/layers outside of `dst`'s bounds"
        );
    }

    pub(crate) fn to_web_sys(&self) -> GpuImageCopyTexture {
        let mut copy_texture = GpuImageCopyTexture::new(&self.texture.texture);

        let mut origin = GpuOrigin3dDict::new();

        origin.x(self.origin_x);
        origin.y(self.origin_y);
        origin.z(self.origin_z);

        copy_texture.aspect(self.aspect);
        copy_texture.mip_level(self.mipmap_level as u32);
        copy_texture.origin(origin.as_ref());

        copy_texture
    }
}

pub struct ImageCopyDst<F> {
    pub(crate) inner: ImageCopyTexture<F>,
}

pub struct SubImageCopyDst<F> {
    pub(crate) inner: ImageCopyTexture<F>,
}

pub struct ImageCopySrc<F> {
    pub(crate) inner: ImageCopyTexture<F>,
}

pub struct SubImageCopySrc<F> {
    pub(crate) inner: ImageCopyTexture<F>,
}

pub struct ImageCopyFromTextureDst<F> {
    pub(crate) inner: ImageCopyTexture<F>,
}

pub struct SubImageCopyFromTextureDst<F> {
    pub(crate) inner: ImageCopyTexture<F>,
}

pub struct ImageCopyToTextureSrc<F> {
    pub(crate) inner: ImageCopyTexture<F>,
}

pub struct SubImageCopyToTextureSrc<F> {
    pub(crate) inner: ImageCopyTexture<F>,
}

pub struct ImageCopyToTextureDstMultisample<F, const SAMPLES: u8> {
    pub(crate) inner: ImageCopyTexture<F>,
}

pub struct ImageCopyToTextureSrcMultisample<F, const SAMPLES: u8> {
    pub(crate) inner: ImageCopyTexture<F>,
}
