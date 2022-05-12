#![allow(non_camel_case_types)]

use web_sys::GpuTextureFormat;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TextureFormatId {
    inner: GpuTextureFormat,
}

impl TextureFormatId {
    pub(crate) fn to_web_sys(&self) -> GpuTextureFormat {
        self.inner
    }

    // Note: below 4 methods exist to determine compatibility with fragment shader outputs. However,
    // I can not currently find an explicit definition in the spec of what compatibility means, so
    // this may well be incorrect.
    pub(crate) fn is_float(&self) -> bool {
        match self.inner {
            GpuTextureFormat::R8unorm
            | GpuTextureFormat::R8snorm
            | GpuTextureFormat::Rg8unorm
            | GpuTextureFormat::Rg8snorm
            | GpuTextureFormat::R32float
            | GpuTextureFormat::Rgba8unorm
            | GpuTextureFormat::Rgba8unormSrgb
            | GpuTextureFormat::Rgba8snorm
            | GpuTextureFormat::Bgra8unorm
            | GpuTextureFormat::Bgra8unormSrgb
            | GpuTextureFormat::Rgb9e5ufloat
            | GpuTextureFormat::Rgb10a2unorm
            | GpuTextureFormat::Rg32float
            | GpuTextureFormat::Rgba32float => true,
            _ => false,
        }
    }

    pub(crate) fn is_half_float(&self) -> bool {
        match self.inner {
            GpuTextureFormat::R16float
            | GpuTextureFormat::Rg16float
            | GpuTextureFormat::Rgba16float => true,
            _ => false,
        }
    }

    pub(crate) fn is_signed_integer(&self) -> bool {
        match self.inner {
            GpuTextureFormat::R8sint
            | GpuTextureFormat::R16sint
            | GpuTextureFormat::Rg8sint
            | GpuTextureFormat::R32sint
            | GpuTextureFormat::Rg16sint
            | GpuTextureFormat::Rgba8sint
            | GpuTextureFormat::Rg32sint
            | GpuTextureFormat::Rgba16sint
            | GpuTextureFormat::Rgba32sint => true,
            _ => false,
        }
    }

    pub(crate) fn is_unsigned_integer(&self) -> bool {
        match self.inner {
            GpuTextureFormat::R8uint
            | GpuTextureFormat::R16uint
            | GpuTextureFormat::Rg8uint
            | GpuTextureFormat::R32uint
            | GpuTextureFormat::Rg16uint
            | GpuTextureFormat::Rgba8uint
            | GpuTextureFormat::Rg32uint
            | GpuTextureFormat::Rgba16uint
            | GpuTextureFormat::Rgba32uint => true,
            _ => false,
        }
    }
}

mod texture_format_seal {
    pub trait Seal {}
}

pub trait TextureFormat: texture_format_seal::Seal {
    const FORMAT_ID: TextureFormatId;

    const BLOCK_SIZE: [u32; 2];
}

macro_rules! typed_texture_format {
    ($format:ident, $web_sys:ident, $block_width:literal, $block_height:literal) => {
        pub struct $format;

        impl texture_format_seal::Seal for $format {}
        impl TextureFormat for $format {
            const FORMAT_ID: TextureFormatId = TextureFormatId {
                inner: GpuTextureFormat::$web_sys,
            };

            const BLOCK_SIZE: [u32; 2] = [$block_width, $block_height];
        }
    };
    ($format:ident, $web_sys:ident) => {
        typed_texture_format!($format, $web_sys, 1, 1);
    };
}

typed_texture_format!(r8unorm, R8unorm);
typed_texture_format!(r8snorm, R8snorm);
typed_texture_format!(r8uint, R8uint);
typed_texture_format!(r8sint, R8sint);
typed_texture_format!(r16uint, R16uint);
typed_texture_format!(r16sint, R16sint);
typed_texture_format!(r16float, R16float);
typed_texture_format!(rg8unorm, Rg8unorm);
typed_texture_format!(rg8snorm, Rg8snorm);
typed_texture_format!(rg8uint, Rg8uint);
typed_texture_format!(rg8sint, Rg8sint);
typed_texture_format!(r32uint, R32uint);
typed_texture_format!(r32sint, R32sint);
typed_texture_format!(r32float, R32float);
typed_texture_format!(rg16uint, Rg16uint);
typed_texture_format!(rg16sint, Rg16sint);
typed_texture_format!(rg16float, Rg16float);
typed_texture_format!(rgba8unorm, Rgba8unorm);
typed_texture_format!(rgba8unorm_srgb, Rgba8unormSrgb);
typed_texture_format!(rgba8snorm, Rgba8snorm);
typed_texture_format!(rgba8uint, Rgba8uint);
typed_texture_format!(rgba8sint, Rgba8sint);
typed_texture_format!(bgra8unorm, Bgra8unorm);
typed_texture_format!(bgra8unorm_srgb, Bgra8unormSrgb);
typed_texture_format!(rgb9e5ufloat, Rgb9e5ufloat);
typed_texture_format!(rgb10a2unorm, Rgb10a2unorm);
typed_texture_format!(rg11b10ufloat, Rg11b10ufloat);
typed_texture_format!(rg32uint, Rg32uint);
typed_texture_format!(rg32sint, Rg32sint);
typed_texture_format!(rg32float, Rg32float);
typed_texture_format!(rgba16uint, Rgba16uint);
typed_texture_format!(rgba16sint, Rgba16sint);
typed_texture_format!(rgba16float, Rgba16float);
typed_texture_format!(rgba32uint, Rgba32uint);
typed_texture_format!(rgba32sint, Rgba32sint);
typed_texture_format!(rgba32float, Rgba32float);
typed_texture_format!(stencil8, Stencil8);
typed_texture_format!(depth16unorm, Depth16unorm);
typed_texture_format!(depth24plus, Depth24plus);
typed_texture_format!(depth24plus_stencil8, Depth24plusStencil8);
typed_texture_format!(depth32float, Depth32float);
typed_texture_format!(depth24unorm_stencil8, Depth24unormStencil8);
typed_texture_format!(depth32float_stencil8, Depth32floatStencil8);
typed_texture_format!(bc1_rgba_unorm, Bc1RgbaUnorm, 4, 4);
typed_texture_format!(bc1_rgba_unorm_srgb, Bc1RgbaUnormSrgb, 4, 4);
typed_texture_format!(bc2_rgba_unorm, Bc2RgbaUnorm, 4, 4);
typed_texture_format!(bc2_rgba_unorm_srgb, Bc2RgbaUnormSrgb, 4, 4);
typed_texture_format!(bc3_rgba_unorm, Bc3RgbaUnorm, 4, 4);
typed_texture_format!(bc3_rgba_unorm_srgb, Bc3RgbaUnormSrgb, 4, 4);
typed_texture_format!(bc4_r_unorm, Bc4RUnorm, 4, 4);
typed_texture_format!(bc4_r_snorm, Bc4RSnorm, 4, 4);
typed_texture_format!(bc5_rg_unorm, Bc5RgUnorm, 4, 4);
typed_texture_format!(bc5_rg_snorm, Bc5RgSnorm, 4, 4);
typed_texture_format!(bc6h_rgb_ufloat, Bc6hRgbUfloat, 4, 4);
typed_texture_format!(bc6h_rgb_float, Bc6hRgbFloat, 4, 4);
typed_texture_format!(bc7_rgba_unorm, Bc7RgbaUnorm, 4, 4);
typed_texture_format!(bc7_rgba_unorm_srgb, Bc7RgbaUnormSrgb, 4, 4);

// TODO: these are given temporary incorrect web_sys tags, because they are currently not in web_sys
// Get web_sys up to date with the spec and replace these tags.
typed_texture_format!(etc2_rgb8unorm, R8snorm, 4, 4);
typed_texture_format!(etc2_rgb8unorm_srgb, R8snorm, 4, 4);
typed_texture_format!(etc2_rgb8a1unorm, R8snorm, 4, 4);
typed_texture_format!(etc2_rgb8a1unorm_srgb, R8snorm, 4, 4);
typed_texture_format!(etc2_rgba8unorm, R8snorm, 4, 4);
typed_texture_format!(etc2_rgba8unorm_srgb, R8snorm, 4, 4);
typed_texture_format!(eac_r11unorm, R8snorm, 4, 4);
typed_texture_format!(eac_r11snorm, R8snorm, 4, 4);
typed_texture_format!(eac_rg11unorm, R8snorm, 4, 4);
typed_texture_format!(eac_rg11snorm, R8snorm, 4, 4);
typed_texture_format!(astc_4x4_unorm, R8snorm, 4, 4);
typed_texture_format!(astc_4x4_unorm_srgb, R8snorm, 4, 4);
typed_texture_format!(astc_5x4_unorm, R8snorm, 5, 4);
typed_texture_format!(astc_5x4_unorm_srgb, R8snorm, 5, 4);
typed_texture_format!(astc_5x5_unorm, R8snorm, 5, 5);
typed_texture_format!(astc_5x5_unorm_srgb, R8snorm, 5, 5);
typed_texture_format!(astc_6x5_unorm, R8snorm, 6, 5);
typed_texture_format!(astc_6x5_unorm_srgb, R8snorm, 6, 5);
typed_texture_format!(astc_6x6_unorm, R8snorm, 6, 6);
typed_texture_format!(astc_6x6_unorm_srgb, R8snorm, 6, 6);
typed_texture_format!(astc_8x5_unorm, R8snorm, 8, 5);
typed_texture_format!(astc_8x5_unorm_srgb, R8snorm, 8, 5);
typed_texture_format!(astc_8x6_unorm, R8snorm, 8, 6);
typed_texture_format!(astc_8x6_unorm_srgb, R8snorm, 8, 6);
typed_texture_format!(astc_8x8_unorm, R8snorm, 8, 8);
typed_texture_format!(astc_8x8_unorm_srgb, R8snorm, 8, 8);
typed_texture_format!(astc_10x5_unorm, R8snorm, 10, 5);
typed_texture_format!(astc_10x5_unorm_srgb, R8snorm, 10, 5);
typed_texture_format!(astc_10x6_unorm, R8snorm, 10, 6);
typed_texture_format!(astc_10x6_unorm_srgb, R8snorm, 10, 6);
typed_texture_format!(astc_10x8_unorm, R8snorm, 10, 8);
typed_texture_format!(astc_10x8_unorm_srgb, R8snorm, 10, 8);
typed_texture_format!(astc_10x10_unorm, R8snorm, 10, 10);
typed_texture_format!(astc_10x10_unorm_srgb, R8snorm, 10, 10);
typed_texture_format!(astc_12x10_unorm, R8snorm, 12, 10);
typed_texture_format!(astc_12x10_unorm_srgb, R8snorm, 12, 10);
typed_texture_format!(astc_12x12_unorm, R8snorm, 12, 12);
typed_texture_format!(astc_12x12_unorm_srgb, R8snorm, 12, 12);

