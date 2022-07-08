use std::cmp::max;
use std::marker;
use std::ops::Rem;
use std::sync::Arc;

use staticvec::StaticVec;
use web_sys::{
    GpuExtent3dDict, GpuTexture, GpuTextureAspect, GpuTextureDescriptor, GpuTextureDimension,
    GpuTextureFormat, GpuTextureView, GpuTextureViewDescriptor, GpuTextureViewDimension,
};

use crate::device::Device;
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
    TextureBinding, TextureDestroyer, UnsupportedViewFormat, UsageFlags,
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
    pub(crate) inner: Arc<TextureDestroyer>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) layers: u32,
    pub(crate) mip_level_count: u8,
    format: FormatKind<F>,
    view_formats: StaticVec<TextureFormatId, 8>,
    _usage: marker::PhantomData<Usage>,
}

impl<F, U> Texture2D<F, U>
where
    F: TextureFormat,
    U: UsageFlags,
{
    pub(crate) fn from_swap_chain_texture(
        web_sys: GpuTexture,
        width: u32,
        height: u32,
        view_formats: &[TextureFormatId],
    ) -> Self {
        let view_formats = StaticVec::from(view_formats);

        Texture2D {
            inner: Arc::new(TextureDestroyer::new(web_sys, true)),
            width,
            height,
            layers: 1,
            mip_level_count: 1,
            format: FormatKind::Typed(Default::default()),
            view_formats,
            _usage: Default::default(),
        }
    }
}

impl<F, U> Texture2D<F, U> {
    pub(crate) fn as_web_sys(&self) -> &GpuTexture {
        &self.inner.texture
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
        let mut size = GpuExtent3dDict::new(*width);

        size.height(*height);
        size.depth_or_array_layers(*layers);

        let mut desc = GpuTextureDescriptor::new(F::FORMAT_ID.to_web_sys(), &size.into(), U::BITS);

        desc.dimension(GpuTextureDimension::N2d);
        desc.mip_level_count(mip_level_count);

        let inner = device.inner.create_texture(&desc);
        let view_formats = view_formats.formats().collect();

        Texture2D {
            inner: Arc::new(TextureDestroyer::new(inner, false)),
            width: *width,
            height: *height,
            layers: *layers,
            mip_level_count: mip_level_count as u8,
            format: FormatKind::Typed(Default::default()),
            view_formats,
            _usage: Default::default(),
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn layers(&self) -> u32 {
        self.layers
    }

    fn view_2d_internal(
        &self,
        format: GpuTextureFormat,
        descriptor: &View2DDescriptor,
    ) -> GpuTextureView {
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
                base_mipmap_level + mipmap_level_count < self.mip_level_count,
                "`base_mipmap_level + mip_level_count` must not exceed the texture's mipmap \
                    level count"
            );

            mipmap_level_count
        } else {
            self.mip_level_count - base_mipmap_level
        };

        let mut desc = GpuTextureViewDescriptor::new();

        desc.base_array_layer(layer);
        desc.dimension(GpuTextureViewDimension::N2d);
        desc.format(format);
        desc.base_mip_level(base_mipmap_level as u32);
        desc.mip_level_count(mipmap_level_count as u32);

        self.as_web_sys().create_view_with_descriptor(&desc)
    }

