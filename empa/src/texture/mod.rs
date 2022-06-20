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

use std::error::Error;
use std::fmt;

use staticvec::StaticVec;

use crate::texture::format::TextureFormatId;

pub(crate) struct TextureDestroyer {
    texture: web_sys::GpuTexture,
    is_swap_chain: bool
}

impl TextureDestroyer {
    fn new(texture: web_sys::GpuTexture, is_swap_chain: bool) -> Self {
        TextureDestroyer { texture, is_swap_chain }
    }
}

impl Drop for TextureDestroyer {
    fn drop(&mut self) {
        if !self.is_swap_chain {
            self.texture.destroy();
        }
    }
}

enum FormatKind<F> {
    Dynamic(F),
    Typed(std::marker::PhantomData<F>),
}

#[derive(Debug)]
pub struct UnsupportedViewFormat {
    pub(crate) format: TextureFormatId,
    pub(crate) supported_formats: StaticVec<TextureFormatId, 8>,
}

impl fmt::Display for UnsupportedViewFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}` is not one of the supported formats: ", self.format)?;

        let mut supported_formats = self.supported_formats.iter();

        if let Some(format) = supported_formats.next() {
            write!(f, "`{}`", format)?;
        }

        for format in supported_formats {
            write!(f, ", `{}`", format)?;
        }

        Ok(())
    }
}

impl Error for UnsupportedViewFormat {}
