use std::cmp::max;
use std::marker;
use std::ops::Rem;

use arrayvec::ArrayVec;

use crate::device::Device;
use crate::driver;
use crate::driver::{
    Device as _, Driver, Dvr, Texture, TextureAspect, TextureDescriptor, TextureDimensions,
    TextureViewDescriptor, TextureViewDimension,
};
use crate::texture::format::{
    DepthSamplable, DepthStencilFormat, FloatSamplable, ImageBufferDataFormat,
    ImageCopyFromBufferFormat, ImageCopyTextureFormat, ImageCopyToBufferFormat, Renderable,
    SignedIntegerSamplable, Storable, SubImageCopyFormat, Texture2DFormat, TextureFormat,
    TextureFormatId, UnfilteredFloatSamplable, UnsignedIntegerSamplable, ViewFormat, ViewFormats,
};
use crate::texture::{
    CopyDst, CopySrc, FormatKind, ImageCopyDst, ImageCopyFromTextureDst, ImageCopySrc,
    ImageCopyTexture, ImageCopyToTextureSrc, MipmapLevels, RenderAttachment, StorageBinding,
    SubImageCopyDst, SubImageCopyFromTextureDst, SubImageCopySrc, SubImageCopyToTextureSrc,
    TextureBinding, UnsupportedViewFormat, UsageFlags,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Texture2DDescriptor<F, U, V>
where
    F: Texture2DFormat,
    U: UsageFlags,
    V: ViewFormats<F>,
{
    pub format: F,
    pub usage: U,
    pub view_formats: V,
    pub width: u32,
    pub height: u32,
    pub layers: u32,
    pub mipmap_levels: MipmapLevels,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct View2DDescriptor {
    pub layer: u32,
    pub base_mipmap_level: u8,
    pub mipmap_level_count: Option<u8>,
}

impl Default for View2DDescriptor {
    fn default() -> Self {
        View2DDescriptor {
            layer: 0,
            base_mipmap_level: 0,
            mipmap_level_count: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct View2DArrayDescriptor {
    pub base_layer: u32,
    pub layer_count: Option<u32>,
    pub base_mipmap_level: u8,
    pub mipmap_level_count: Option<u8>,
}

impl Default for View2DArrayDescriptor {
    fn default() -> Self {
        View2DArrayDescriptor {
            base_layer: 0,
            layer_count: None,
            base_mipmap_level: 0,
            mipmap_level_count: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ViewCubeDescriptor {
    pub base_layer: u32,
    pub base_mipmap_level: u8,
    pub mipmap_level_count: Option<u8>,
}

impl Default for ViewCubeDescriptor {
    fn default() -> Self {
        ViewCubeDescriptor {
            base_layer: 0,
            base_mipmap_level: 0,
            mipmap_level_count: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ViewCubeArrayDescriptor {
    pub base_layer: u32,
    pub cube_count: Option<u32>,
    pub base_mipmap_level: u8,
    pub mipmap_level_count: Option<u8>,
}

impl Default for ViewCubeArrayDescriptor {
    fn default() -> Self {
        ViewCubeArrayDescriptor {
            base_layer: 0,
            cube_count: None,
            base_mipmap_level: 0,
            mipmap_level_count: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Storage2DDescriptor {
    pub layer: u32,
    pub mipmap_level: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Storage2DArrayDescriptor {
    pub base_layer: u32,
    pub layer_count: u32,
    pub mipmap_level: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AttachableImageDescriptor {
    pub layer: u32,
    pub mipmap_level: u8,
}

impl Default for AttachableImageDescriptor {
    fn default() -> Self {
        AttachableImageDescriptor {
            layer: 0,
            mipmap_level: 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SubImageCopy2DDescriptor {
    pub mipmap_level: u8,
    pub origin_x: u32,
    pub origin_y: u32,
    pub origin_layer: u32,
}

pub struct Texture2D<F, Usage> {
    pub(crate) handle: <Dvr as Driver>::TextureHandle,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) layers: u32,
    pub(crate) mip_level_count: u8,
    view_formats: ArrayVec<TextureFormatId, 8>,
    usage: Usage,
    _format: FormatKind<F>,
}

impl<F, U> Texture2D<F, U>
where
    F: TextureFormat,
    U: UsageFlags,
{
    pub(crate) fn from_swap_chain_texture(
        handle: <Dvr as Driver>::TextureHandle,
        width: u32,
        height: u32,
        view_formats: &[TextureFormatId],
        usage: U,
    ) -> Self {
        let view_formats = view_formats.iter().copied().collect();

        Texture2D {
            handle,
            width,
            height,
            layers: 1,
            mip_level_count: 1,
            view_formats,
            usage,
            _format: FormatKind::Typed(Default::default()),
        }
    }
}

impl<F, U> Texture2D<F, U>
where
    F: Texture2DFormat,
    U: UsageFlags,
{
    pub(crate) fn new<V: ViewFormats<F>>(
        device: &Device,
        descriptor: &Texture2DDescriptor<F, U, V>,
    ) -> Self {
        let Texture2DDescriptor {
            view_formats,
            width,
            height,
            layers,
            mipmap_levels,
            usage,
            ..
        } = descriptor;

        assert!(*width > 0, "width must be greater than `0`");
        assert!(*height > 0, "height must be greater than `0`");
        assert!(*layers > 0, "must have at least one layer");

        let [block_width, block_height] = F::BLOCK_SIZE;

        assert!(
            width.rem(block_width) == 0,
            "width must be a multiple of the format's block width (`{}`)",
            block_width
        );
        assert!(
            height.rem(block_height) == 0,
            "height must be a multiple of the format's block height (`{}`)",
            block_height
        );

        let mip_level_count = mipmap_levels.to_u32(max(*width, *height));
        let view_formats = view_formats.formats().collect::<ArrayVec<_, 8>>();

        let handle = device.device_handle.create_texture(&TextureDescriptor {
            size: (*width, *height, *layers),
            mipmap_levels: mip_level_count,
            sample_count: 1,
            dimensions: TextureDimensions::Two,
            format: F::FORMAT_ID,
            usage_flags: U::FLAG_SET,
            view_formats: view_formats.as_slice(),
        });

        Texture2D {
            handle,
            width: *width,
            height: *height,
            layers: *layers,
            mip_level_count: mip_level_count as u8,
            view_formats,
            usage: *usage,
            _format: FormatKind::Typed(Default::default()),
        }
    }
}

impl<F, U> Texture2D<F, U>
where
    U: UsageFlags,
{
    pub fn usage(&self) -> U {
        self.usage
    }
}

impl<F, U> Texture2D<F, U> {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn layers(&self) -> u32 {
        self.layers
    }

    pub fn levels(&self) -> u8 {
        self.mip_level_count
    }

    fn view_2d_internal<'a>(
        &'a self,
        format: TextureFormatId,
        descriptor: &View2DDescriptor,
    ) -> <Dvr as Driver>::TextureView {
        let View2DDescriptor {
            layer,
            base_mipmap_level,
            mipmap_level_count,
        } = *descriptor;

        assert!(layer < self.layers, "`layer` out of bounds");
        assert!(
            base_mipmap_level < self.mip_level_count,
            "`base_mipmap_level` must not exceed the texture's mipmap level count"
        );

        let mipmap_level_count = if let Some(mipmap_level_count) = mipmap_level_count {
            assert!(
                base_mipmap_level + mipmap_level_count <= self.mip_level_count,
                "`base_mipmap_level + mip_level_count` must not exceed the texture's mipmap \
                    level count"
            );

            mipmap_level_count
        } else {
            self.mip_level_count - base_mipmap_level
        };

        let layers_start = layer;
        let layers_end = layers_start + 1;

        let mip_levels_start = base_mipmap_level as u32;
        let mip_levels_end = mip_levels_start + mipmap_level_count as u32;

        self.handle.texture_view(&TextureViewDescriptor {
            format,
            dimensions: TextureViewDimension::Two,
            aspect: TextureAspect::All,
            mip_levels: mip_levels_start..mip_levels_end,
            layers: layers_start..layers_end,
        })
    }

    pub fn sampled_float(&self, descriptor: &View2DDescriptor) -> Sampled2DFloat
    where
        F: FloatSamplable,
        U: TextureBinding,
    {
        Sampled2DFloat {
            inner: self.view_2d_internal(F::FORMAT_ID, descriptor),
            _marker: Default::default(),
        }
    }

    pub fn try_as_sampled_float<ViewedFormat>(
        &self,
        descriptor: &View2DDescriptor,
    ) -> Result<Sampled2DFloat, UnsupportedViewFormat>
    where
        ViewedFormat: ViewFormat<F> + FloatSamplable,
        U: TextureBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(Sampled2DFloat {
                inner: self.view_2d_internal(ViewedFormat::FORMAT_ID, descriptor),
                _marker: Default::default(),
            })
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    pub fn sampled_unfilterable_float(
        &self,
        descriptor: &View2DDescriptor,
    ) -> Sampled2DUnfilteredFloat
    where
        F: UnfilteredFloatSamplable,
        U: TextureBinding,
    {
        Sampled2DUnfilteredFloat {
            inner: self.view_2d_internal(F::FORMAT_ID, descriptor),
            _marker: Default::default(),
        }
    }

    pub fn try_as_sampled_unfilterable_float<ViewedFormat>(
        &self,
        descriptor: &View2DDescriptor,
    ) -> Result<Sampled2DUnfilteredFloat, UnsupportedViewFormat>
    where
        ViewedFormat: ViewFormat<F> + UnfilteredFloatSamplable,
        U: TextureBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(Sampled2DUnfilteredFloat {
                inner: self.view_2d_internal(ViewedFormat::FORMAT_ID, descriptor),
                _marker: Default::default(),
            })
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    pub fn sampled_signed_integer(&self, descriptor: &View2DDescriptor) -> Sampled2DSignedInteger
    where
        F: SignedIntegerSamplable,
        U: TextureBinding,
    {
        Sampled2DSignedInteger {
            inner: self.view_2d_internal(F::FORMAT_ID, descriptor),
            _marker: Default::default(),
        }
    }

    pub fn try_as_sampled_signed_integer<ViewedFormat>(
        &self,
        descriptor: &View2DDescriptor,
    ) -> Result<Sampled2DSignedInteger, UnsupportedViewFormat>
    where
        ViewedFormat: ViewFormat<F> + SignedIntegerSamplable,
        U: TextureBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(Sampled2DSignedInteger {
                inner: self.view_2d_internal(ViewedFormat::FORMAT_ID, descriptor),
                _marker: Default::default(),
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
        descriptor: &View2DDescriptor,
    ) -> Sampled2DUnsignedInteger
    where
        F: UnsignedIntegerSamplable,
        U: TextureBinding,
    {
        Sampled2DUnsignedInteger {
            inner: self.view_2d_internal(F::FORMAT_ID, descriptor),
            _marker: Default::default(),
        }
    }

    pub fn try_as_sampled_unsigned_integer<ViewedFormat>(
        &self,
        descriptor: &View2DDescriptor,
    ) -> Result<Sampled2DUnsignedInteger, UnsupportedViewFormat>
    where
        ViewedFormat: ViewFormat<F> + UnsignedIntegerSamplable,
        U: TextureBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(Sampled2DUnsignedInteger {
                inner: self.view_2d_internal(ViewedFormat::FORMAT_ID, descriptor),
                _marker: Default::default(),
            })
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    pub fn sampled_depth(&self, descriptor: &View2DDescriptor) -> Sampled2DDepth
    where
        F: DepthSamplable,
        U: TextureBinding,
    {
        Sampled2DDepth {
            inner: self.view_2d_internal(F::FORMAT_ID, descriptor),
            _marker: Default::default(),
        }
    }

    pub fn try_as_sampled_depth<ViewedFormat>(
        &self,
        descriptor: &View2DDescriptor,
    ) -> Result<Sampled2DDepth, UnsupportedViewFormat>
    where
        ViewedFormat: ViewFormat<F> + DepthSamplable,
        U: TextureBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(Sampled2DDepth {
                inner: self.view_2d_internal(ViewedFormat::FORMAT_ID, descriptor),
                _marker: Default::default(),
            })
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    pub fn sampled_depth_aspect(&self, descriptor: &View2DDescriptor) -> Sampled2DDepth
    where
        F: DepthStencilFormat,
        U: TextureBinding,
    {
        Sampled2DDepth {
            inner: self.view_2d_internal(F::DepthAspect::FORMAT_ID, descriptor),
            _marker: Default::default(),
        }
    }

    fn view_2d_array_internal<'a>(
        &'a self,
        format: TextureFormatId,
        descriptor: &View2DArrayDescriptor,
    ) -> <Dvr as Driver>::TextureView {
        let View2DArrayDescriptor {
            base_layer,
            layer_count,
            base_mipmap_level,
            mipmap_level_count,
        } = *descriptor;

        let layer_count = if let Some(layer_count) = layer_count {
            layer_count
        } else {
            self.layers - base_layer
        };

        assert!(layer_count > 0, "must view at least one layer");
        assert!(
            base_layer + layer_count < self.layers,
            "layer range out of bounds"
        );
        assert!(
            base_mipmap_level < self.mip_level_count,
            "`base_mipmap_level` must not exceed the texture's mipmap level count"
        );

        let mipmap_level_count = if let Some(mipmap_level_count) = mipmap_level_count {
            assert!(
                base_mipmap_level + mipmap_level_count <= self.mip_level_count,
                "`base_mipmap_level + mip_level_count` must not exceed the texture's mipmap \
                    level count"
            );

            mipmap_level_count
        } else {
            self.mip_level_count - base_mipmap_level
        };

        let layers_start = base_layer as u32;
        let layers_end = base_layer + layer_count as u32;

        let mip_levels_start = base_mipmap_level as u32;
        let mip_levels_end = mip_levels_start + mipmap_level_count as u32;

        self.handle.texture_view(&TextureViewDescriptor {
            format,
            dimensions: TextureViewDimension::TwoArray,
            aspect: TextureAspect::All,
            mip_levels: mip_levels_start..mip_levels_end,
            layers: layers_start..layers_end,
        })
    }

    pub fn sampled_array_float(&self, descriptor: &View2DArrayDescriptor) -> Sampled2DArrayFloat
    where
        F: FloatSamplable,
        U: TextureBinding,
    {
        Sampled2DArrayFloat {
            inner: self.view_2d_array_internal(F::FORMAT_ID, descriptor),
            _marker: Default::default(),
        }
    }

    pub fn try_as_sampled_array_float<ViewedFormat>(
        &self,
        descriptor: &View2DArrayDescriptor,
    ) -> Result<Sampled2DArrayFloat, UnsupportedViewFormat>
    where
        ViewedFormat: ViewFormat<F> + FloatSamplable,
        U: TextureBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(Sampled2DArrayFloat {
                inner: self.view_2d_array_internal(ViewedFormat::FORMAT_ID, descriptor),
                _marker: Default::default(),
            })
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    pub fn sampled_array_unfilterable_float(
        &self,
        descriptor: &View2DArrayDescriptor,
    ) -> Sampled2DArrayUnfilteredFloat
    where
        F: UnfilteredFloatSamplable,
        U: TextureBinding,
    {
        Sampled2DArrayUnfilteredFloat {
            inner: self.view_2d_array_internal(F::FORMAT_ID, descriptor),
            _marker: Default::default(),
        }
    }

    pub fn try_as_sampled_array_unfilterable_float<ViewedFormat>(
        &self,
        descriptor: &View2DArrayDescriptor,
    ) -> Result<Sampled2DArrayUnfilteredFloat, UnsupportedViewFormat>
    where
        ViewedFormat: ViewFormat<F> + UnfilteredFloatSamplable,
        U: TextureBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(Sampled2DArrayUnfilteredFloat {
                inner: self.view_2d_array_internal(ViewedFormat::FORMAT_ID, descriptor),
                _marker: Default::default(),
            })
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    pub fn sampled_array_signed_integer(
        &self,
        descriptor: &View2DArrayDescriptor,
    ) -> Sampled2DArraySignedInteger
    where
        F: SignedIntegerSamplable,
        U: TextureBinding,
    {
        Sampled2DArraySignedInteger {
            inner: self.view_2d_array_internal(F::FORMAT_ID, descriptor),
            _marker: Default::default(),
        }
    }

    pub fn try_as_sampled_array_signed_integer<ViewedFormat>(
        &self,
        descriptor: &View2DArrayDescriptor,
    ) -> Result<Sampled2DArraySignedInteger, UnsupportedViewFormat>
    where
        ViewedFormat: ViewFormat<F> + SignedIntegerSamplable,
        U: TextureBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(Sampled2DArraySignedInteger {
                inner: self.view_2d_array_internal(ViewedFormat::FORMAT_ID, descriptor),
                _marker: Default::default(),
            })
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    pub fn sampled_array_unsigned_integer(
        &self,
        descriptor: &View2DArrayDescriptor,
    ) -> Sampled2DArrayUnsignedInteger
    where
        F: UnsignedIntegerSamplable,
        U: TextureBinding,
    {
        Sampled2DArrayUnsignedInteger {
            inner: self.view_2d_array_internal(F::FORMAT_ID, descriptor),
            _marker: Default::default(),
        }
    }

    pub fn try_as_sampled_array_unsigned_integer<ViewedFormat>(
        &self,
        descriptor: &View2DArrayDescriptor,
    ) -> Result<Sampled2DArrayUnsignedInteger, UnsupportedViewFormat>
    where
        ViewedFormat: ViewFormat<F> + UnsignedIntegerSamplable,
        U: TextureBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(Sampled2DArrayUnsignedInteger {
                inner: self.view_2d_array_internal(ViewedFormat::FORMAT_ID, descriptor),
                _marker: Default::default(),
            })
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    pub fn sampled_array_depth(&self, descriptor: &View2DArrayDescriptor) -> Sampled2DArrayDepth
    where
        F: DepthSamplable,
        U: TextureBinding,
    {
        Sampled2DArrayDepth {
            inner: self.view_2d_array_internal(F::FORMAT_ID, descriptor),
            _marker: Default::default(),
        }
    }

    pub fn try_as_sampled_array_depth<ViewedFormat>(
        &self,
        descriptor: &View2DArrayDescriptor,
    ) -> Result<Sampled2DArrayDepth, UnsupportedViewFormat>
    where
        ViewedFormat: ViewFormat<F> + DepthSamplable,
        U: TextureBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(Sampled2DArrayDepth {
                inner: self.view_2d_array_internal(ViewedFormat::FORMAT_ID, descriptor),
                _marker: Default::default(),
            })
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    pub fn sampled_array_depth_aspect(
        &self,
        descriptor: &View2DArrayDescriptor,
    ) -> Sampled2DArrayDepth
    where
        F: DepthStencilFormat,
        U: TextureBinding,
    {
        Sampled2DArrayDepth {
            inner: self.view_2d_array_internal(F::DepthAspect::FORMAT_ID, descriptor),
            _marker: Default::default(),
        }
    }

    fn view_cube_internal<'a>(
        &'a self,
        format: TextureFormatId,
        descriptor: &ViewCubeDescriptor,
    ) -> <Dvr as Driver>::TextureView {
        let ViewCubeDescriptor {
            base_layer,
            base_mipmap_level,
            mipmap_level_count,
        } = *descriptor;

        assert!(
            self.width == self.height,
            "can only view a square texture (`width == height`) as cube-sampled"
        );
        assert!(base_layer + 6 < self.layers, "layer range out of bounds");
        assert!(
            base_mipmap_level < self.mip_level_count,
            "`base_mipmap_level` must not exceed the texture's mipmap level count"
        );

        let mipmap_level_count = if let Some(mipmap_level_count) = mipmap_level_count {
            assert!(
                base_mipmap_level + mipmap_level_count <= self.mip_level_count,
                "`base_mipmap_level + mip_level_count` must not exceed the texture's mipmap \
                    level count"
            );

            mipmap_level_count
        } else {
            self.mip_level_count - base_mipmap_level
        };

        let layers_start = base_layer as u32;
        let layers_end = base_layer + 6;

        let mip_levels_start = base_mipmap_level as u32;
        let mip_levels_end = mip_levels_start + mipmap_level_count as u32;

        self.handle.texture_view(&TextureViewDescriptor {
            format,
            dimensions: TextureViewDimension::Cube,
            aspect: TextureAspect::All,
            mip_levels: mip_levels_start..mip_levels_end,
            layers: layers_start..layers_end,
        })
    }

    pub fn sampled_cube_float(&self, descriptor: &ViewCubeDescriptor) -> SampledCubeFloat
    where
        F: FloatSamplable,
        U: TextureBinding,
    {
        SampledCubeFloat {
            inner: self.view_cube_internal(F::FORMAT_ID, descriptor),
            _marker: Default::default(),
        }
    }

    pub fn try_as_sampled_cube_float<ViewedFormat>(
        &self,
        descriptor: &ViewCubeDescriptor,
    ) -> Result<SampledCubeFloat, UnsupportedViewFormat>
    where
        ViewedFormat: ViewFormat<F> + FloatSamplable,
        U: TextureBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(SampledCubeFloat {
                inner: self.view_cube_internal(ViewedFormat::FORMAT_ID, descriptor),
                _marker: Default::default(),
            })
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    pub fn sampled_cube_unfilterable_float(
        &self,
        descriptor: &ViewCubeDescriptor,
    ) -> SampledCubeUnfilteredFloat
    where
        F: UnfilteredFloatSamplable,
        U: TextureBinding,
    {
        SampledCubeUnfilteredFloat {
            inner: self.view_cube_internal(F::FORMAT_ID, descriptor),
            _marker: Default::default(),
        }
    }

    pub fn try_as_sampled_cube_unfilterable_float<ViewedFormat>(
        &self,
        descriptor: &ViewCubeDescriptor,
    ) -> Result<SampledCubeUnfilteredFloat, UnsupportedViewFormat>
    where
        ViewedFormat: ViewFormat<F> + UnfilteredFloatSamplable,
        U: TextureBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(SampledCubeUnfilteredFloat {
                inner: self.view_cube_internal(ViewedFormat::FORMAT_ID, descriptor),
                _marker: Default::default(),
            })
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    pub fn sampled_cube_signed_integer(
        &self,
        descriptor: &ViewCubeDescriptor,
    ) -> SampledCubeSignedInteger
    where
        F: SignedIntegerSamplable,
        U: TextureBinding,
    {
        SampledCubeSignedInteger {
            inner: self.view_cube_internal(F::FORMAT_ID, descriptor),
            _marker: Default::default(),
        }
    }

    pub fn try_as_sampled_cube_signed_integer<ViewedFormat>(
        &self,
        descriptor: &ViewCubeDescriptor,
    ) -> Result<SampledCubeSignedInteger, UnsupportedViewFormat>
    where
        ViewedFormat: ViewFormat<F> + SignedIntegerSamplable,
        U: TextureBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(SampledCubeSignedInteger {
                inner: self.view_cube_internal(ViewedFormat::FORMAT_ID, descriptor),
                _marker: Default::default(),
            })
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    pub fn sampled_cube_unsigned_integer(
        &self,
        descriptor: &ViewCubeDescriptor,
    ) -> SampledCubeUnsignedInteger
    where
        F: UnsignedIntegerSamplable,
        U: TextureBinding,
    {
        SampledCubeUnsignedInteger {
            inner: self.view_cube_internal(F::FORMAT_ID, descriptor),
            _marker: Default::default(),
        }
    }

    pub fn try_as_sampled_cube_unsigned_integer<ViewedFormat>(
        &self,
        descriptor: &ViewCubeDescriptor,
    ) -> Result<SampledCubeUnsignedInteger, UnsupportedViewFormat>
    where
        ViewedFormat: ViewFormat<F> + UnsignedIntegerSamplable,
        U: TextureBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(SampledCubeUnsignedInteger {
                inner: self.view_cube_internal(ViewedFormat::FORMAT_ID, descriptor),
                _marker: Default::default(),
            })
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    pub fn sampled_cube_depth(&self, descriptor: &ViewCubeDescriptor) -> SampledCubeDepth
    where
        F: DepthSamplable,
        U: TextureBinding,
    {
        SampledCubeDepth {
            inner: self.view_cube_internal(F::FORMAT_ID, descriptor),
            _marker: Default::default(),
        }
    }

    pub fn try_as_sampled_cube_depth<ViewedFormat>(
        &self,
        descriptor: &ViewCubeDescriptor,
    ) -> Result<SampledCubeDepth, UnsupportedViewFormat>
    where
        ViewedFormat: ViewFormat<F> + DepthSamplable,
        U: TextureBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(SampledCubeDepth {
                inner: self.view_cube_internal(ViewedFormat::FORMAT_ID, descriptor),
                _marker: Default::default(),
            })
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    pub fn sampled_cube_depth_aspect(&self, descriptor: &ViewCubeDescriptor) -> SampledCubeDepth
    where
        F: DepthStencilFormat,
        U: TextureBinding,
    {
        SampledCubeDepth {
            inner: self.view_cube_internal(F::DepthAspect::FORMAT_ID, descriptor),
            _marker: Default::default(),
        }
    }

    fn view_cube_array_internal<'a>(
        &'a self,
        format: TextureFormatId,
        descriptor: &ViewCubeArrayDescriptor,
    ) -> <Dvr as Driver>::TextureView {
        let ViewCubeArrayDescriptor {
            base_layer,
            cube_count,
            base_mipmap_level,
            mipmap_level_count,
        } = *descriptor;

        let cube_count = if let Some(cube_count) = cube_count {
            cube_count
        } else {
            (self.layers - base_layer) / 6
        };
        let layer_count = cube_count * 6;

        assert!(cube_count > 0, "must view at least one cube");
        assert!(
            self.width == self.height,
            "can only view a square texture (`width == height`) as cube-sampled"
        );
        assert!(
            base_layer + layer_count < self.layers,
            "layer range out of bounds"
        );
        assert!(
            base_mipmap_level < self.mip_level_count,
            "`base_mipmap_level` must not exceed the texture's mipmap level count"
        );

        let mipmap_level_count = if let Some(mipmap_level_count) = mipmap_level_count {
            assert!(
                base_mipmap_level + mipmap_level_count <= self.mip_level_count,
                "`base_mipmap_level + mip_level_count` must not exceed the texture's mipmap \
                    level count"
            );

            mipmap_level_count
        } else {
            self.mip_level_count - base_mipmap_level
        };

        let layers_start = base_layer as u32;
        let layers_end = base_layer + layer_count;

        let mip_levels_start = base_mipmap_level as u32;
        let mip_levels_end = mip_levels_start + mipmap_level_count as u32;

        self.handle.texture_view(&TextureViewDescriptor {
            format,
            dimensions: TextureViewDimension::CubeArray,
            aspect: TextureAspect::All,
            mip_levels: mip_levels_start..mip_levels_end,
            layers: layers_start..layers_end,
        })
    }

    pub fn sampled_cube_array_float(
        &self,
        descriptor: &ViewCubeArrayDescriptor,
    ) -> SampledCubeArrayFloat
    where
        F: FloatSamplable,
        U: TextureBinding,
    {
        SampledCubeArrayFloat {
            inner: self.view_cube_array_internal(F::FORMAT_ID, descriptor),
            _marker: Default::default(),
        }
    }

    pub fn try_as_sampled_cube_array_float<ViewedFormat>(
        &self,
        descriptor: &ViewCubeArrayDescriptor,
    ) -> Result<SampledCubeArrayFloat, UnsupportedViewFormat>
    where
        ViewedFormat: ViewFormat<F> + FloatSamplable,
        U: TextureBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(SampledCubeArrayFloat {
                inner: self.view_cube_array_internal(ViewedFormat::FORMAT_ID, descriptor),
                _marker: Default::default(),
            })
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    pub fn sampled_cube_array_unfilterable_float(
        &self,
        descriptor: &ViewCubeArrayDescriptor,
    ) -> SampledCubeArrayUnfilteredFloat
    where
        F: UnfilteredFloatSamplable,
        U: TextureBinding,
    {
        SampledCubeArrayUnfilteredFloat {
            inner: self.view_cube_array_internal(F::FORMAT_ID, descriptor),
            _marker: Default::default(),
        }
    }

    pub fn try_as_sampled_cube_array_unfilterable_float<ViewedFormat>(
        &self,
        descriptor: &ViewCubeArrayDescriptor,
    ) -> Result<SampledCubeArrayUnfilteredFloat, UnsupportedViewFormat>
    where
        ViewedFormat: ViewFormat<F> + UnfilteredFloatSamplable,
        U: TextureBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(SampledCubeArrayUnfilteredFloat {
                inner: self.view_cube_array_internal(ViewedFormat::FORMAT_ID, descriptor),
                _marker: Default::default(),
            })
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    pub fn sampled_cube_array_signed_integer(
        &self,
        descriptor: &ViewCubeArrayDescriptor,
    ) -> SampledCubeArraySignedInteger
    where
        F: SignedIntegerSamplable,
        U: TextureBinding,
    {
        SampledCubeArraySignedInteger {
            inner: self.view_cube_array_internal(F::FORMAT_ID, descriptor),
            _marker: Default::default(),
        }
    }

    pub fn try_as_sampled_cube_array_signed_integer<ViewedFormat>(
        &self,
        descriptor: &ViewCubeArrayDescriptor,
    ) -> Result<SampledCubeArraySignedInteger, UnsupportedViewFormat>
    where
        ViewedFormat: ViewFormat<F> + SignedIntegerSamplable,
        U: TextureBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(SampledCubeArraySignedInteger {
                inner: self.view_cube_array_internal(ViewedFormat::FORMAT_ID, descriptor),
                _marker: Default::default(),
            })
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    pub fn sampled_cube_array_unsigned_integer(
        &self,
        descriptor: &ViewCubeArrayDescriptor,
    ) -> SampledCubeArrayUnsignedInteger
    where
        F: UnsignedIntegerSamplable,
        U: TextureBinding,
    {
        SampledCubeArrayUnsignedInteger {
            inner: self.view_cube_array_internal(F::FORMAT_ID, descriptor),
            _marker: Default::default(),
        }
    }

    pub fn try_as_sampled_cube_array_unsigned_integer<ViewedFormat>(
        &self,
        descriptor: &ViewCubeArrayDescriptor,
    ) -> Result<SampledCubeArrayUnsignedInteger, UnsupportedViewFormat>
    where
        ViewedFormat: ViewFormat<F> + UnsignedIntegerSamplable,
        U: TextureBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(SampledCubeArrayUnsignedInteger {
                inner: self.view_cube_array_internal(ViewedFormat::FORMAT_ID, descriptor),
                _marker: Default::default(),
            })
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    pub fn sampled_cube_array_depth(
        &self,
        descriptor: &ViewCubeArrayDescriptor,
    ) -> SampledCubeArrayDepth
    where
        F: DepthSamplable,
        U: TextureBinding,
    {
        SampledCubeArrayDepth {
            inner: self.view_cube_array_internal(F::FORMAT_ID, descriptor),
            _marker: Default::default(),
        }
    }

    pub fn try_as_sampled_cube_array_depth<ViewedFormat>(
        &self,
        descriptor: &ViewCubeArrayDescriptor,
    ) -> Result<SampledCubeArrayDepth, UnsupportedViewFormat>
    where
        ViewedFormat: ViewFormat<F> + DepthSamplable,
        U: TextureBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(SampledCubeArrayDepth {
                inner: self.view_cube_array_internal(ViewedFormat::FORMAT_ID, descriptor),
                _marker: Default::default(),
            })
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    pub fn sampled_cube_array_depth_aspect(
        &self,
        descriptor: &ViewCubeArrayDescriptor,
    ) -> SampledCubeArrayDepth
    where
        F: DepthStencilFormat,
        U: TextureBinding,
    {
        SampledCubeArrayDepth {
            inner: self.view_cube_array_internal(F::DepthAspect::FORMAT_ID, descriptor),
            _marker: Default::default(),
        }
    }

    fn attachable_image_internal<ViewedFormat>(
        &self,
        descriptor: &AttachableImageDescriptor,
    ) -> AttachableImage<ViewedFormat>
    where
        ViewedFormat: Renderable,
        U: RenderAttachment,
    {
        let AttachableImageDescriptor {
            layer,
            mipmap_level,
        } = *descriptor;

        assert!(layer < self.layers, "`layer` out of bounds");
        assert!(
            mipmap_level < self.mip_level_count,
            "`base_mipmap_level` must not exceed the texture's mipmap level count"
        );

        let mip_levels_start = mipmap_level as u32;
        let mip_levels_end = mip_levels_start + 1;

        let layers_start = layer;
        let layers_end = layers_start + 1;

        let inner = self.handle.texture_view(&TextureViewDescriptor {
            format: ViewedFormat::FORMAT_ID,
            dimensions: TextureViewDimension::Two,
            aspect: TextureAspect::All,
            mip_levels: mip_levels_start..mip_levels_end,
            layers: layers_start..layers_end,
        });

        AttachableImage {
            inner,
            width: self.width,
            height: self.height,
            _marker: Default::default(),
        }
    }

    pub fn attachable_image(&self, descriptor: &AttachableImageDescriptor) -> AttachableImage<F>
    where
        F: Renderable,
        U: RenderAttachment,
    {
        self.attachable_image_internal(descriptor)
    }

    pub fn try_as_attachable_image<ViewedFormat>(
        &self,
        descriptor: &AttachableImageDescriptor,
    ) -> Result<AttachableImage<ViewedFormat>, UnsupportedViewFormat>
    where
        ViewedFormat: ViewFormat<F> + Renderable,
        U: RenderAttachment,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(self.attachable_image_internal(descriptor))
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    fn storage_internal<ViewedFormat>(
        &self,
        descriptor: &Storage2DDescriptor,
    ) -> Storage2D<ViewedFormat>
    where
        ViewedFormat: Storable,
        U: StorageBinding,
    {
        let Storage2DDescriptor {
            layer,
            mipmap_level,
        } = *descriptor;

        assert!(layer < self.layers, "`layer` out of bounds");
        assert!(
            mipmap_level < self.mip_level_count,
            "`mipmap_level` must not exceed the texture's mipmap level count"
        );

        let mip_levels_start = mipmap_level as u32;
        let mip_levels_end = mip_levels_start + 1;

        let layers_start = layer;
        let layers_end = layers_start + 1;

        let inner = self.handle.texture_view(&TextureViewDescriptor {
            format: ViewedFormat::FORMAT_ID,
            dimensions: TextureViewDimension::Two,
            aspect: TextureAspect::All,
            mip_levels: mip_levels_start..mip_levels_end,
            layers: layers_start..layers_end,
        });

        Storage2D {
            inner,
            _marker: Default::default(),
        }
    }

    pub fn storage(&self, descriptor: &Storage2DDescriptor) -> Storage2D<F>
    where
        F: Storable,
        U: StorageBinding,
    {
        self.storage_internal(descriptor)
    }

    pub fn try_as_storage<ViewedFormat>(
        &self,
        descriptor: &Storage2DDescriptor,
    ) -> Result<Storage2D<ViewedFormat>, UnsupportedViewFormat>
    where
        ViewedFormat: ViewFormat<F> + Storable,
        U: StorageBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(self.storage_internal(descriptor))
        } else {
            Err(UnsupportedViewFormat {
                format: ViewedFormat::FORMAT_ID,
                supported_formats: self.view_formats.clone(),
            })
        }
    }

    fn storage_array_internal<ViewedFormat>(
        &self,
        descriptor: &Storage2DArrayDescriptor,
    ) -> Storage2DArray<ViewedFormat>
    where
        ViewedFormat: Storable,
        U: StorageBinding,
    {
        let Storage2DArrayDescriptor {
            base_layer,
            layer_count,
            mipmap_level,
        } = *descriptor;

        assert!(layer_count > 0, "must view at least one layer");
        assert!(
            base_layer + layer_count < self.layers,
            "layer range out of bounds"
        );
        assert!(
            mipmap_level < self.mip_level_count,
            "`mipmap_level` must not exceed the texture's mipmap level count"
        );

        let mip_levels_start = mipmap_level as u32;
        let mip_levels_end = mip_levels_start + 1;

        let layers_start = base_layer;
        let layers_end = layers_start + layer_count;

        let inner = self.handle.texture_view(&TextureViewDescriptor {
            format: ViewedFormat::FORMAT_ID,
            dimensions: TextureViewDimension::TwoArray,
            aspect: TextureAspect::All,
            mip_levels: mip_levels_start..mip_levels_end,
            layers: layers_start..layers_end,
        });

        Storage2DArray {
            inner,
            _marker: Default::default(),
        }
    }

    pub fn storage_array(&self, descriptor: &Storage2DArrayDescriptor) -> Storage2DArray<F>
    where
        F: Storable,
        U: StorageBinding,
    {
        self.storage_array_internal(descriptor)
    }

    pub fn try_as_storage_array<ViewedFormat>(
        &self,
        descriptor: &Storage2DArrayDescriptor,
    ) -> Result<Storage2DArray<ViewedFormat>, UnsupportedViewFormat>
    where
        ViewedFormat: ViewFormat<F> + Storable,
        U: StorageBinding,
    {
        if self.view_formats.contains(&ViewedFormat::FORMAT_ID) {
            Ok(self.storage_array_internal(descriptor))
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
        aspect: TextureAspect,
    ) -> ImageCopyTexture<F> {
        assert!(
            mipmap_level < self.mip_level_count,
            "mipmap level out of bounds"
        );

        let inner = driver::ImageCopyTexture {
            texture_handle: &self.handle,
            mip_level: mipmap_level as u32,
            origin: (0, 0, 0),
            aspect,
        };

        ImageCopyTexture {
            inner,
            width: self.width,
            height: self.height,
            depth_or_layers: self.layers,
            bytes_per_block,
            block_size,
            _marker: Default::default(),
        }
    }

    fn sub_image_copy_internal(
        &self,
        descriptor: SubImageCopy2DDescriptor,
        bytes_per_block: u32,
        block_size: [u32; 2],
    ) -> ImageCopyTexture<F> {
        let SubImageCopy2DDescriptor {
            mipmap_level,
            origin_x,
            origin_y,
            origin_layer,
        } = descriptor;

        assert!(
            mipmap_level < self.mip_level_count,
            "mipmap level out of bounds"
        );
        assert!(origin_x < self.width, "`x` origin out of bounds");
        assert!(origin_y < self.height, "`y` origin out of bounds");
        assert!(origin_layer < self.layers, "layer origin out of bounds");

        let [block_width, block_height] = block_size;

        assert!(
            origin_x.rem(block_width) == 0,
            "`x` origin must be a multiple of the format's block width (`{}`)",
            block_width
        );
        assert!(
            origin_y.rem(block_height) == 0,
            "`x` origin must be a multiple of the format's block height (`{}`)",
            block_height
        );

        let inner = driver::ImageCopyTexture {
            texture_handle: &self.handle,
            mip_level: mipmap_level as u32,
            origin: (origin_x, origin_y, origin_layer),
            aspect: TextureAspect::All,
        };

        ImageCopyTexture {
            inner,
            width: self.width,
            height: self.height,
            depth_or_layers: self.layers,
            bytes_per_block,
            block_size,
            _marker: Default::default(),
        }
    }

    pub fn image_copy_to_buffer_src(&self, mipmap_level: u8) -> ImageCopySrc<F>
    where
        F: ImageCopyToBufferFormat,
        U: CopySrc,
    {
        ImageCopySrc {
            inner: self.image_copy_internal(
                mipmap_level,
                F::BYTES_PER_BLOCK,
                F::BLOCK_SIZE,
                TextureAspect::All,
            ),
        }
    }

    pub fn image_copy_to_buffer_src_depth(&self, mipmap_level: u8) -> ImageCopySrc<F>
    where
        F: DepthStencilFormat,
        F::DepthAspect: ImageCopyToBufferFormat,
        U: CopySrc,
    {
        ImageCopySrc {
            inner: self.image_copy_internal(
                mipmap_level,
                F::DepthAspect::BYTES_PER_BLOCK,
                F::DepthAspect::BLOCK_SIZE,
                TextureAspect::DepthOnly,
            ),
        }
    }

    pub fn image_copy_to_buffer_src_stencil(&self, mipmap_level: u8) -> ImageCopySrc<F>
    where
        F: DepthStencilFormat,
        F::StencilAspect: ImageCopyToBufferFormat,
        U: CopySrc,
    {
        ImageCopySrc {
            inner: self.image_copy_internal(
                mipmap_level,
                F::StencilAspect::BYTES_PER_BLOCK,
                F::StencilAspect::BLOCK_SIZE,
                TextureAspect::StencilOnly,
            ),
        }
    }

    pub fn image_copy_from_buffer_dst(&self, mipmap_level: u8) -> ImageCopyDst<F>
    where
        F: ImageCopyFromBufferFormat,
        U: CopyDst,
    {
        ImageCopyDst {
            inner: self.image_copy_internal(
                mipmap_level,
                F::BYTES_PER_BLOCK,
                F::BLOCK_SIZE,
                TextureAspect::All,
            ),
        }
    }

    // Note: including this function for completeness sake, but this should not currently be
    // invokable, no format meets the constraints.
    pub fn image_copy_from_buffer_dst_depth(&self, mipmap_level: u8) -> ImageCopyDst<F>
    where
        F: DepthStencilFormat,
        F::DepthAspect: ImageCopyFromBufferFormat,
        U: CopyDst,
    {
        ImageCopyDst {
            inner: self.image_copy_internal(
                mipmap_level,
                F::DepthAspect::BYTES_PER_BLOCK,
                F::DepthAspect::BLOCK_SIZE,
                TextureAspect::DepthOnly,
            ),
        }
    }

    pub fn image_copy_from_buffer_dst_stencil(&self, mipmap_level: u8) -> ImageCopyDst<F>
    where
        F: DepthStencilFormat,
        F::StencilAspect: ImageCopyFromBufferFormat,
        U: CopyDst,
    {
        ImageCopyDst {
            inner: self.image_copy_internal(
                mipmap_level,
                F::StencilAspect::BYTES_PER_BLOCK,
                F::StencilAspect::BLOCK_SIZE,
                TextureAspect::StencilOnly,
            ),
        }
    }

    pub fn image_copy_to_texture_src(&self, mipmap_level: u8) -> ImageCopyToTextureSrc<F>
    where
        F: ImageCopyTextureFormat,
        U: CopySrc,
    {
        ImageCopyToTextureSrc {
            inner: self.image_copy_internal(mipmap_level, 0, F::BLOCK_SIZE, TextureAspect::All),
        }
    }

    pub fn image_copy_from_texture_dst(&self, mipmap_level: u8) -> ImageCopyFromTextureDst<F>
    where
        F: ImageCopyTextureFormat,
        U: CopyDst,
    {
        ImageCopyFromTextureDst {
            inner: self.image_copy_internal(mipmap_level, 0, F::BLOCK_SIZE, TextureAspect::All),
        }
    }

    pub fn sub_image_copy_to_buffer_src(
        &self,
        descriptor: SubImageCopy2DDescriptor,
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
        descriptor: SubImageCopy2DDescriptor,
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
        descriptor: SubImageCopy2DDescriptor,
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
        descriptor: SubImageCopy2DDescriptor,
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

#[derive(Clone)]
pub struct Sampled2DFloat<'a> {
    pub(crate) inner: <Dvr as Driver>::TextureView,
    _marker: marker::PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct Sampled2DUnfilteredFloat<'a> {
    pub(crate) inner: <Dvr as Driver>::TextureView,
    _marker: marker::PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct Sampled2DSignedInteger<'a> {
    pub(crate) inner: <Dvr as Driver>::TextureView,
    _marker: marker::PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct Sampled2DUnsignedInteger<'a> {
    pub(crate) inner: <Dvr as Driver>::TextureView,
    _marker: marker::PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct Sampled2DDepth<'a> {
    pub(crate) inner: <Dvr as Driver>::TextureView,
    _marker: marker::PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct Sampled2DArrayFloat<'a> {
    pub(crate) inner: <Dvr as Driver>::TextureView,
    _marker: marker::PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct Sampled2DArrayUnfilteredFloat<'a> {
    pub(crate) inner: <Dvr as Driver>::TextureView,
    _marker: marker::PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct Sampled2DArraySignedInteger<'a> {
    pub(crate) inner: <Dvr as Driver>::TextureView,
    _marker: marker::PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct Sampled2DArrayUnsignedInteger<'a> {
    pub(crate) inner: <Dvr as Driver>::TextureView,
    _marker: marker::PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct Sampled2DArrayDepth<'a> {
    pub(crate) inner: <Dvr as Driver>::TextureView,
    _marker: marker::PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct SampledCubeFloat<'a> {
    pub(crate) inner: <Dvr as Driver>::TextureView,
    _marker: marker::PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct SampledCubeUnfilteredFloat<'a> {
    pub(crate) inner: <Dvr as Driver>::TextureView,
    _marker: marker::PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct SampledCubeSignedInteger<'a> {
    pub(crate) inner: <Dvr as Driver>::TextureView,
    _marker: marker::PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct SampledCubeUnsignedInteger<'a> {
    pub(crate) inner: <Dvr as Driver>::TextureView,
    _marker: marker::PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct SampledCubeDepth<'a> {
    pub(crate) inner: <Dvr as Driver>::TextureView,
    _marker: marker::PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct SampledCubeArrayFloat<'a> {
    pub(crate) inner: <Dvr as Driver>::TextureView,
    _marker: marker::PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct SampledCubeArrayUnfilteredFloat<'a> {
    pub(crate) inner: <Dvr as Driver>::TextureView,
    _marker: marker::PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct SampledCubeArraySignedInteger<'a> {
    pub(crate) inner: <Dvr as Driver>::TextureView,
    _marker: marker::PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct SampledCubeArrayUnsignedInteger<'a> {
    pub(crate) inner: <Dvr as Driver>::TextureView,
    _marker: marker::PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct SampledCubeArrayDepth<'a> {
    pub(crate) inner: <Dvr as Driver>::TextureView,
    _marker: marker::PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct Storage2D<'a, F> {
    pub(crate) inner: <Dvr as Driver>::TextureView,
    _marker: marker::PhantomData<&'a F>,
}

#[derive(Clone)]
pub struct Storage2DArray<'a, F> {
    pub(crate) inner: <Dvr as Driver>::TextureView,
    _marker: marker::PhantomData<&'a F>,
}

#[derive(Clone)]
pub struct AttachableImage<'a, F> {
    pub(crate) inner: <Dvr as Driver>::TextureView,
    pub(crate) width: u32,
    pub(crate) height: u32,
    _marker: marker::PhantomData<&'a F>,
}