pub trait Texture1DFormat: TextureFormat {}

impl Texture1DFormat for r8unorm {}
impl Texture1DFormat for r8snorm {}
impl Texture1DFormat for r8uint {}
impl Texture1DFormat for r8sint {}
impl Texture1DFormat for r16uint {}
impl Texture1DFormat for r16sint {}
impl Texture1DFormat for r16float {}
impl Texture1DFormat for rg8unorm {}
impl Texture1DFormat for rg8snorm {}
impl Texture1DFormat for rg8uint {}
impl Texture1DFormat for rg8sint {}
impl Texture1DFormat for r32uint {}
impl Texture1DFormat for r32sint {}
impl Texture1DFormat for r32float {}
impl Texture1DFormat for rg16uint {}
impl Texture1DFormat for rg16sint {}
impl Texture1DFormat for rg16float {}
impl Texture1DFormat for rgba8unorm {}
impl Texture1DFormat for rgba8unorm_srgb {}
impl Texture1DFormat for rgba8snorm {}
impl Texture1DFormat for rgba8uint {}
impl Texture1DFormat for rgba8sint {}
impl Texture1DFormat for bgra8unorm {}
impl Texture1DFormat for bgra8unorm_srgb {}
impl Texture1DFormat for rgb9e5ufloat {}
impl Texture1DFormat for rgb10a2unorm {}
impl Texture1DFormat for rg11b10ufloat {}
impl Texture1DFormat for rg32uint {}
impl Texture1DFormat for rg32sint {}
impl Texture1DFormat for rg32float {}
impl Texture1DFormat for rgba16uint {}
impl Texture1DFormat for rgba16sint {}
impl Texture1DFormat for rgba16float {}
impl Texture1DFormat for rgba32uint {}
impl Texture1DFormat for rgba32sint {}
impl Texture1DFormat for rgba32float {}

pub trait Texture2DFormat: TextureFormat {}

impl Texture2DFormat for r8unorm {}
impl Texture2DFormat for r8snorm {}
impl Texture2DFormat for r8uint {}
impl Texture2DFormat for r8sint {}
impl Texture2DFormat for r16uint {}
impl Texture2DFormat for r16sint {}
impl Texture2DFormat for r16float {}
impl Texture2DFormat for rg8unorm {}
impl Texture2DFormat for rg8snorm {}
impl Texture2DFormat for rg8uint {}
impl Texture2DFormat for rg8sint {}
impl Texture2DFormat for r32uint {}
impl Texture2DFormat for r32sint {}
impl Texture2DFormat for r32float {}
impl Texture2DFormat for rg16uint {}
impl Texture2DFormat for rg16sint {}
impl Texture2DFormat for rg16float {}
impl Texture2DFormat for rgba8unorm {}
impl Texture2DFormat for rgba8unorm_srgb {}
impl Texture2DFormat for rgba8snorm {}
impl Texture2DFormat for rgba8uint {}
impl Texture2DFormat for rgba8sint {}
impl Texture2DFormat for bgra8unorm {}
impl Texture2DFormat for bgra8unorm_srgb {}
impl Texture2DFormat for rgb9e5ufloat {}
impl Texture2DFormat for rgb10a2unorm {}
impl Texture2DFormat for rg11b10ufloat {}
impl Texture2DFormat for rg32uint {}
impl Texture2DFormat for rg32sint {}
impl Texture2DFormat for rg32float {}
impl Texture2DFormat for rgba16uint {}
impl Texture2DFormat for rgba16sint {}
impl Texture2DFormat for rgba16float {}
impl Texture2DFormat for rgba32uint {}
impl Texture2DFormat for rgba32sint {}
impl Texture2DFormat for rgba32float {}
impl Texture2DFormat for stencil8 {}
impl Texture2DFormat for depth16unorm {}
impl Texture2DFormat for depth24plus {}
impl Texture2DFormat for depth24plus_stencil8 {}
impl Texture2DFormat for depth32float {}
impl Texture2DFormat for depth24unorm_stencil8 {}
impl Texture2DFormat for depth32float_stencil8 {}
impl Texture2DFormat for bc1_rgba_unorm {}
impl Texture2DFormat for bc1_rgba_unorm_srgb {}
impl Texture2DFormat for bc2_rgba_unorm {}
impl Texture2DFormat for bc2_rgba_unorm_srgb {}
impl Texture2DFormat for bc3_rgba_unorm {}
impl Texture2DFormat for bc3_rgba_unorm_srgb {}
impl Texture2DFormat for bc4_r_unorm {}
impl Texture2DFormat for bc4_r_snorm {}
impl Texture2DFormat for bc5_rg_unorm {}
impl Texture2DFormat for bc5_rg_snorm {}
impl Texture2DFormat for bc6h_rgb_ufloat {}
impl Texture2DFormat for bc6h_rgb_float {}
impl Texture2DFormat for bc7_rgba_unorm {}
impl Texture2DFormat for bc7_rgba_unorm_srgb {}
impl Texture2DFormat for etc2_rgb8unorm {}
impl Texture2DFormat for etc2_rgb8unorm_srgb {}
impl Texture2DFormat for etc2_rgb8a1unorm {}
impl Texture2DFormat for etc2_rgb8a1unorm_srgb {}
impl Texture2DFormat for etc2_rgba8unorm {}
impl Texture2DFormat for etc2_rgba8unorm_srgb {}
impl Texture2DFormat for eac_r11unorm {}
impl Texture2DFormat for eac_r11snorm {}
impl Texture2DFormat for eac_rg11unorm {}
impl Texture2DFormat for eac_rg11snorm {}
impl Texture2DFormat for astc_4x4_unorm {}
impl Texture2DFormat for astc_4x4_unorm_srgb {}
impl Texture2DFormat for astc_5x4_unorm {}
impl Texture2DFormat for astc_5x4_unorm_srgb {}
impl Texture2DFormat for astc_5x5_unorm {}
impl Texture2DFormat for astc_5x5_unorm_srgb {}
impl Texture2DFormat for astc_6x5_unorm {}
impl Texture2DFormat for astc_6x5_unorm_srgb {}
impl Texture2DFormat for astc_6x6_unorm {}
impl Texture2DFormat for astc_6x6_unorm_srgb {}
impl Texture2DFormat for astc_8x5_unorm {}
impl Texture2DFormat for astc_8x5_unorm_srgb {}
impl Texture2DFormat for astc_8x6_unorm {}
impl Texture2DFormat for astc_8x6_unorm_srgb {}
impl Texture2DFormat for astc_8x8_unorm {}
impl Texture2DFormat for astc_8x8_unorm_srgb {}
impl Texture2DFormat for astc_10x5_unorm {}
impl Texture2DFormat for astc_10x5_unorm_srgb {}
impl Texture2DFormat for astc_10x6_unorm {}
impl Texture2DFormat for astc_10x6_unorm_srgb {}
impl Texture2DFormat for astc_10x8_unorm {}
impl Texture2DFormat for astc_10x8_unorm_srgb {}
impl Texture2DFormat for astc_10x10_unorm {}
impl Texture2DFormat for astc_10x10_unorm_srgb {}
impl Texture2DFormat for astc_12x10_unorm {}
impl Texture2DFormat for astc_12x10_unorm_srgb {}
impl Texture2DFormat for astc_12x12_unorm {}
impl Texture2DFormat for astc_12x12_unorm_srgb {}

pub trait Texture3DFormat: TextureFormat {}

impl Texture3DFormat for r8unorm {}
impl Texture3DFormat for r8snorm {}
impl Texture3DFormat for r8uint {}
impl Texture3DFormat for r8sint {}
impl Texture3DFormat for r16uint {}
impl Texture3DFormat for r16sint {}
impl Texture3DFormat for r16float {}
impl Texture3DFormat for rg8unorm {}
impl Texture3DFormat for rg8snorm {}
impl Texture3DFormat for rg8uint {}
impl Texture3DFormat for rg8sint {}
impl Texture3DFormat for r32uint {}
impl Texture3DFormat for r32sint {}
impl Texture3DFormat for r32float {}
impl Texture3DFormat for rg16uint {}
impl Texture3DFormat for rg16sint {}
impl Texture3DFormat for rg16float {}
impl Texture3DFormat for rgba8unorm {}
impl Texture3DFormat for rgba8unorm_srgb {}
impl Texture3DFormat for rgba8snorm {}
impl Texture3DFormat for rgba8uint {}
impl Texture3DFormat for rgba8sint {}
impl Texture3DFormat for bgra8unorm {}
impl Texture3DFormat for bgra8unorm_srgb {}
impl Texture3DFormat for rgb9e5ufloat {}
impl Texture3DFormat for rgb10a2unorm {}
impl Texture3DFormat for rg11b10ufloat {}
impl Texture3DFormat for rg32uint {}
impl Texture3DFormat for rg32sint {}
impl Texture3DFormat for rg32float {}
impl Texture3DFormat for rgba16uint {}
impl Texture3DFormat for rgba16sint {}
impl Texture3DFormat for rgba16float {}
impl Texture3DFormat for rgba32uint {}
impl Texture3DFormat for rgba32sint {}
impl Texture3DFormat for rgba32float {}