    pub fn sampled_float(&self, descriptor: &View2DDescriptor) -> Sampled2DFloat
    where
        F: FloatSamplable,
        U: TextureBinding,
    {
        Sampled2DFloat {
            inner: self.view_2d_internal(F::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
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
                inner: self.view_2d_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
                texture_destroyer: self.inner.clone(),
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
            inner: self.view_2d_internal(F::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
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
                inner: self.view_2d_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
                texture_destroyer: self.inner.clone(),
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
            inner: self.view_2d_internal(F::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
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
                inner: self.view_2d_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
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
        descriptor: &View2DDescriptor,
    ) -> Sampled2DUnsignedInteger
    where
        F: UnsignedIntegerSamplable,
        U: TextureBinding,
    {
        Sampled2DUnsignedInteger {
            inner: self.view_2d_internal(F::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
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
                inner: self.view_2d_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
                texture_destroyer: self.inner.clone(),
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
            inner: self.view_2d_internal(F::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
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
                inner: self.view_2d_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
                texture_destroyer: self.inner.clone(),
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
            inner: self.view_2d_internal(F::DepthAspect::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
        }
    }

    fn view_2d_array_internal(
        &self,
        format: GpuTextureFormat,
        descriptor: &View2DArrayDescriptor,
    ) -> GpuTextureView {
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
                base_mipmap_level + mipmap_level_count < self.mip_level_count,
                "`base_mipmap_level + mip_level_count` must not exceed the texture's mipmap \
                    level count"
            );

            mipmap_level_count
        } else {
            self.mip_level_count - base_mipmap_level
        };

        let mut desc = GpuTextureViewDescriptor::new();

        desc.base_array_layer(base_layer);
        desc.array_layer_count(layer_count);
        desc.dimension(GpuTextureViewDimension::N2dArray);
        desc.format(format);
        desc.base_mip_level(base_mipmap_level as u32);
        desc.mip_level_count(mipmap_level_count as u32);

        self.as_web_sys().create_view_with_descriptor(&desc)
    }

    pub fn sampled_array_float(&self, descriptor: &View2DArrayDescriptor) -> Sampled2DArrayFloat
    where
        F: FloatSamplable,
        U: TextureBinding,
    {
        Sampled2DArrayFloat {
            inner: self.view_2d_array_internal(F::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
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
                inner: self
                    .view_2d_array_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
                texture_destroyer: self.inner.clone(),
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
            inner: self.view_2d_array_internal(F::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
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
                inner: self
                    .view_2d_array_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
                texture_destroyer: self.inner.clone(),
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
            inner: self.view_2d_array_internal(F::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
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
                inner: self
                    .view_2d_array_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
                texture_destroyer: self.inner.clone(),
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
            inner: self.view_2d_array_internal(F::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
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
                inner: self
                    .view_2d_array_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
                texture_destroyer: self.inner.clone(),
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
            inner: self.view_2d_array_internal(F::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
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
                inner: self
                    .view_2d_array_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
                texture_destroyer: self.inner.clone(),
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
            inner: self.view_2d_array_internal(F::DepthAspect::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
        }
    }

    fn view_cube_internal(
        &self,
        format: GpuTextureFormat,
        descriptor: &ViewCubeDescriptor,
    ) -> GpuTextureView {
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
                base_mipmap_level + mipmap_level_count < self.mip_level_count,
                "`base_mipmap_level + mip_level_count` must not exceed the texture's mipmap \
                    level count"
            );

            mipmap_level_count
        } else {
            self.mip_level_count - base_mipmap_level
        };

        let mut desc = GpuTextureViewDescriptor::new();

        desc.base_array_layer(base_layer);
        desc.array_layer_count(6);
        desc.dimension(GpuTextureViewDimension::Cube);
        desc.format(format);
        desc.base_mip_level(base_mipmap_level as u32);
        desc.mip_level_count(mipmap_level_count as u32);

        self.as_web_sys().create_view_with_descriptor(&desc)
    }

    pub fn sampled_cube_float(&self, descriptor: &ViewCubeDescriptor) -> SampledCubeFloat
    where
        F: FloatSamplable,
        U: TextureBinding,
    {
        SampledCubeFloat {
            inner: self.view_cube_internal(F::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
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
                inner: self.view_cube_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
                texture_destroyer: self.inner.clone(),
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
            inner: self.view_cube_internal(F::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
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
                inner: self.view_cube_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
                texture_destroyer: self.inner.clone(),
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
            inner: self.view_cube_internal(F::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
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
                inner: self.view_cube_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
                texture_destroyer: self.inner.clone(),
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
            inner: self.view_cube_internal(F::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
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
                inner: self.view_cube_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
                texture_destroyer: self.inner.clone(),
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
            inner: self.view_cube_internal(F::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
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
                inner: self.view_cube_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
                texture_destroyer: self.inner.clone(),
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
            inner: self.view_cube_internal(F::DepthAspect::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
        }
    }

    fn view_cube_array_internal(
        &self,
        format: GpuTextureFormat,
        descriptor: &ViewCubeArrayDescriptor,
    ) -> GpuTextureView {
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
                base_mipmap_level + mipmap_level_count < self.mip_level_count,
                "`base_mipmap_level + mip_level_count` must not exceed the texture's mipmap \
                    level count"
            );

            mipmap_level_count
        } else {
            self.mip_level_count - base_mipmap_level
        };

        let mut desc = GpuTextureViewDescriptor::new();

        desc.base_array_layer(base_layer);
        desc.array_layer_count(layer_count);
        desc.dimension(GpuTextureViewDimension::CubeArray);
        desc.format(format);
        desc.base_mip_level(base_mipmap_level as u32);
        desc.mip_level_count(mipmap_level_count as u32);

        self.as_web_sys().create_view_with_descriptor(&desc)
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
            inner: self.view_cube_array_internal(F::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
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
                inner: self
                    .view_cube_array_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
                texture_destroyer: self.inner.clone(),
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
            inner: self.view_cube_array_internal(F::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
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
                inner: self
                    .view_cube_array_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
                texture_destroyer: self.inner.clone(),
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
            inner: self.view_cube_array_internal(F::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
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
                inner: self
                    .view_cube_array_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
                texture_destroyer: self.inner.clone(),
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
            inner: self.view_cube_array_internal(F::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
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
                inner: self
                    .view_cube_array_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
                texture_destroyer: self.inner.clone(),
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
            inner: self.view_cube_array_internal(F::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
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
                inner: self
                    .view_cube_array_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
                texture_destroyer: self.inner.clone(),
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
            inner: self
                .view_cube_array_internal(F::DepthAspect::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
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

        let mut desc = GpuTextureViewDescriptor::new();

        desc.base_array_layer(layer);
        desc.dimension(GpuTextureViewDimension::N2d);
        desc.format(ViewedFormat::FORMAT_ID.to_web_sys());
        desc.base_mip_level(mipmap_level as u32);

        let inner = self.as_web_sys().create_view_with_descriptor(&desc);

        AttachableImage {
            inner,
            width: self.width,
            height: self.height,
            texture_destroyer: self.inner.clone(),
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

        let mut desc = GpuTextureViewDescriptor::new();

        desc.base_array_layer(layer);
        desc.dimension(GpuTextureViewDimension::N2d);
        desc.format(ViewedFormat::FORMAT_ID.to_web_sys());
        desc.base_mip_level(mipmap_level as u32);

        let inner = self.as_web_sys().create_view_with_descriptor(&desc);

        Storage2D {
            inner,
            texture_destroyer: self.inner.clone(),
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

        let mut desc = GpuTextureViewDescriptor::new();

        desc.base_array_layer(base_layer);
        desc.array_layer_count(layer_count);
        desc.dimension(GpuTextureViewDimension::N2dArray);
        desc.format(ViewedFormat::FORMAT_ID.to_web_sys());
        desc.base_mip_level(mipmap_level as u32);

        let inner = self.as_web_sys().create_view_with_descriptor(&desc);

        Storage2DArray {
            inner,
            texture_destroyer: self.inner.clone(),
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
        aspect: GpuTextureAspect,
    ) -> ImageCopyTexture<F> {
        assert!(
            mipmap_level < self.mip_level_count,
            "mipmap level out of bounds"
        );

        ImageCopyTexture {
            texture: self.inner.clone(),
            aspect,
            mipmap_level,
            width: self.width,
            height: self.height,
            depth_or_layers: self.layers,
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

        ImageCopyTexture {
            texture: self.inner.clone(),
            aspect: GpuTextureAspect::All,
            mipmap_level,
            width: self.width,
            height: self.height,
            depth_or_layers: self.layers,
            bytes_per_block,
            block_size,
            origin_x,
            origin_y,
            origin_z: origin_layer,
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
                GpuTextureAspect::All,
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
                GpuTextureAspect::DepthOnly,
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
                GpuTextureAspect::StencilOnly,
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
                GpuTextureAspect::All,
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
                GpuTextureAspect::DepthOnly,
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
                GpuTextureAspect::StencilOnly,
            ),
        }
    }

    pub fn image_copy_to_texture_src(&self, mipmap_level: u8) -> ImageCopyToTextureSrc<F>
    where
        F: ImageCopyTextureFormat,
        U: CopySrc,
    {
        ImageCopyToTextureSrc {
            inner: self.image_copy_internal(mipmap_level, 0, F::BLOCK_SIZE, GpuTextureAspect::All),
        }
    }

    pub fn image_copy_from_texture_dst(&self, mipmap_level: u8) -> ImageCopyFromTextureDst<F>
    where
        F: ImageCopyTextureFormat,
        U: CopyDst,
    {
        ImageCopyFromTextureDst {
            inner: self.image_copy_internal(mipmap_level, 0, F::BLOCK_SIZE, GpuTextureAspect::All),
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

pub struct Sampled2DFloat {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
}

pub struct Sampled2DUnfilteredFloat {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
}

pub struct Sampled2DSignedInteger {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
}

pub struct Sampled2DUnsignedInteger {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
}

pub struct Sampled2DDepth {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
}

pub struct Sampled2DArrayFloat {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
}

pub struct Sampled2DArrayUnfilteredFloat {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
}

pub struct Sampled2DArraySignedInteger {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
}

pub struct Sampled2DArrayUnsignedInteger {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
}

pub struct Sampled2DArrayDepth {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
}

pub struct SampledCubeFloat {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
}

pub struct SampledCubeUnfilteredFloat {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
}

pub struct SampledCubeSignedInteger {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
}

pub struct SampledCubeUnsignedInteger {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
}

pub struct SampledCubeDepth {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
}

pub struct SampledCubeArrayFloat {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
}

pub struct SampledCubeArrayUnfilteredFloat {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
}

pub struct SampledCubeArraySignedInteger {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
}

pub struct SampledCubeArrayUnsignedInteger {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
}

pub struct SampledCubeArrayDepth {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
}

pub struct Storage2D<F> {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
    _marker: marker::PhantomData<*const F>,
}

pub struct Storage2DArray<F> {
    pub(crate) inner: GpuTextureView,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
    _marker: marker::PhantomData<*const F>,
}

pub struct AttachableImage<F> {
    pub(crate) inner: GpuTextureView,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) texture_destroyer: Arc<TextureDestroyer>,
    _marker: marker::PhantomData<*const F>,
}
