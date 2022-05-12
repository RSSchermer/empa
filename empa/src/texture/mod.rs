mod image_copy_texture;
pub use self::image_copy_texture::*;

mod mipmap_levels;
pub use self::mipmap_levels::*;

mod texture_1d;
pub use self::texture_1d::*;

mod texture_2d;
pub use self::texture_2d::*;

mod texture_3d;
pub use self::texture_3d::*;

mod texture_multisampled_2d;
pub use self::texture_multisampled_2d::*;

mod usage;
pub use self::usage::*;

pub mod format;

pub(crate) struct TextureDestroyer {
    texture: web_sys::GpuTexture,
}

impl TextureDestroyer {
    fn new(texture: web_sys::GpuTexture) -> Self {
        TextureDestroyer { texture }
    }
}

impl Drop for TextureDestroyer {
    fn drop(&mut self) {
        self.texture.destroy();
    }
}

enum FormatKind<F> {
    Dynamic(F),
    Typed(std::marker::PhantomData<F>),
}