pub trait FloatSamplable: TextureFormat {}

impl FloatSamplable for r8unorm {}
impl FloatSamplable for r8snorm {}
impl FloatSamplable for rg8unorm {}
impl FloatSamplable for rg8snorm {}
impl FloatSamplable for rgba8unorm {}
impl FloatSamplable for rgba8unorm_srgb {}
impl FloatSamplable for rgba8snorm {}
impl FloatSamplable for bgra8unorm {}
impl FloatSamplable for bgra8unorm_srgb {}
impl FloatSamplable for r16float {}
impl FloatSamplable for rg16float {}
impl FloatSamplable for rgba16float {}
impl FloatSamplable for rgb10a2unorm {}
impl FloatSamplable for rg11b10ufloat {}
impl FloatSamplable for rgb9e5ufloat {}
impl FloatSamplable for bc1_rgba_unorm {}
impl FloatSamplable for bc1_rgba_unorm_srgb {}
impl FloatSamplable for bc2_rgba_unorm {}
impl FloatSamplable for bc2_rgba_unorm_srgb {}
impl FloatSamplable for bc3_rgba_unorm {}
impl FloatSamplable for bc3_rgba_unorm_srgb {}
impl FloatSamplable for bc4_r_unorm {}
impl FloatSamplable for bc4_r_snorm {}
impl FloatSamplable for bc5_rg_unorm {}
impl FloatSamplable for bc5_rg_snorm {}
impl FloatSamplable for bc6h_rgb_ufloat {}
impl FloatSamplable for bc6h_rgb_float {}
impl FloatSamplable for bc7_rgba_unorm {}
impl FloatSamplable for bc7_rgba_unorm_srgb {}
impl FloatSamplable for etc2_rgb8unorm {}
impl FloatSamplable for etc2_rgb8unorm_srgb {}
impl FloatSamplable for etc2_rgb8a1unorm {}
impl FloatSamplable for etc2_rgb8a1unorm_srgb {}
impl FloatSamplable for etc2_rgba8unorm {}
impl FloatSamplable for etc2_rgba8unorm_srgb {}
impl FloatSamplable for eac_r11unorm {}
impl FloatSamplable for eac_r11snorm {}
impl FloatSamplable for eac_rg11unorm {}
impl FloatSamplable for eac_rg11snorm {}
impl FloatSamplable for astc_4x4_unorm {}
impl FloatSamplable for astc_4x4_unorm_srgb {}
impl FloatSamplable for astc_5x4_unorm {}
impl FloatSamplable for astc_5x4_unorm_srgb {}
impl FloatSamplable for astc_5x5_unorm {}
impl FloatSamplable for astc_5x5_unorm_srgb {}
impl FloatSamplable for astc_6x5_unorm {}
impl FloatSamplable for astc_6x5_unorm_srgb {}
impl FloatSamplable for astc_6x6_unorm {}
impl FloatSamplable for astc_6x6_unorm_srgb {}
impl FloatSamplable for astc_8x5_unorm {}
impl FloatSamplable for astc_8x5_unorm_srgb {}
impl FloatSamplable for astc_8x6_unorm {}
impl FloatSamplable for astc_8x6_unorm_srgb {}
impl FloatSamplable for astc_8x8_unorm {}
impl FloatSamplable for astc_8x8_unorm_srgb {}
impl FloatSamplable for astc_10x5_unorm {}
impl FloatSamplable for astc_10x5_unorm_srgb {}
impl FloatSamplable for astc_10x6_unorm {}
impl FloatSamplable for astc_10x6_unorm_srgb {}
impl FloatSamplable for astc_10x8_unorm {}
impl FloatSamplable for astc_10x8_unorm_srgb {}
impl FloatSamplable for astc_10x10_unorm {}
impl FloatSamplable for astc_10x10_unorm_srgb {}
impl FloatSamplable for astc_12x10_unorm {}
impl FloatSamplable for astc_12x10_unorm_srgb {}
impl FloatSamplable for astc_12x12_unorm {}
impl FloatSamplable for astc_12x12_unorm_srgb {}

pub trait UnfilteredFloatSamplable: TextureFormat {}

impl UnfilteredFloatSamplable for r8unorm {}
impl UnfilteredFloatSamplable for r8snorm {}
impl UnfilteredFloatSamplable for rg8unorm {}
impl UnfilteredFloatSamplable for rg8snorm {}
impl UnfilteredFloatSamplable for rgba8unorm {}
impl UnfilteredFloatSamplable for rgba8unorm_srgb {}
impl UnfilteredFloatSamplable for rgba8snorm {}
impl UnfilteredFloatSamplable for bgra8unorm {}
impl UnfilteredFloatSamplable for bgra8unorm_srgb {}
impl UnfilteredFloatSamplable for r16float {}
impl UnfilteredFloatSamplable for rg16float {}
impl UnfilteredFloatSamplable for rgba16float {}
impl UnfilteredFloatSamplable for rgb10a2unorm {}
impl UnfilteredFloatSamplable for rg11b10ufloat {}
impl UnfilteredFloatSamplable for rgb9e5ufloat {}
impl UnfilteredFloatSamplable for bc1_rgba_unorm {}
impl UnfilteredFloatSamplable for bc1_rgba_unorm_srgb {}
impl UnfilteredFloatSamplable for bc2_rgba_unorm {}
impl UnfilteredFloatSamplable for bc2_rgba_unorm_srgb {}
impl UnfilteredFloatSamplable for bc3_rgba_unorm {}
impl UnfilteredFloatSamplable for bc3_rgba_unorm_srgb {}
impl UnfilteredFloatSamplable for bc4_r_unorm {}
impl UnfilteredFloatSamplable for bc4_r_snorm {}
impl UnfilteredFloatSamplable for bc5_rg_unorm {}
impl UnfilteredFloatSamplable for bc5_rg_snorm {}
impl UnfilteredFloatSamplable for bc6h_rgb_ufloat {}
impl UnfilteredFloatSamplable for bc6h_rgb_float {}
impl UnfilteredFloatSamplable for bc7_rgba_unorm {}
impl UnfilteredFloatSamplable for bc7_rgba_unorm_srgb {}
impl UnfilteredFloatSamplable for etc2_rgb8unorm {}
impl UnfilteredFloatSamplable for etc2_rgb8unorm_srgb {}
impl UnfilteredFloatSamplable for etc2_rgb8a1unorm {}
impl UnfilteredFloatSamplable for etc2_rgb8a1unorm_srgb {}
impl UnfilteredFloatSamplable for etc2_rgba8unorm {}
impl UnfilteredFloatSamplable for etc2_rgba8unorm_srgb {}
impl UnfilteredFloatSamplable for eac_r11unorm {}
impl UnfilteredFloatSamplable for eac_r11snorm {}
impl UnfilteredFloatSamplable for eac_rg11unorm {}
impl UnfilteredFloatSamplable for eac_rg11snorm {}
impl UnfilteredFloatSamplable for astc_4x4_unorm {}
impl UnfilteredFloatSamplable for astc_4x4_unorm_srgb {}
impl UnfilteredFloatSamplable for astc_5x4_unorm {}
impl UnfilteredFloatSamplable for astc_5x4_unorm_srgb {}
impl UnfilteredFloatSamplable for astc_5x5_unorm {}
impl UnfilteredFloatSamplable for astc_5x5_unorm_srgb {}
impl UnfilteredFloatSamplable for astc_6x5_unorm {}
impl UnfilteredFloatSamplable for astc_6x5_unorm_srgb {}
impl UnfilteredFloatSamplable for astc_6x6_unorm {}
impl UnfilteredFloatSamplable for astc_6x6_unorm_srgb {}
impl UnfilteredFloatSamplable for astc_8x5_unorm {}
impl UnfilteredFloatSamplable for astc_8x5_unorm_srgb {}
impl UnfilteredFloatSamplable for astc_8x6_unorm {}
impl UnfilteredFloatSamplable for astc_8x6_unorm_srgb {}
impl UnfilteredFloatSamplable for astc_8x8_unorm {}
impl UnfilteredFloatSamplable for astc_8x8_unorm_srgb {}
impl UnfilteredFloatSamplable for astc_10x5_unorm {}
impl UnfilteredFloatSamplable for astc_10x5_unorm_srgb {}
impl UnfilteredFloatSamplable for astc_10x6_unorm {}
impl UnfilteredFloatSamplable for astc_10x6_unorm_srgb {}
impl UnfilteredFloatSamplable for astc_10x8_unorm {}
impl UnfilteredFloatSamplable for astc_10x8_unorm_srgb {}
impl UnfilteredFloatSamplable for astc_10x10_unorm {}
impl UnfilteredFloatSamplable for astc_10x10_unorm_srgb {}
impl UnfilteredFloatSamplable for astc_12x10_unorm {}
impl UnfilteredFloatSamplable for astc_12x10_unorm_srgb {}
impl UnfilteredFloatSamplable for astc_12x12_unorm {}
impl UnfilteredFloatSamplable for astc_12x12_unorm_srgb {}
impl UnfilteredFloatSamplable for r32float {}
impl UnfilteredFloatSamplable for rg32float {}
impl UnfilteredFloatSamplable for rgba32float {}

