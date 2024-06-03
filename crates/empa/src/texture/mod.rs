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
use std::ops::Rem;

use arrayvec::ArrayVec;

use crate::driver;
use crate::texture::format::TextureFormatId;

#[allow(unused)]
enum FormatKind<F> {
    Dynamic(F),
    Typed(std::marker::PhantomData<F>),
}

#[derive(Debug)]
pub struct UnsupportedViewFormat {
    pub(crate) format: TextureFormatId,
    pub(crate) supported_formats: ArrayVec<TextureFormatId, 8>,
}

impl fmt::Display for UnsupportedViewFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "`{:?}` is not one of the supported formats: ",
            self.format
        )?;

        let mut supported_formats = self.supported_formats.iter();

        if let Some(format) = supported_formats.next() {
            write!(f, "`{:?}`", format)?;
        }

        for format in supported_formats {
            write!(f, ", `{:?}`", format)?;
        }

        Ok(())
    }
}

impl Error for UnsupportedViewFormat {}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ImageDataLayout {
    pub blocks_per_row: u32,
    pub rows_per_image: u32,
}

impl ImageDataLayout {
    pub(crate) fn to_byte_layout(&self, bytes_per_block: u32) -> ImageDataByteLayout {
        let ImageDataLayout {
            blocks_per_row,
            rows_per_image,
        } = *self;

        ImageDataByteLayout {
            bytes_per_block,
            blocks_per_row,
            rows_per_image,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ImageDataByteLayout {
    pub bytes_per_block: u32,
    pub blocks_per_row: u32,
    pub rows_per_image: u32,
}

impl ImageDataByteLayout {
    pub(crate) fn to_driver(&self) -> driver::ImageDataLayout {
        let bytes_per_row = self.blocks_per_row * self.bytes_per_block;

        driver::ImageDataLayout {
            offset: 0,
            bytes_per_row,
            rows_per_image: self.rows_per_image,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ImageCopySize2D {
    pub width: u32,
    pub height: u32,
}

impl Default for ImageCopySize2D {
    fn default() -> Self {
        ImageCopySize2D {
            width: 1,
            height: 1,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ImageCopySize3D {
    pub width: u32,
    pub height: u32,
    pub depth_or_layers: u32,
}

impl Default for ImageCopySize3D {
    fn default() -> Self {
        ImageCopySize3D {
            width: 1,
            height: 1,
            depth_or_layers: 1,
        }
    }
}

impl ImageCopySize3D {
    pub(crate) fn validate_with_block_size(&self, block_size: [u32; 2]) {
        let ImageCopySize3D {
            width,
            height,
            depth_or_layers,
        } = *self;

        assert!(width != 0, "copy width cannot be `0`");
        assert!(height != 0, "copy height cannot be `0`");
        assert!(
            depth_or_layers != 0,
            "copy depth or layer count cannot be `0`"
        );

        let [block_width, block_height] = block_size;

        assert!(
            width.rem(block_width) == 0,
            "copy width must be a multiple of the block width (`{}`)",
            block_width
        );
        assert!(
            height.rem(block_height) == 0,
            "copy height must be a multiple of the block height (`{}`)",
            block_height
        );
    }
}
