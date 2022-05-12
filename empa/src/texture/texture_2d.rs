use crate::device::Device;
use crate::texture::format::{
    DepthSamplable, DepthStencilFormat, FloatRenderable, FloatSamplable, ImageBufferDataFormat,
    ImageCopyFromBufferFormat, ImageCopyTextureFormat, ImageCopyToBufferFormat, Renderable,
    SignedIntegerSamplable, Storable, SubImageCopyFormat, Texture2DFormat, TextureFormat,
    UnfilteredFloatSamplable, UnsignedIntegerSamplable, ViewFormat, ViewFormats,
};
use crate::texture::{
    CopyDst, CopySrc, FormatKind, ImageCopyFromBufferDst, ImageCopyFromTextureDst,
    ImageCopyTexture, ImageCopyToBufferSrc, ImageCopyToTextureSrc, MipmapLevels, RenderAttachment,
    StorageBinding, SubImageCopyFromBufferDst, SubImageCopyFromTextureDst, SubImageCopyToBufferSrc,
    SubImageCopyToTextureSrc, TextureBinding, TextureDestroyer, UsageFlags,
};
use std::cmp::max;
use std::marker;
use std::ops::Rem;
use std::sync::Arc;
use web_sys::{
    GpuExtent3dDict, GpuTexture, GpuTextureAspect, GpuTextureDescriptor, GpuTextureDimension,
    GpuTextureFormat, GpuTextureView, GpuTextureViewDescriptor, GpuTextureViewDimension,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Texture2DDescriptor {
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

pub struct Texture2D<F, Usage, ViewFormats = F> {
    inner: Arc<TextureDestroyer>,
    width: u32,
    height: u32,
    layers: u32,
    mip_level_count: u8,
    format: FormatKind<F>,
    _usage: marker::PhantomData<Usage>,
    _view_formats: marker::PhantomData<ViewFormats>,
}

impl<F, U, V> Texture2D<F, U, V> {
    pub(crate) fn as_web_sys(&self) -> &GpuTexture {
        &self.inner.texture
    }
}

impl<F, U, V> Texture2D<F, U, V>
where
    F: Texture2DFormat,
    U: UsageFlags,
    V: ViewFormats<F>,
{
    pub(crate) fn new(device: &Device, descriptor: &Texture2DDescriptor) -> Self {
        let Texture2DDescriptor {
            width,
            height,
            layers,
            mipmap_levels,
        } = *descriptor;

        assert!(width > 0, "width must be greater than `0`");
        assert!(height > 0, "height must be greater than `0`");
        assert!(layers > 0, "must have at least one layer");

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

        let mip_level_count = mipmap_levels.to_u32(max(width, height));
        let mut size = GpuExtent3dDict::new(width);

        size.height(height);
        size.depth_or_array_layers(layers);

        let mut desc = GpuTextureDescriptor::new(F::FORMAT_ID.to_web_sys(), &size.into(), U::BITS);

        desc.dimension(GpuTextureDimension::N3d);
        desc.mip_level_count(mip_level_count);

        let inner = device.inner.create_texture(&desc);

        Texture2D {
            inner: Arc::new(TextureDestroyer::new(inner)),
            width,
            height,
            layers,
            mip_level_count: mip_level_count as u8,
            format: FormatKind::Typed(Default::default()),
            _usage: Default::default(),
            _view_formats: Default::default(),
        }
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

    pub fn sampled_float<ViewedFormat>(&self, descriptor: &View2DDescriptor) -> Sampled2DFloat
    where
        ViewedFormat: ViewFormat<V> + FloatSamplable,
        U: TextureBinding,
    {
        Sampled2DFloat {
            inner: self.view_2d_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
        }
    }

    pub fn sampled_unfilterable_float<ViewedFormat>(
        &self,
        descriptor: &View2DDescriptor,
    ) -> Sampled2DUnfilteredFloat
    where
        ViewedFormat: ViewFormat<V> + UnfilteredFloatSamplable,
        U: TextureBinding,
    {
        Sampled2DUnfilteredFloat {
            inner: self.view_2d_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
        }
    }

    pub fn sampled_signed_integer<ViewedFormat>(
        &self,
        descriptor: &View2DDescriptor,
    ) -> Sampled2DSignedInteger
    where
        ViewedFormat: ViewFormat<V> + SignedIntegerSamplable,
        U: TextureBinding,
    {
        Sampled2DSignedInteger {
            inner: self.view_2d_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
        }
    }

    pub fn sampled_unsigned_integer<ViewedFormat>(
        &self,
        descriptor: &View2DDescriptor,
    ) -> Sampled2DUnsignedInteger
    where
        ViewedFormat: ViewFormat<V> + UnsignedIntegerSamplable,
        U: TextureBinding,
    {
        Sampled2DUnsignedInteger {
            inner: self.view_2d_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
        }
    }

    pub fn sampled_depth<ViewedFormat>(&self, descriptor: &View2DDescriptor) -> Sampled2DDepth
    where
        ViewedFormat: ViewFormat<V> + DepthSamplable,
        U: TextureBinding,
    {
        Sampled2DDepth {
            inner: self.view_2d_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
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

    pub fn sampled_array_float<ViewedFormat>(
        &self,
        descriptor: &View2DArrayDescriptor,
    ) -> Sampled2DArrayFloat
    where
        ViewedFormat: ViewFormat<V> + FloatSamplable,
        U: TextureBinding,
    {
        Sampled2DArrayFloat {
            inner: self.view_2d_array_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
        }
    }

    pub fn sampled_array_unfilterable_float<ViewedFormat>(
        &self,
        descriptor: &View2DArrayDescriptor,
    ) -> Sampled2DArrayUnfilteredFloat
    where
        ViewedFormat: ViewFormat<V> + UnfilteredFloatSamplable,
        U: TextureBinding,
    {
        Sampled2DArrayUnfilteredFloat {
            inner: self.view_2d_array_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
        }
    }

    pub fn sampled_array_signed_integer<ViewedFormat>(
        &self,
        descriptor: &View2DArrayDescriptor,
    ) -> Sampled2DArraySignedInteger
    where
        ViewedFormat: ViewFormat<V> + SignedIntegerSamplable,
        U: TextureBinding,
    {
        Sampled2DArraySignedInteger {
            inner: self.view_2d_array_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
        }
    }

    pub fn sampled_array_unsigned_integer<ViewedFormat>(
        &self,
        descriptor: &View2DArrayDescriptor,
    ) -> Sampled2DArrayUnsignedInteger
    where
        ViewedFormat: ViewFormat<V> + UnsignedIntegerSamplable,
        U: TextureBinding,
    {
        Sampled2DArrayUnsignedInteger {
            inner: self.view_2d_array_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
        }
    }

    pub fn sampled_array_depth<ViewedFormat>(
        &self,
        descriptor: &View2DArrayDescriptor,
    ) -> Sampled2DArrayDepth
    where
        ViewedFormat: ViewFormat<V> + DepthSamplable,
        U: TextureBinding,
    {
        Sampled2DArrayDepth {
            inner: self.view_2d_array_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
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

    pub fn sampled_cube_float<ViewedFormat>(
        &self,
        descriptor: &ViewCubeDescriptor,
    ) -> SampledCubeFloat
    where
        ViewedFormat: ViewFormat<V> + FloatSamplable,
        U: TextureBinding,
    {
        SampledCubeFloat {
            inner: self.view_cube_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
        }
    }

    pub fn sampled_cube_unfilterable_float<ViewedFormat>(
        &self,
        descriptor: &ViewCubeDescriptor,
    ) -> SampledCubeUnfilteredFloat
    where
        ViewedFormat: ViewFormat<V> + UnfilteredFloatSamplable,
        U: TextureBinding,
    {
        SampledCubeUnfilteredFloat {
            inner: self.view_cube_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
        }
    }

    pub fn sampled_cube_signed_integer<ViewedFormat>(
        &self,
        descriptor: &ViewCubeDescriptor,
    ) -> SampledCubeSignedInteger
    where
        ViewedFormat: ViewFormat<V> + SignedIntegerSamplable,
        U: TextureBinding,
    {
        SampledCubeSignedInteger {
            inner: self.view_cube_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
        }
    }

    pub fn sampled_cube_unsigned_integer<ViewedFormat>(
        &self,
        descriptor: &ViewCubeDescriptor,
    ) -> SampledCubeUnsignedInteger
    where
        ViewedFormat: ViewFormat<V> + UnsignedIntegerSamplable,
        U: TextureBinding,
    {
        SampledCubeUnsignedInteger {
            inner: self.view_cube_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
        }
    }

    pub fn sampled_cube_depth<ViewedFormat>(
        &self,
        descriptor: &ViewCubeDescriptor,
    ) -> SampledCubeDepth
    where
        ViewedFormat: ViewFormat<V> + DepthSamplable,
        U: TextureBinding,
    {
        SampledCubeDepth {
            inner: self.view_cube_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
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

    pub fn sampled_cube_array_float<ViewedFormat>(
        &self,
        descriptor: &ViewCubeArrayDescriptor,
    ) -> SampledCubeArrayFloat
    where
        ViewedFormat: ViewFormat<V> + FloatSamplable,
        U: TextureBinding,
    {
        SampledCubeArrayFloat {
            inner: self.view_cube_array_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
        }
    }

    pub fn sampled_cube_array_unfilterable_float<ViewedFormat>(
        &self,
        descriptor: &ViewCubeArrayDescriptor,
    ) -> SampledCubeArrayUnfilteredFloat
    where
        ViewedFormat: ViewFormat<V> + UnfilteredFloatSamplable,
        U: TextureBinding,
    {
        SampledCubeArrayUnfilteredFloat {
            inner: self.view_cube_array_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
        }
    }

    pub fn sampled_cube_array_signed_integer<ViewedFormat>(
        &self,
        descriptor: &ViewCubeArrayDescriptor,
    ) -> SampledCubeArraySignedInteger
    where
        ViewedFormat: ViewFormat<V> + SignedIntegerSamplable,
        U: TextureBinding,
    {
        SampledCubeArraySignedInteger {
            inner: self.view_cube_array_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
        }
    }

    pub fn sampled_cube_array_unsigned_integer<ViewedFormat>(
        &self,
        descriptor: &ViewCubeArrayDescriptor,
    ) -> SampledCubeArrayUnsignedInteger
    where
        ViewedFormat: ViewFormat<V> + UnsignedIntegerSamplable,
        U: TextureBinding,
    {
        SampledCubeArrayUnsignedInteger {
            inner: self.view_cube_array_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
        }
    }

    pub fn sampled_cube_array_depth<ViewedFormat>(
        &self,
        descriptor: &ViewCubeArrayDescriptor,
    ) -> SampledCubeArrayDepth
    where
        ViewedFormat: ViewFormat<V> + DepthSamplable,
        U: TextureBinding,
    {
        SampledCubeArrayDepth {
            inner: self.view_cube_array_internal(ViewedFormat::FORMAT_ID.to_web_sys(), descriptor),
            texture_destroyer: self.inner.clone(),
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

    pub fn attachable_image<ViewedFormat>(
        &self,
        descriptor: &AttachableImageDescriptor,
    ) -> AttachableImage<ViewedFormat>
    where
        ViewedFormat: ViewFormat<V> + Renderable,
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

    pub fn storage<ViewedFormat>(&self, descriptor: &Storage2DDescriptor) -> Storage2D<ViewedFormat>
    where
        ViewedFormat: ViewFormat<V> + Storable,
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

    pub fn storage_array<ViewedFormat>(
        &self,
        descriptor: &Storage2DArrayDescriptor,
    ) -> Storage2DArray<ViewedFormat>
    where
        ViewedFormat: ViewFormat<V> + Storable,
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

    fn image_copy_internal(
        &self,
        descriptor: SubImageCopy2DDescriptor,
        bytes_per_block: u32,
        block_size: [u32; 2],
        aspect: GpuTextureAspect,
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

        ImageCopyTexture {
            texture: self.inner.clone(),
            aspect,
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
        assert!(origin_y < self.width, "`y` origin out of bounds");
        assert!(origin_layer < self.width, "layer origin out of bounds");

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

    pub fn image_copy_to_buffer_src(&self, mipmap_level: u8) -> ImageCopyToBufferSrc<F>
    where
        F: ImageCopyToBufferFormat,
        U: CopySrc,
    {
        ImageCopyToBufferSrc {
            inner: self.image_copy_internal(
                SubImageCopy2DDescriptor {
                    mipmap_level,
                    origin_x: 0,
                    origin_y: 0,
                    origin_layer: 0,
                },
                F::BYTES_PER_BLOCK,
                F::BLOCK_SIZE,
                GpuTextureAspect::All,
            ),
        }
    }

    pub fn image_copy_to_buffer_src_depth(&self, mipmap_level: u8) -> ImageCopyToBufferSrc<F>
    where
        F: DepthStencilFormat,
        F::DepthAspect: ImageCopyToBufferFormat,
        U: CopySrc,
    {
        ImageCopyToBufferSrc {
            inner: self.image_copy_internal(
                SubImageCopy2DDescriptor {
                    mipmap_level,
                    origin_x: 0,
                    origin_y: 0,
                    origin_layer: 0,
                },
                F::DepthAspect::BYTES_PER_BLOCK,
                F::DepthAspect::BLOCK_SIZE,
                GpuTextureAspect::DepthOnly,
            ),
        }
    }

    pub fn image_copy_to_buffer_src_stencil(&self, mipmap_level: u8) -> ImageCopyToBufferSrc<F>
    where
        F: DepthStencilFormat,
        F::StencilAspect: ImageCopyToBufferFormat,
        U: CopySrc,
    {
        ImageCopyToBufferSrc {
            inner: self.image_copy_internal(
                SubImageCopy2DDescriptor {
                    mipmap_level,
                    origin_x: 0,
                    origin_y: 0,
                    origin_layer: 0,
                },
                F::StencilAspect::BYTES_PER_BLOCK,
                F::StencilAspect::BLOCK_SIZE,
                GpuTextureAspect::StencilOnly,
            ),
        }
    }

    pub fn image_copy_from_buffer_dst(&self, mipmap_level: u8) -> ImageCopyFromBufferDst<F>
    where
        F: ImageCopyFromBufferFormat,
        U: CopyDst,
    {
        ImageCopyFromBufferDst {
            inner: self.image_copy_internal(
                SubImageCopy2DDescriptor {
                    mipmap_level,
                    origin_x: 0,
                    origin_y: 0,
                    origin_layer: 0,
                },
                F::BYTES_PER_BLOCK,
                F::BLOCK_SIZE,
                GpuTextureAspect::All,
            ),
        }
    }

    // Note: including this function for completeness sake, but this should not currently be
    // invokable, no format meets the constraints.
    pub fn image_copy_from_buffer_dst_depth(&self, mipmap_level: u8) -> ImageCopyFromBufferDst<F>
    where
        F: DepthStencilFormat,
        F::DepthAspect: ImageCopyFromBufferFormat,
        U: CopyDst,
    {
        ImageCopyFromBufferDst {
            inner: self.image_copy_internal(
                SubImageCopy2DDescriptor {
                    mipmap_level,
                    origin_x: 0,
                    origin_y: 0,
                    origin_layer: 0,
                },
                F::DepthAspect::BYTES_PER_BLOCK,
                F::DepthAspect::BLOCK_SIZE,
                GpuTextureAspect::DepthOnly,
            ),
        }
    }

    pub fn image_copy_from_buffer_dst_stencil(&self, mipmap_level: u8) -> ImageCopyFromBufferDst<F>
    where
        F: DepthStencilFormat,
        F::StencilAspect: ImageCopyFromBufferFormat,
        U: CopyDst,
    {
        ImageCopyFromBufferDst {
            inner: self.image_copy_internal(
                SubImageCopy2DDescriptor {
                    mipmap_level,
                    origin_x: 0,
                    origin_y: 0,
                    origin_layer: 0,
                },
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
            inner: self.image_copy_internal(
                SubImageCopy2DDescriptor {
                    mipmap_level,
                    origin_x: 0,
                    origin_y: 0,
                    origin_layer: 0,
                },
                0,
                F::BLOCK_SIZE,
                GpuTextureAspect::All,
            ),
        }
    }

    pub fn image_copy_from_texture_dst(&self, mipmap_level: u8) -> ImageCopyFromTextureDst<F>
    where
        F: ImageCopyTextureFormat,
        U: CopyDst,
    {
        ImageCopyFromTextureDst {
            inner: self.image_copy_internal(
                SubImageCopy2DDescriptor {
                    mipmap_level,
                    origin_x: 0,
                    origin_y: 0,
                    origin_layer: 0,
                },
                0,
                F::BLOCK_SIZE,
                GpuTextureAspect::All,
            ),
        }
    }

    pub fn sub_image_copy_to_buffer_src(
        &self,
        descriptor: SubImageCopy2DDescriptor,
    ) -> SubImageCopyToBufferSrc<F>
    where
        F: ImageCopyToBufferFormat + SubImageCopyFormat,
        U: CopySrc,
    {
        SubImageCopyToBufferSrc {
            inner: self.sub_image_copy_internal(descriptor, F::BYTES_PER_BLOCK, F::BLOCK_SIZE),
        }
    }

    pub fn sub_image_copy_from_buffer_dst(
        &self,
        descriptor: SubImageCopy2DDescriptor,
    ) -> SubImageCopyFromBufferDst<F>
    where
        F: ImageCopyFromBufferFormat + SubImageCopyFormat,
        U: CopyDst,
    {
        SubImageCopyFromBufferDst {
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