pub trait SignedIntegerSamplable: TextureFormat {}

impl SignedIntegerSamplable for r8sint {}
impl SignedIntegerSamplable for rg8sint {}
impl SignedIntegerSamplable for rgba8sint {}
impl SignedIntegerSamplable for r16sint {}
impl SignedIntegerSamplable for rg16sint {}
impl SignedIntegerSamplable for rgba16sint {}
impl SignedIntegerSamplable for r32sint {}
impl SignedIntegerSamplable for rgba32sint {}

pub trait UnsignedIntegerSamplable: TextureFormat {}

impl UnsignedIntegerSamplable for r8uint {}
impl UnsignedIntegerSamplable for rg8uint {}
impl UnsignedIntegerSamplable for rgba8uint {}
impl UnsignedIntegerSamplable for r16uint {}
impl UnsignedIntegerSamplable for rg16uint {}
impl UnsignedIntegerSamplable for rgba16uint {}
impl UnsignedIntegerSamplable for r32uint {}
impl UnsignedIntegerSamplable for rgba32uint {}

pub trait DepthSamplable: TextureFormat {}

impl DepthSamplable for depth16unorm {}
impl DepthSamplable for depth24plus {}
impl DepthSamplable for depth32float {}

pub trait DepthStencilFormat: TextureFormat {
    type DepthAspect: TextureFormat;

    type StencilAspect: TextureFormat;
}

impl DepthStencilFormat for depth24plus_stencil8 {
    type DepthAspect = depth24plus;
    type StencilAspect = stencil8;
}

impl DepthStencilFormat for depth24unorm_stencil8 {
    type DepthAspect = depth24plus;
    type StencilAspect = stencil8;
}

impl DepthStencilFormat for depth32float_stencil8 {
    type DepthAspect = depth32float;
    type StencilAspect = stencil8;
}

pub trait Storable: TextureFormat {}

impl Storable for rgba8unorm {}
impl Storable for rgba8snorm {}
impl Storable for rgba8uint {}
impl Storable for rgba8sint {}
impl Storable for rgba16uint {}
impl Storable for rgba16sint {}
impl Storable for rgba16float {}
impl Storable for r32uint {}
impl Storable for r32sint {}
impl Storable for r32float {}
impl Storable for rg32uint {}
impl Storable for rg32sint {}
impl Storable for rg32float {}
impl Storable for rgba32uint {}
impl Storable for rgba32sint {}
impl Storable for rgba32float {}

pub trait Renderable: TextureFormat {}

impl Renderable for r8unorm {}
impl Renderable for rg8unorm {}
impl Renderable for rgba8unorm {}
impl Renderable for rgba8unorm_srgb {}
impl Renderable for bgra8unorm {}
impl Renderable for bgra8unorm_srgb {}
impl Renderable for r16float {}
impl Renderable for rg16float {}
impl Renderable for rgba16float {}
impl Renderable for r32float {}
impl Renderable for rgba32float {}
impl Renderable for rgb10a2unorm {}
impl Renderable for r8sint {}
impl Renderable for rg8sint {}
impl Renderable for rgba8sint {}
impl Renderable for r16sint {}
impl Renderable for rg16sint {}
impl Renderable for rgba16sint {}
impl Renderable for r32sint {}
impl Renderable for rg32sint {}
impl Renderable for rgba32sint {}
impl Renderable for r8uint {}
impl Renderable for rg8uint {}
impl Renderable for rgba8uint {}
impl Renderable for r16uint {}
impl Renderable for rg16uint {}
impl Renderable for rgba16uint {}
impl Renderable for r32uint {}
impl Renderable for rg32uint {}
impl Renderable for rgba32uint {}
impl Renderable for stencil8 {}
impl Renderable for depth16unorm {}
impl Renderable for depth24plus {}
impl Renderable for depth24plus_stencil8 {}
impl Renderable for depth32float {}
impl Renderable for depth24unorm_stencil8 {}
impl Renderable for depth32float_stencil8 {}

pub trait ColorRenderable: Renderable {}

impl ColorRenderable for r8unorm {}
impl ColorRenderable for rg8unorm {}
impl ColorRenderable for rgba8unorm {}
impl ColorRenderable for rgba8unorm_srgb {}
impl ColorRenderable for bgra8unorm {}
impl ColorRenderable for bgra8unorm_srgb {}
impl ColorRenderable for r16float {}
impl ColorRenderable for rg16float {}
impl ColorRenderable for rgba16float {}
impl ColorRenderable for r32float {}
impl ColorRenderable for rgba32float {}
impl ColorRenderable for rgb10a2unorm {}
impl ColorRenderable for r8sint {}
impl ColorRenderable for rg8sint {}
impl ColorRenderable for rgba8sint {}
impl ColorRenderable for r16sint {}
impl ColorRenderable for rg16sint {}
impl ColorRenderable for rgba16sint {}
impl ColorRenderable for r32sint {}
impl ColorRenderable for rg32sint {}
impl ColorRenderable for rgba32sint {}
impl ColorRenderable for r8uint {}
impl ColorRenderable for rg8uint {}
impl ColorRenderable for rgba8uint {}
impl ColorRenderable for r16uint {}
impl ColorRenderable for rg16uint {}
impl ColorRenderable for rgba16uint {}
impl ColorRenderable for r32uint {}
impl ColorRenderable for rg32uint {}
impl ColorRenderable for rgba32uint {}

pub trait FloatRenderable: ColorRenderable {}

impl FloatRenderable for r8unorm {}
impl FloatRenderable for rg8unorm {}
impl FloatRenderable for rgba8unorm {}
impl FloatRenderable for rgba8unorm_srgb {}
impl FloatRenderable for bgra8unorm {}
impl FloatRenderable for bgra8unorm_srgb {}
impl FloatRenderable for r16float {}
impl FloatRenderable for rg16float {}
impl FloatRenderable for rgba16float {}
impl FloatRenderable for r32float {}
impl FloatRenderable for rgba32float {}
impl FloatRenderable for rgb10a2unorm {}

pub trait SignedIntegerRenderable: ColorRenderable {}

impl SignedIntegerRenderable for r8sint {}
impl SignedIntegerRenderable for rg8sint {}
impl SignedIntegerRenderable for rgba8sint {}
impl SignedIntegerRenderable for r16sint {}
impl SignedIntegerRenderable for rg16sint {}
impl SignedIntegerRenderable for rgba16sint {}
impl SignedIntegerRenderable for r32sint {}
impl SignedIntegerRenderable for rg32sint {}
impl SignedIntegerRenderable for rgba32sint {}

pub trait UnsignedIntegerRenderable: ColorRenderable {}

impl UnsignedIntegerRenderable for r8uint {}
impl UnsignedIntegerRenderable for rg8uint {}
impl UnsignedIntegerRenderable for rgba8uint {}
impl UnsignedIntegerRenderable for r16uint {}
impl UnsignedIntegerRenderable for rg16uint {}
impl UnsignedIntegerRenderable for rgba16uint {}
impl UnsignedIntegerRenderable for r32uint {}
impl UnsignedIntegerRenderable for rg32uint {}
impl UnsignedIntegerRenderable for rgba32uint {}

pub trait DepthStencilRenderable: Renderable {}

impl DepthStencilRenderable for stencil8 {}
impl DepthStencilRenderable for depth16unorm {}
impl DepthStencilRenderable for depth24plus {}
impl DepthStencilRenderable for depth24plus_stencil8 {}
impl DepthStencilRenderable for depth32float {}
impl DepthStencilRenderable for depth24unorm_stencil8 {}
impl DepthStencilRenderable for depth32float_stencil8 {}

pub trait CombinedDepthStencilRenderable: DepthStencilRenderable {}

impl CombinedDepthStencilRenderable for depth24plus_stencil8 {}
impl CombinedDepthStencilRenderable for depth24unorm_stencil8 {}
impl CombinedDepthStencilRenderable for depth32float_stencil8 {}

pub trait DepthRenderable: DepthStencilRenderable {}

impl DepthRenderable for depth16unorm {}
impl DepthRenderable for depth24plus {}
impl DepthRenderable for depth32float {}

pub trait StencilRenderable: DepthStencilRenderable {}

impl StencilRenderable for stencil8 {}

pub trait MultisampleFormat: TextureFormat {}

impl MultisampleFormat for r8unorm {}
impl MultisampleFormat for r8snorm {}
impl MultisampleFormat for r8uint {}
impl MultisampleFormat for r8sint {}
impl MultisampleFormat for rg8unorm {}
impl MultisampleFormat for rg8snorm {}
impl MultisampleFormat for rg8uint {}
impl MultisampleFormat for rg8sint {}
impl MultisampleFormat for rgba8unorm {}
impl MultisampleFormat for rgba8unorm_srgb {}
impl MultisampleFormat for rgba8snorm {}
impl MultisampleFormat for rgba8uint {}
impl MultisampleFormat for rgba8sint {}
impl MultisampleFormat for bgra8unorm {}
impl MultisampleFormat for bgra8unorm_srgb {}
impl MultisampleFormat for r16uint {}
impl MultisampleFormat for r16sint {}
impl MultisampleFormat for r16float {}
impl MultisampleFormat for rg16uint {}
impl MultisampleFormat for rg16sint {}
impl MultisampleFormat for rg16float {}
impl MultisampleFormat for rgba16uint {}
impl MultisampleFormat for rgba16sint {}
impl MultisampleFormat for rgba16float {}
impl MultisampleFormat for r32float {}
impl MultisampleFormat for rgb10a2unorm {}
impl MultisampleFormat for rg11b10ufloat {}
impl MultisampleFormat for stencil8 {}
impl MultisampleFormat for depth16unorm {}
impl MultisampleFormat for depth24plus {}
impl MultisampleFormat for depth24plus_stencil8 {}
impl MultisampleFormat for depth32float {}
impl MultisampleFormat for depth24unorm_stencil8 {}
impl MultisampleFormat for depth32float_stencil8 {}

pub trait MultisampleColorRenderable: MultisampleFormat {}

impl MultisampleColorRenderable for r8unorm {}
impl MultisampleColorRenderable for r8uint {}
impl MultisampleColorRenderable for r8sint {}
impl MultisampleColorRenderable for rg8unorm {}
impl MultisampleColorRenderable for rg8uint {}
impl MultisampleColorRenderable for rg8sint {}
impl MultisampleColorRenderable for rgba8unorm {}
impl MultisampleColorRenderable for rgba8unorm_srgb {}
impl MultisampleColorRenderable for rgba8uint {}
impl MultisampleColorRenderable for rgba8sint {}
impl MultisampleColorRenderable for bgra8unorm {}
impl MultisampleColorRenderable for bgra8unorm_srgb {}
impl MultisampleColorRenderable for r16uint {}
impl MultisampleColorRenderable for r16sint {}
impl MultisampleColorRenderable for r16float {}
impl MultisampleColorRenderable for rg16uint {}
impl MultisampleColorRenderable for rg16sint {}
impl MultisampleColorRenderable for rg16float {}
impl MultisampleColorRenderable for rgba16uint {}
impl MultisampleColorRenderable for rgba16sint {}
impl MultisampleColorRenderable for rgba16float {}
impl MultisampleColorRenderable for r32float {}
impl MultisampleColorRenderable for rgb10a2unorm {}

pub trait MultisampleFloatRenderable: MultisampleColorRenderable {}

impl MultisampleFloatRenderable for r8unorm {}
impl MultisampleFloatRenderable for rg8unorm {}
impl MultisampleFloatRenderable for rgba8unorm {}
impl MultisampleFloatRenderable for rgba8unorm_srgb {}
impl MultisampleFloatRenderable for bgra8unorm {}
impl MultisampleFloatRenderable for bgra8unorm_srgb {}
impl MultisampleFloatRenderable for r16float {}
impl MultisampleFloatRenderable for rg16float {}
impl MultisampleFloatRenderable for rgba16float {}
impl MultisampleFloatRenderable for r32float {}
impl MultisampleFloatRenderable for rgb10a2unorm {}

pub trait MultisampleSignedIntegerRenderable: MultisampleColorRenderable {}

impl MultisampleSignedIntegerRenderable for r8sint {}
impl MultisampleSignedIntegerRenderable for rg8sint {}
impl MultisampleSignedIntegerRenderable for rgba8sint {}
impl MultisampleSignedIntegerRenderable for r16sint {}
impl MultisampleSignedIntegerRenderable for rg16sint {}
impl MultisampleSignedIntegerRenderable for rgba16sint {}

pub trait MultisampleUnsignedIntegerRenderable: MultisampleColorRenderable {}

impl MultisampleUnsignedIntegerRenderable for r8uint {}
impl MultisampleUnsignedIntegerRenderable for rg8uint {}
impl MultisampleUnsignedIntegerRenderable for rgba8uint {}
impl MultisampleUnsignedIntegerRenderable for r16uint {}
impl MultisampleUnsignedIntegerRenderable for rg16uint {}
impl MultisampleUnsignedIntegerRenderable for rgba16uint {}

pub trait Resolvable: MultisampleColorRenderable {}

impl Resolvable for r8unorm {}
impl Resolvable for rg8unorm {}
impl Resolvable for rgba8unorm {}
impl Resolvable for rgba8unorm_srgb {}
impl Resolvable for bgra8unorm {}
impl Resolvable for bgra8unorm_srgb {}
impl Resolvable for r16float {}
impl Resolvable for rg16float {}
impl Resolvable for rgba16float {}
impl Resolvable for rgb10a2unorm {}

pub trait Blendable: ColorRenderable {}

impl Blendable for r8unorm {}
impl Blendable for rg8unorm {}
impl Blendable for rgba8unorm {}
impl Blendable for rgba8unorm_srgb {}
impl Blendable for bgra8unorm {}
impl Blendable for bgra8unorm_srgb {}
impl Blendable for r16float {}
impl Blendable for rg16float {}
impl Blendable for rgba16float {}
impl Blendable for rgb10a2unorm {}

/// Implemented from formats that can either be copied from a texture to a buffer or from a buffer
/// a texture, or both.
///
/// See [ImageCopyToBufferFormat] for the specific set of formats that can be copied from a texture
/// to a buffer.
///
/// See [ImageCopyFromBufferFormat] for the specific set of formats that can be copied from a buffer
/// to a texture.
pub trait ImageBufferDataFormat: TextureFormat {
    const BYTES_PER_BLOCK: u32;
}

macro_rules! impl_buffer_data_format {
    ($format:ident, $bytes_per_block:literal) => {
        impl ImageBufferDataFormat for $format {
            const BYTES_PER_BLOCK: u32 = $bytes_per_block;
        }
    };
}

impl_buffer_data_format!(r8unorm, 1);
impl_buffer_data_format!(r8snorm, 1);
impl_buffer_data_format!(r8uint, 1);
impl_buffer_data_format!(r8sint, 1);
impl_buffer_data_format!(r16uint, 2);
impl_buffer_data_format!(r16sint, 2);
impl_buffer_data_format!(r16float, 2);
impl_buffer_data_format!(rg8unorm, 2);
impl_buffer_data_format!(rg8snorm, 2);
impl_buffer_data_format!(rg8uint, 2);
impl_buffer_data_format!(rg8sint, 2);
impl_buffer_data_format!(r32uint, 4);
impl_buffer_data_format!(r32sint, 4);
impl_buffer_data_format!(r32float, 4);
impl_buffer_data_format!(rg16uint, 4);
impl_buffer_data_format!(rg16sint, 4);
impl_buffer_data_format!(rg16float, 4);
impl_buffer_data_format!(rgba8unorm, 4);
impl_buffer_data_format!(rgba8unorm_srgb, 4);
impl_buffer_data_format!(rgba8snorm, 4);
impl_buffer_data_format!(rgba8uint, 4);
impl_buffer_data_format!(rgba8sint, 4);
impl_buffer_data_format!(bgra8unorm, 4);
impl_buffer_data_format!(bgra8unorm_srgb, 4);
impl_buffer_data_format!(rgb9e5ufloat, 4);
impl_buffer_data_format!(rgb10a2unorm, 4);
impl_buffer_data_format!(rg11b10ufloat, 4);
impl_buffer_data_format!(rg32uint, 8);
impl_buffer_data_format!(rg32sint, 8);
impl_buffer_data_format!(rg32float, 8);
impl_buffer_data_format!(rgba16uint, 8);
impl_buffer_data_format!(rgba16sint, 8);
impl_buffer_data_format!(rgba16float, 8);
impl_buffer_data_format!(rgba32uint, 16);
impl_buffer_data_format!(rgba32sint, 16);
impl_buffer_data_format!(rgba32float, 16);
impl_buffer_data_format!(stencil8, 1);
impl_buffer_data_format!(depth16unorm, 2);
impl_buffer_data_format!(depth32float, 4);
impl_buffer_data_format!(bc1_rgba_unorm, 8);
impl_buffer_data_format!(bc1_rgba_unorm_srgb, 8);
impl_buffer_data_format!(bc2_rgba_unorm, 16);
impl_buffer_data_format!(bc2_rgba_unorm_srgb, 16);
impl_buffer_data_format!(bc3_rgba_unorm, 16);
impl_buffer_data_format!(bc3_rgba_unorm_srgb, 16);
impl_buffer_data_format!(bc4_r_unorm, 8);
impl_buffer_data_format!(bc4_r_snorm, 8);
impl_buffer_data_format!(bc5_rg_unorm, 16);
impl_buffer_data_format!(bc5_rg_snorm, 16);
impl_buffer_data_format!(bc6h_rgb_ufloat, 16);
impl_buffer_data_format!(bc6h_rgb_float, 16);
impl_buffer_data_format!(bc7_rgba_unorm, 16);
impl_buffer_data_format!(bc7_rgba_unorm_srgb, 16);
impl_buffer_data_format!(etc2_rgb8unorm, 8);
impl_buffer_data_format!(etc2_rgb8unorm_srgb, 8);
impl_buffer_data_format!(etc2_rgb8a1unorm, 8);
impl_buffer_data_format!(etc2_rgb8a1unorm_srgb, 8);
impl_buffer_data_format!(etc2_rgba8unorm, 16);
impl_buffer_data_format!(etc2_rgba8unorm_srgb, 16);
impl_buffer_data_format!(eac_r11unorm, 8);
impl_buffer_data_format!(eac_r11snorm, 8);
impl_buffer_data_format!(eac_rg11unorm, 16);
impl_buffer_data_format!(eac_rg11snorm, 16);
impl_buffer_data_format!(astc_4x4_unorm, 16);
impl_buffer_data_format!(astc_4x4_unorm_srgb, 16);
impl_buffer_data_format!(astc_5x4_unorm, 16);
impl_buffer_data_format!(astc_5x4_unorm_srgb, 16);
impl_buffer_data_format!(astc_5x5_unorm, 16);
impl_buffer_data_format!(astc_5x5_unorm_srgb, 16);
impl_buffer_data_format!(astc_6x5_unorm, 16);
impl_buffer_data_format!(astc_6x5_unorm_srgb, 16);
impl_buffer_data_format!(astc_6x6_unorm, 16);
impl_buffer_data_format!(astc_6x6_unorm_srgb, 16);
impl_buffer_data_format!(astc_8x5_unorm, 16);
impl_buffer_data_format!(astc_8x5_unorm_srgb, 16);
impl_buffer_data_format!(astc_8x6_unorm, 16);
impl_buffer_data_format!(astc_8x6_unorm_srgb, 16);
impl_buffer_data_format!(astc_8x8_unorm, 16);
impl_buffer_data_format!(astc_8x8_unorm_srgb, 16);
impl_buffer_data_format!(astc_10x5_unorm, 16);
impl_buffer_data_format!(astc_10x5_unorm_srgb, 16);
impl_buffer_data_format!(astc_10x6_unorm, 16);
impl_buffer_data_format!(astc_10x6_unorm_srgb, 16);
impl_buffer_data_format!(astc_10x8_unorm, 16);
impl_buffer_data_format!(astc_10x8_unorm_srgb, 16);
impl_buffer_data_format!(astc_10x10_unorm, 16);
impl_buffer_data_format!(astc_10x10_unorm_srgb, 16);
impl_buffer_data_format!(astc_12x10_unorm, 16);
impl_buffer_data_format!(astc_12x10_unorm_srgb, 16);
impl_buffer_data_format!(astc_12x12_unorm, 16);
impl_buffer_data_format!(astc_12x12_unorm_srgb, 16);

/// Marker trait for types that can be copied from a texture to a buffer.
pub trait ImageCopyToBufferFormat: ImageBufferDataFormat {}

impl ImageCopyToBufferFormat for r8unorm {}
impl ImageCopyToBufferFormat for r8snorm {}
impl ImageCopyToBufferFormat for r8uint {}
impl ImageCopyToBufferFormat for r8sint {}
impl ImageCopyToBufferFormat for r16uint {}
impl ImageCopyToBufferFormat for r16sint {}
impl ImageCopyToBufferFormat for r16float {}
impl ImageCopyToBufferFormat for rg8unorm {}
impl ImageCopyToBufferFormat for rg8snorm {}
impl ImageCopyToBufferFormat for rg8uint {}
impl ImageCopyToBufferFormat for rg8sint {}
impl ImageCopyToBufferFormat for r32uint {}
impl ImageCopyToBufferFormat for r32sint {}
impl ImageCopyToBufferFormat for r32float {}
impl ImageCopyToBufferFormat for rg16uint {}
impl ImageCopyToBufferFormat for rg16sint {}
impl ImageCopyToBufferFormat for rg16float {}
impl ImageCopyToBufferFormat for rgba8unorm {}
impl ImageCopyToBufferFormat for rgba8unorm_srgb {}
impl ImageCopyToBufferFormat for rgba8snorm {}
impl ImageCopyToBufferFormat for rgba8uint {}
impl ImageCopyToBufferFormat for rgba8sint {}
impl ImageCopyToBufferFormat for bgra8unorm {}
impl ImageCopyToBufferFormat for bgra8unorm_srgb {}
impl ImageCopyToBufferFormat for rgb9e5ufloat {}
impl ImageCopyToBufferFormat for rgb10a2unorm {}
impl ImageCopyToBufferFormat for rg11b10ufloat {}
impl ImageCopyToBufferFormat for rg32uint {}
impl ImageCopyToBufferFormat for rg32sint {}
impl ImageCopyToBufferFormat for rg32float {}
impl ImageCopyToBufferFormat for rgba16uint {}
impl ImageCopyToBufferFormat for rgba16sint {}
impl ImageCopyToBufferFormat for rgba16float {}
impl ImageCopyToBufferFormat for rgba32uint {}
impl ImageCopyToBufferFormat for rgba32sint {}
impl ImageCopyToBufferFormat for rgba32float {}
impl ImageCopyToBufferFormat for stencil8 {}
impl ImageCopyToBufferFormat for depth16unorm {}
impl ImageCopyToBufferFormat for depth32float {}
impl ImageCopyToBufferFormat for bc1_rgba_unorm {}
impl ImageCopyToBufferFormat for bc1_rgba_unorm_srgb {}
impl ImageCopyToBufferFormat for bc2_rgba_unorm {}
impl ImageCopyToBufferFormat for bc2_rgba_unorm_srgb {}
impl ImageCopyToBufferFormat for bc3_rgba_unorm {}
impl ImageCopyToBufferFormat for bc3_rgba_unorm_srgb {}
impl ImageCopyToBufferFormat for bc4_r_unorm {}
impl ImageCopyToBufferFormat for bc4_r_snorm {}
impl ImageCopyToBufferFormat for bc5_rg_unorm {}
impl ImageCopyToBufferFormat for bc5_rg_snorm {}
impl ImageCopyToBufferFormat for bc6h_rgb_ufloat {}
impl ImageCopyToBufferFormat for bc6h_rgb_float {}
impl ImageCopyToBufferFormat for bc7_rgba_unorm {}
impl ImageCopyToBufferFormat for bc7_rgba_unorm_srgb {}
impl ImageCopyToBufferFormat for etc2_rgb8unorm {}
impl ImageCopyToBufferFormat for etc2_rgb8unorm_srgb {}
impl ImageCopyToBufferFormat for etc2_rgb8a1unorm {}
impl ImageCopyToBufferFormat for etc2_rgb8a1unorm_srgb {}
impl ImageCopyToBufferFormat for etc2_rgba8unorm {}
impl ImageCopyToBufferFormat for etc2_rgba8unorm_srgb {}
impl ImageCopyToBufferFormat for eac_r11unorm {}
impl ImageCopyToBufferFormat for eac_r11snorm {}
impl ImageCopyToBufferFormat for eac_rg11unorm {}
impl ImageCopyToBufferFormat for eac_rg11snorm {}
impl ImageCopyToBufferFormat for astc_4x4_unorm {}
impl ImageCopyToBufferFormat for astc_4x4_unorm_srgb {}
impl ImageCopyToBufferFormat for astc_5x4_unorm {}
impl ImageCopyToBufferFormat for astc_5x4_unorm_srgb {}
impl ImageCopyToBufferFormat for astc_5x5_unorm {}
impl ImageCopyToBufferFormat for astc_5x5_unorm_srgb {}
impl ImageCopyToBufferFormat for astc_6x5_unorm {}
impl ImageCopyToBufferFormat for astc_6x5_unorm_srgb {}
impl ImageCopyToBufferFormat for astc_6x6_unorm {}
impl ImageCopyToBufferFormat for astc_6x6_unorm_srgb {}
impl ImageCopyToBufferFormat for astc_8x5_unorm {}
impl ImageCopyToBufferFormat for astc_8x5_unorm_srgb {}
impl ImageCopyToBufferFormat for astc_8x6_unorm {}
impl ImageCopyToBufferFormat for astc_8x6_unorm_srgb {}
impl ImageCopyToBufferFormat for astc_8x8_unorm {}
impl ImageCopyToBufferFormat for astc_8x8_unorm_srgb {}
impl ImageCopyToBufferFormat for astc_10x5_unorm {}
impl ImageCopyToBufferFormat for astc_10x5_unorm_srgb {}
impl ImageCopyToBufferFormat for astc_10x6_unorm {}
impl ImageCopyToBufferFormat for astc_10x6_unorm_srgb {}
impl ImageCopyToBufferFormat for astc_10x8_unorm {}
impl ImageCopyToBufferFormat for astc_10x8_unorm_srgb {}
impl ImageCopyToBufferFormat for astc_10x10_unorm {}
impl ImageCopyToBufferFormat for astc_10x10_unorm_srgb {}
impl ImageCopyToBufferFormat for astc_12x10_unorm {}
impl ImageCopyToBufferFormat for astc_12x10_unorm_srgb {}
impl ImageCopyToBufferFormat for astc_12x12_unorm {}
impl ImageCopyToBufferFormat for astc_12x12_unorm_srgb {}

/// Marker trait for formats that can be copied from a buffer to a texture.
pub trait ImageCopyFromBufferFormat: ImageBufferDataFormat {}

impl ImageCopyFromBufferFormat for r8unorm {}
impl ImageCopyFromBufferFormat for r8snorm {}
impl ImageCopyFromBufferFormat for r8uint {}
impl ImageCopyFromBufferFormat for r8sint {}
impl ImageCopyFromBufferFormat for r16uint {}
impl ImageCopyFromBufferFormat for r16sint {}
impl ImageCopyFromBufferFormat for r16float {}
impl ImageCopyFromBufferFormat for rg8unorm {}
impl ImageCopyFromBufferFormat for rg8snorm {}
impl ImageCopyFromBufferFormat for rg8uint {}
impl ImageCopyFromBufferFormat for rg8sint {}
impl ImageCopyFromBufferFormat for r32uint {}
impl ImageCopyFromBufferFormat for r32sint {}
impl ImageCopyFromBufferFormat for r32float {}
impl ImageCopyFromBufferFormat for rg16uint {}
impl ImageCopyFromBufferFormat for rg16sint {}
impl ImageCopyFromBufferFormat for rg16float {}
impl ImageCopyFromBufferFormat for rgba8unorm {}
impl ImageCopyFromBufferFormat for rgba8unorm_srgb {}
impl ImageCopyFromBufferFormat for rgba8snorm {}
impl ImageCopyFromBufferFormat for rgba8uint {}
impl ImageCopyFromBufferFormat for rgba8sint {}
impl ImageCopyFromBufferFormat for bgra8unorm {}
impl ImageCopyFromBufferFormat for bgra8unorm_srgb {}
impl ImageCopyFromBufferFormat for rgb9e5ufloat {}
impl ImageCopyFromBufferFormat for rgb10a2unorm {}
impl ImageCopyFromBufferFormat for rg11b10ufloat {}
impl ImageCopyFromBufferFormat for rg32uint {}
impl ImageCopyFromBufferFormat for rg32sint {}
impl ImageCopyFromBufferFormat for rg32float {}
impl ImageCopyFromBufferFormat for rgba16uint {}
impl ImageCopyFromBufferFormat for rgba16sint {}
impl ImageCopyFromBufferFormat for rgba16float {}
impl ImageCopyFromBufferFormat for rgba32uint {}
impl ImageCopyFromBufferFormat for rgba32sint {}
impl ImageCopyFromBufferFormat for rgba32float {}
impl ImageCopyFromBufferFormat for stencil8 {}
impl ImageCopyFromBufferFormat for depth16unorm {}
impl ImageCopyFromBufferFormat for bc1_rgba_unorm {}
impl ImageCopyFromBufferFormat for bc1_rgba_unorm_srgb {}
impl ImageCopyFromBufferFormat for bc2_rgba_unorm {}
impl ImageCopyFromBufferFormat for bc2_rgba_unorm_srgb {}
impl ImageCopyFromBufferFormat for bc3_rgba_unorm {}
impl ImageCopyFromBufferFormat for bc3_rgba_unorm_srgb {}
impl ImageCopyFromBufferFormat for bc4_r_unorm {}
impl ImageCopyFromBufferFormat for bc4_r_snorm {}
impl ImageCopyFromBufferFormat for bc5_rg_unorm {}
impl ImageCopyFromBufferFormat for bc5_rg_snorm {}
impl ImageCopyFromBufferFormat for bc6h_rgb_ufloat {}
impl ImageCopyFromBufferFormat for bc6h_rgb_float {}
impl ImageCopyFromBufferFormat for bc7_rgba_unorm {}
impl ImageCopyFromBufferFormat for bc7_rgba_unorm_srgb {}
impl ImageCopyFromBufferFormat for etc2_rgb8unorm {}
impl ImageCopyFromBufferFormat for etc2_rgb8unorm_srgb {}
impl ImageCopyFromBufferFormat for etc2_rgb8a1unorm {}
impl ImageCopyFromBufferFormat for etc2_rgb8a1unorm_srgb {}
impl ImageCopyFromBufferFormat for etc2_rgba8unorm {}
impl ImageCopyFromBufferFormat for etc2_rgba8unorm_srgb {}
impl ImageCopyFromBufferFormat for eac_r11unorm {}
impl ImageCopyFromBufferFormat for eac_r11snorm {}
impl ImageCopyFromBufferFormat for eac_rg11unorm {}
impl ImageCopyFromBufferFormat for eac_rg11snorm {}
impl ImageCopyFromBufferFormat for astc_4x4_unorm {}
impl ImageCopyFromBufferFormat for astc_4x4_unorm_srgb {}
impl ImageCopyFromBufferFormat for astc_5x4_unorm {}
impl ImageCopyFromBufferFormat for astc_5x4_unorm_srgb {}
impl ImageCopyFromBufferFormat for astc_5x5_unorm {}
impl ImageCopyFromBufferFormat for astc_5x5_unorm_srgb {}
impl ImageCopyFromBufferFormat for astc_6x5_unorm {}
impl ImageCopyFromBufferFormat for astc_6x5_unorm_srgb {}
impl ImageCopyFromBufferFormat for astc_6x6_unorm {}
impl ImageCopyFromBufferFormat for astc_6x6_unorm_srgb {}
impl ImageCopyFromBufferFormat for astc_8x5_unorm {}
impl ImageCopyFromBufferFormat for astc_8x5_unorm_srgb {}
impl ImageCopyFromBufferFormat for astc_8x6_unorm {}
impl ImageCopyFromBufferFormat for astc_8x6_unorm_srgb {}
impl ImageCopyFromBufferFormat for astc_8x8_unorm {}
impl ImageCopyFromBufferFormat for astc_8x8_unorm_srgb {}
impl ImageCopyFromBufferFormat for astc_10x5_unorm {}
impl ImageCopyFromBufferFormat for astc_10x5_unorm_srgb {}
impl ImageCopyFromBufferFormat for astc_10x6_unorm {}
impl ImageCopyFromBufferFormat for astc_10x6_unorm_srgb {}
impl ImageCopyFromBufferFormat for astc_10x8_unorm {}
impl ImageCopyFromBufferFormat for astc_10x8_unorm_srgb {}
impl ImageCopyFromBufferFormat for astc_10x10_unorm {}
impl ImageCopyFromBufferFormat for astc_10x10_unorm_srgb {}
impl ImageCopyFromBufferFormat for astc_12x10_unorm {}
impl ImageCopyFromBufferFormat for astc_12x10_unorm_srgb {}
impl ImageCopyFromBufferFormat for astc_12x12_unorm {}
impl ImageCopyFromBufferFormat for astc_12x12_unorm_srgb {}

/// Marker trait for formats that can be copied from texture to texture.
pub trait ImageCopyTextureFormat: TextureFormat {}

impl ImageCopyTextureFormat for r8unorm {}
impl ImageCopyTextureFormat for r8snorm {}
impl ImageCopyTextureFormat for r8uint {}
impl ImageCopyTextureFormat for r8sint {}
impl ImageCopyTextureFormat for r16uint {}
impl ImageCopyTextureFormat for r16sint {}
impl ImageCopyTextureFormat for r16float {}
impl ImageCopyTextureFormat for rg8unorm {}
impl ImageCopyTextureFormat for rg8snorm {}
impl ImageCopyTextureFormat for rg8uint {}
impl ImageCopyTextureFormat for rg8sint {}
impl ImageCopyTextureFormat for r32uint {}
impl ImageCopyTextureFormat for r32sint {}
impl ImageCopyTextureFormat for r32float {}
impl ImageCopyTextureFormat for rg16uint {}
impl ImageCopyTextureFormat for rg16sint {}
impl ImageCopyTextureFormat for rg16float {}
impl ImageCopyTextureFormat for rgba8unorm {}
impl ImageCopyTextureFormat for rgba8unorm_srgb {}
impl ImageCopyTextureFormat for rgba8snorm {}
impl ImageCopyTextureFormat for rgba8uint {}
impl ImageCopyTextureFormat for rgba8sint {}
impl ImageCopyTextureFormat for bgra8unorm {}
impl ImageCopyTextureFormat for bgra8unorm_srgb {}
impl ImageCopyTextureFormat for rgb9e5ufloat {}
impl ImageCopyTextureFormat for rgb10a2unorm {}
impl ImageCopyTextureFormat for rg11b10ufloat {}
impl ImageCopyTextureFormat for rg32uint {}
impl ImageCopyTextureFormat for rg32sint {}
impl ImageCopyTextureFormat for rg32float {}
impl ImageCopyTextureFormat for rgba16uint {}
impl ImageCopyTextureFormat for rgba16sint {}
impl ImageCopyTextureFormat for rgba16float {}
impl ImageCopyTextureFormat for rgba32uint {}
impl ImageCopyTextureFormat for rgba32sint {}
impl ImageCopyTextureFormat for rgba32float {}
impl ImageCopyTextureFormat for stencil8 {}
impl ImageCopyTextureFormat for depth16unorm {}
impl ImageCopyTextureFormat for depth32float {}
impl ImageCopyTextureFormat for depth32float_stencil8 {}
impl ImageCopyTextureFormat for bc1_rgba_unorm {}
impl ImageCopyTextureFormat for bc1_rgba_unorm_srgb {}
impl ImageCopyTextureFormat for bc2_rgba_unorm {}
impl ImageCopyTextureFormat for bc2_rgba_unorm_srgb {}
impl ImageCopyTextureFormat for bc3_rgba_unorm {}
impl ImageCopyTextureFormat for bc3_rgba_unorm_srgb {}
impl ImageCopyTextureFormat for bc4_r_unorm {}
impl ImageCopyTextureFormat for bc4_r_snorm {}
impl ImageCopyTextureFormat for bc5_rg_unorm {}
impl ImageCopyTextureFormat for bc5_rg_snorm {}
impl ImageCopyTextureFormat for bc6h_rgb_ufloat {}
impl ImageCopyTextureFormat for bc6h_rgb_float {}
impl ImageCopyTextureFormat for bc7_rgba_unorm {}
impl ImageCopyTextureFormat for bc7_rgba_unorm_srgb {}
impl ImageCopyTextureFormat for etc2_rgb8unorm {}
impl ImageCopyTextureFormat for etc2_rgb8unorm_srgb {}
impl ImageCopyTextureFormat for etc2_rgb8a1unorm {}
impl ImageCopyTextureFormat for etc2_rgb8a1unorm_srgb {}
impl ImageCopyTextureFormat for etc2_rgba8unorm {}
impl ImageCopyTextureFormat for etc2_rgba8unorm_srgb {}
impl ImageCopyTextureFormat for eac_r11unorm {}
impl ImageCopyTextureFormat for eac_r11snorm {}
impl ImageCopyTextureFormat for eac_rg11unorm {}
impl ImageCopyTextureFormat for eac_rg11snorm {}
impl ImageCopyTextureFormat for astc_4x4_unorm {}
impl ImageCopyTextureFormat for astc_4x4_unorm_srgb {}
impl ImageCopyTextureFormat for astc_5x4_unorm {}
impl ImageCopyTextureFormat for astc_5x4_unorm_srgb {}
impl ImageCopyTextureFormat for astc_5x5_unorm {}
impl ImageCopyTextureFormat for astc_5x5_unorm_srgb {}
impl ImageCopyTextureFormat for astc_6x5_unorm {}
impl ImageCopyTextureFormat for astc_6x5_unorm_srgb {}
impl ImageCopyTextureFormat for astc_6x6_unorm {}
impl ImageCopyTextureFormat for astc_6x6_unorm_srgb {}
impl ImageCopyTextureFormat for astc_8x5_unorm {}
impl ImageCopyTextureFormat for astc_8x5_unorm_srgb {}
impl ImageCopyTextureFormat for astc_8x6_unorm {}
impl ImageCopyTextureFormat for astc_8x6_unorm_srgb {}
impl ImageCopyTextureFormat for astc_8x8_unorm {}
impl ImageCopyTextureFormat for astc_8x8_unorm_srgb {}
impl ImageCopyTextureFormat for astc_10x5_unorm {}
impl ImageCopyTextureFormat for astc_10x5_unorm_srgb {}
impl ImageCopyTextureFormat for astc_10x6_unorm {}
impl ImageCopyTextureFormat for astc_10x6_unorm_srgb {}
impl ImageCopyTextureFormat for astc_10x8_unorm {}
impl ImageCopyTextureFormat for astc_10x8_unorm_srgb {}
impl ImageCopyTextureFormat for astc_10x10_unorm {}
impl ImageCopyTextureFormat for astc_10x10_unorm_srgb {}
impl ImageCopyTextureFormat for astc_12x10_unorm {}
impl ImageCopyTextureFormat for astc_12x10_unorm_srgb {}
impl ImageCopyTextureFormat for astc_12x12_unorm {}
impl ImageCopyTextureFormat for astc_12x12_unorm_srgb {}

/// Marker trait for formats that can be used in sub-image copy operations.
///
/// Not all copyable formats (see [ImageCopyToBufferFormat], [ImageCopyFromBufferFormat],
/// [ImageCopyTextureFormat]) implement this trait. Specifically, the depth-stencil formats are
/// cannot be used in sub-image copy operations.
pub trait SubImageCopyFormat: TextureFormat {}

impl SubImageCopyFormat for r8unorm {}
impl SubImageCopyFormat for r8snorm {}
impl SubImageCopyFormat for r8uint {}
impl SubImageCopyFormat for r8sint {}
impl SubImageCopyFormat for r16uint {}
impl SubImageCopyFormat for r16sint {}
impl SubImageCopyFormat for r16float {}
impl SubImageCopyFormat for rg8unorm {}
impl SubImageCopyFormat for rg8snorm {}
impl SubImageCopyFormat for rg8uint {}
impl SubImageCopyFormat for rg8sint {}
impl SubImageCopyFormat for r32uint {}
impl SubImageCopyFormat for r32sint {}
impl SubImageCopyFormat for r32float {}
impl SubImageCopyFormat for rg16uint {}
impl SubImageCopyFormat for rg16sint {}
impl SubImageCopyFormat for rg16float {}
impl SubImageCopyFormat for rgba8unorm {}
impl SubImageCopyFormat for rgba8unorm_srgb {}
impl SubImageCopyFormat for rgba8snorm {}
impl SubImageCopyFormat for rgba8uint {}
impl SubImageCopyFormat for rgba8sint {}
impl SubImageCopyFormat for bgra8unorm {}
impl SubImageCopyFormat for bgra8unorm_srgb {}
impl SubImageCopyFormat for rgb9e5ufloat {}
impl SubImageCopyFormat for rgb10a2unorm {}
impl SubImageCopyFormat for rg11b10ufloat {}
impl SubImageCopyFormat for rg32uint {}
impl SubImageCopyFormat for rg32sint {}
impl SubImageCopyFormat for rg32float {}
impl SubImageCopyFormat for rgba16uint {}
impl SubImageCopyFormat for rgba16sint {}
impl SubImageCopyFormat for rgba16float {}
impl SubImageCopyFormat for rgba32uint {}
impl SubImageCopyFormat for rgba32sint {}
impl SubImageCopyFormat for rgba32float {}
impl SubImageCopyFormat for bc1_rgba_unorm {}
impl SubImageCopyFormat for bc1_rgba_unorm_srgb {}
impl SubImageCopyFormat for bc2_rgba_unorm {}
impl SubImageCopyFormat for bc2_rgba_unorm_srgb {}
impl SubImageCopyFormat for bc3_rgba_unorm {}
impl SubImageCopyFormat for bc3_rgba_unorm_srgb {}
impl SubImageCopyFormat for bc4_r_unorm {}
impl SubImageCopyFormat for bc4_r_snorm {}
impl SubImageCopyFormat for bc5_rg_unorm {}
impl SubImageCopyFormat for bc5_rg_snorm {}
impl SubImageCopyFormat for bc6h_rgb_ufloat {}
impl SubImageCopyFormat for bc6h_rgb_float {}
impl SubImageCopyFormat for bc7_rgba_unorm {}
impl SubImageCopyFormat for bc7_rgba_unorm_srgb {}
impl SubImageCopyFormat for etc2_rgb8unorm {}
impl SubImageCopyFormat for etc2_rgb8unorm_srgb {}
impl SubImageCopyFormat for etc2_rgb8a1unorm {}
impl SubImageCopyFormat for etc2_rgb8a1unorm_srgb {}
impl SubImageCopyFormat for etc2_rgba8unorm {}
impl SubImageCopyFormat for etc2_rgba8unorm_srgb {}
impl SubImageCopyFormat for eac_r11unorm {}
impl SubImageCopyFormat for eac_r11snorm {}
impl SubImageCopyFormat for eac_rg11unorm {}
impl SubImageCopyFormat for eac_rg11snorm {}
impl SubImageCopyFormat for astc_4x4_unorm {}
impl SubImageCopyFormat for astc_4x4_unorm_srgb {}
impl SubImageCopyFormat for astc_5x4_unorm {}
impl SubImageCopyFormat for astc_5x4_unorm_srgb {}
impl SubImageCopyFormat for astc_5x5_unorm {}
impl SubImageCopyFormat for astc_5x5_unorm_srgb {}
impl SubImageCopyFormat for astc_6x5_unorm {}
impl SubImageCopyFormat for astc_6x5_unorm_srgb {}
impl SubImageCopyFormat for astc_6x6_unorm {}
impl SubImageCopyFormat for astc_6x6_unorm_srgb {}
impl SubImageCopyFormat for astc_8x5_unorm {}
impl SubImageCopyFormat for astc_8x5_unorm_srgb {}
impl SubImageCopyFormat for astc_8x6_unorm {}
impl SubImageCopyFormat for astc_8x6_unorm_srgb {}
impl SubImageCopyFormat for astc_8x8_unorm {}
impl SubImageCopyFormat for astc_8x8_unorm_srgb {}
impl SubImageCopyFormat for astc_10x5_unorm {}
impl SubImageCopyFormat for astc_10x5_unorm_srgb {}
impl SubImageCopyFormat for astc_10x6_unorm {}
impl SubImageCopyFormat for astc_10x6_unorm_srgb {}
impl SubImageCopyFormat for astc_10x8_unorm {}
impl SubImageCopyFormat for astc_10x8_unorm_srgb {}
impl SubImageCopyFormat for astc_10x10_unorm {}
impl SubImageCopyFormat for astc_10x10_unorm_srgb {}
impl SubImageCopyFormat for astc_12x10_unorm {}
impl SubImageCopyFormat for astc_12x10_unorm_srgb {}
impl SubImageCopyFormat for astc_12x12_unorm {}
impl SubImageCopyFormat for astc_12x12_unorm_srgb {}

mod view_formats_seal {
    pub trait Seal {}
}

pub trait ViewFormats<F>: view_formats_seal::Seal {}

impl<F> view_formats_seal::Seal for F where F: TextureFormat {}
impl<F> ViewFormats<F> for F where F: TextureFormat {}

mod view_format_seal {
    pub trait Seal {}
}

pub trait ViewFormat<V>: TextureFormat {}

impl<F> ViewFormat<F> for F where F: TextureFormat {}

pub unsafe trait ImageData<F>
where
    F: TextureFormat,
{
}

unsafe impl ImageData<r8unorm> for u8 {}
unsafe impl ImageData<r8snorm> for i8 {}
unsafe impl ImageData<r8uint> for u8 {}
unsafe impl ImageData<r8sint> for i8 {}
unsafe impl ImageData<r16uint> for u16 {}
unsafe impl ImageData<r16sint> for i16 {}
unsafe impl ImageData<rg8unorm> for [u8; 2] {}
unsafe impl ImageData<rg8snorm> for [i8; 2] {}
unsafe impl ImageData<rg8uint> for [u8; 2] {}
unsafe impl ImageData<rg8sint> for [i8; 2] {}
unsafe impl ImageData<r32uint> for u32 {}
unsafe impl ImageData<r32sint> for i32 {}
unsafe impl ImageData<r32float> for f32 {}
unsafe impl ImageData<rg16uint> for [u16; 2] {}
unsafe impl ImageData<rg16sint> for [i16; 2] {}
unsafe impl ImageData<rgba8unorm> for [u8; 4] {}
unsafe impl ImageData<rgba8unorm_srgb> for [u8; 4] {}
unsafe impl ImageData<rgba8snorm> for [i8; 4] {}
unsafe impl ImageData<rgba8uint> for [u8; 4] {}
unsafe impl ImageData<rgba8sint> for [i8; 4] {}
unsafe impl ImageData<bgra8unorm> for [u8; 4] {}
unsafe impl ImageData<bgra8unorm_srgb> for [u8; 4] {}
unsafe impl ImageData<rg32uint> for [u32; 2] {}
unsafe impl ImageData<rg32sint> for [i32; 2] {}
unsafe impl ImageData<rg32float> for [f32; 2] {}
unsafe impl ImageData<rgba16uint> for [u16; 4] {}
unsafe impl ImageData<rgba16sint> for [i16; 4] {}
unsafe impl ImageData<rgba32uint> for [u32; 4] {}
unsafe impl ImageData<rgba32sint> for [i32; 4] {}
unsafe impl ImageData<rgba32float> for [f32; 4] {}
unsafe impl ImageData<stencil8> for u8 {}
unsafe impl ImageData<depth16unorm> for u16 {}
unsafe impl ImageData<depth32float> for f32 {}
