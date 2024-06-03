use std::future::Future;
use std::marker;
use std::pin::Pin;
use std::task::{Context, Poll};

use arrayvec::ArrayVec;
use arwa::html::HtmlCanvasElement;
use arwa::image_bitmap::ImageBitmap;
use arwa::window::WindowNavigator;
use arwa::worker::WorkerNavigator;
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Gpu, GpuCanvasAlphaMode, GpuCanvasConfiguration, GpuCanvasContext, GpuImageCopyExternalImage,
    GpuImageCopyTextureTagged, GpuOrigin2dDict, GpuOrigin3dDict, GpuPowerPreference,
    GpuRequestAdapterOptions,
};

use crate::adapter::Adapter;
use crate::device::{Device, Queue};
use crate::driver::web::{size_3d_to_web_sys, texture_format_to_str, texture_format_to_web_sys};
use crate::texture;
use crate::texture::format::{
    bgra8unorm, bgra8unorm_srgb, r16float, r32float, r8unorm, rg16float, rg32float, rg8unorm,
    rgb10a2unorm, rgba16float, rgba32float, rgba8unorm, rgba8unorm_srgb, TextureFormat,
    TextureFormatId, ViewFormats,
};
use crate::texture::{ImageCopySize2D, Texture2D};

mod navigator_ext_seal {
    pub trait Seal {}
}

pub trait NavigatorExt: navigator_ext_seal::Seal {
    fn empa(&self) -> Empa;
}

impl navigator_ext_seal::Seal for WindowNavigator {}
impl NavigatorExt for WindowNavigator {
    fn empa(&self) -> Empa {
        let as_web_sys: &web_sys::Navigator = self.as_ref();

        Empa {
            inner: as_web_sys.gpu(),
        }
    }
}

impl navigator_ext_seal::Seal for WorkerNavigator {}
impl NavigatorExt for WorkerNavigator {
    fn empa(&self) -> Empa {
        let as_web_sys: &web_sys::WorkerNavigator = self.as_ref();

        Empa {
            inner: as_web_sys.gpu(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PowerPreference {
    DontCare,
    LowPower,
    HighPerformance,
}

impl Default for PowerPreference {
    fn default() -> Self {
        PowerPreference::DontCare
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RequestAdapterOptions {
    pub power_preference: PowerPreference,
    pub force_fallback_adapter: bool,
}

impl Default for RequestAdapterOptions {
    fn default() -> Self {
        RequestAdapterOptions {
            power_preference: Default::default(),
            force_fallback_adapter: false,
        }
    }
}

pub struct Empa {
    inner: Gpu,
}

impl Empa {
    pub fn request_adapter(&self, options: &RequestAdapterOptions) -> RequestAdapter {
        let RequestAdapterOptions {
            power_preference,
            force_fallback_adapter,
        } = *options;

        let mut opts = GpuRequestAdapterOptions::new();

        match power_preference {
            PowerPreference::LowPower => {
                opts.power_preference(GpuPowerPreference::LowPower);
            }
            PowerPreference::HighPerformance => {
                opts.power_preference(GpuPowerPreference::HighPerformance);
            }
            PowerPreference::DontCare => (),
        }

        opts.force_fallback_adapter(force_fallback_adapter);

        let promise = self.inner.request_adapter_with_options(&opts);

        RequestAdapter {
            inner: JsFuture::from(promise),
        }
    }
}

pub struct RequestAdapter {
    inner: JsFuture,
}

impl Future for RequestAdapter {
    type Output = Option<Adapter>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.get_mut().inner).poll(cx).map(|result| {
            result.ok().and_then(|v| {
                if v.is_null() {
                    None
                } else {
                    Some(Adapter::from_handle(
                        v.unchecked_into::<web_sys::GpuAdapter>().into(),
                    ))
                }
            })
        })
    }
}

pub trait CanvasContextFormat: TextureFormat {}

impl CanvasContextFormat for bgra8unorm {}
impl CanvasContextFormat for rgba8unorm {}
impl CanvasContextFormat for rgba16float {}

// TODO: ignoring for now, this type is now part of an update to the canvas spec that is not yet in
// web-sys
//
// #[derive(Clone, Copy, PartialEq, Eq, Debug)]
// #[allow(non_camel_case_types)]
// pub enum PredefinedColorSpace {
//     srgb,
// }
//
// impl PredefinedColorSpace {
//     fn to_web_sys(&self) -> GpuPredefinedColorSpace {
//         match self {
//             PredefinedColorSpace::srgb => GpuPredefinedColorSpace::Srgb,
//         }
//     }
// }
//
// impl Default for PredefinedColorSpace {
//     fn default() -> Self {
//         PredefinedColorSpace::srgb
//     }
// }

pub enum AlphaMode {
    Opaque,
    Premultiplied,
}

impl AlphaMode {
    fn to_web_sys(&self) -> GpuCanvasAlphaMode {
        match self {
            AlphaMode::Opaque => GpuCanvasAlphaMode::Opaque,
            AlphaMode::Premultiplied => GpuCanvasAlphaMode::Premultiplied,
        }
    }
}

pub struct CanvasConfiguration<'a, F, U, V>
where
    F: CanvasContextFormat,
    U: texture::UsageFlags,
    V: ViewFormats<F>,
{
    pub device: &'a Device,
    pub format: F,
    pub usage: U,
    pub view_formats: V,
    // Ignoring for now, see comment above.
    // pub color_space: PredefinedColorSpace,
    pub alpha_mode: AlphaMode,
}

pub struct CanvasContext {
    inner: GpuCanvasContext,
    canvas: HtmlCanvasElement,
}

impl CanvasContext {
    pub fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }

    pub fn configure<F, U, V>(
        self,
        configuration: &CanvasConfiguration<F, U, V>,
    ) -> ConfiguredCanvasContext<F, U>
    where
        F: CanvasContextFormat,
        U: texture::UsageFlags,
        V: ViewFormats<F>,
    {
        let CanvasConfiguration {
            device,
            view_formats,
            // Ignoring for now, see comment above.
            // color_space,
            alpha_mode,
            usage,
            ..
        } = configuration;

        let mut config = GpuCanvasConfiguration::new(
            &device.handle.inner,
            texture_format_to_web_sys(&F::FORMAT_ID),
        );

        config.usage(U::FLAG_SET.bits());

        let formats = js_sys::Array::new();

        for format in view_formats.formats() {
            formats.push(&JsValue::from(texture_format_to_str(&format)));
        }

        // TODO: view formats not in web-sys

        // Ignoring for now, see comment above.
        // config.color_space(color_space.to_web_sys());
        config.alpha_mode(alpha_mode.to_web_sys());

        self.inner.configure(&config);

        ConfiguredCanvasContext {
            inner: self.inner,
            canvas: self.canvas,
            view_formats: view_formats.formats().collect(),
            usage: *usage,
            _marker: Default::default(),
        }
    }
}

pub struct ConfiguredCanvasContext<F, U> {
    inner: GpuCanvasContext,
    canvas: HtmlCanvasElement,
    view_formats: ArrayVec<TextureFormatId, 8>,
    usage: U,
    _marker: marker::PhantomData<F>,
}

impl<F, U> ConfiguredCanvasContext<F, U>
where
    F: CanvasContextFormat,
    U: texture::UsageFlags,
{
    pub fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }

    pub fn get_current_texture(&self) -> Texture2D<F, U> {
        Texture2D::from_swap_chain_texture(
            self.inner.get_current_texture().into(),
            self.canvas.width(),
            self.canvas.height(),
            &self.view_formats,
            self.usage,
        )
    }

    pub fn unconfigure(self) -> CanvasContext {
        let ConfiguredCanvasContext { inner, canvas, .. } = self;

        inner.unconfigure();

        CanvasContext { inner, canvas }
    }
}

mod html_canvas_element_ext_seal {
    pub trait Seal {}
}

pub trait HtmlCanvasElementExt: html_canvas_element_ext_seal::Seal {
    fn empa_context(&self) -> CanvasContext;
}

impl html_canvas_element_ext_seal::Seal for HtmlCanvasElement {}
impl HtmlCanvasElementExt for HtmlCanvasElement {
    fn empa_context(&self) -> CanvasContext {
        let as_web_sys: &web_sys::HtmlCanvasElement = self.as_ref();
        let inner = as_web_sys
            .get_context("webgpu")
            .unwrap_throw()
            .unwrap_throw();

        CanvasContext {
            inner: inner.unchecked_into(),
            canvas: self.clone(),
        }
    }
}

mod queue_ext_seal {
    pub trait Seal {}
}

pub trait QueueExt: queue_ext_seal::Seal {
    fn copy_external_image_to_texture(
        &self,
        src: &ExternalImageCopySrc,
        dst: &ExternalImageCopyDst,
        size: ImageCopySize2D,
    );
}

impl queue_ext_seal::Seal for Queue {}
impl QueueExt for Queue {
    fn copy_external_image_to_texture(
        &self,
        src: &ExternalImageCopySrc,
        dst: &ExternalImageCopyDst,
        size: ImageCopySize2D,
    ) {
        let ImageCopySize2D { width, height } = size;

        assert!(width <= src.width, "copy width outside of `src` bounds");
        assert!(height <= src.height, "copy height outside of `src` bounds");
        assert!(width <= dst.width, "copy width outside of `dst` bounds");
        assert!(height <= dst.height, "copy height outside of `dst` bounds");

        self.handle
            .inner
            .copy_external_image_to_texture_with_gpu_extent_3d_dict(
                &src.inner,
                &dst.inner,
                &size_3d_to_web_sys((width, height, 1)),
            );
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ExternalImageCopySrcOptions {
    pub origin_x: u32,
    pub origin_y: u32,
    pub flip_y: bool,
}

pub struct ExternalImageCopySrc {
    inner: GpuImageCopyExternalImage,
    width: u32,
    height: u32,
}

impl ExternalImageCopySrc {
    fn new(
        source: &js_sys::Object,
        options: ExternalImageCopySrcOptions,
        width: u32,
        height: u32,
    ) -> Self {
        let ExternalImageCopySrcOptions {
            origin_x,
            origin_y,
            flip_y,
        } = options;

        let mut origin = GpuOrigin2dDict::new();

        origin.x(origin_x);
        origin.y(origin_y);

        let mut inner = GpuImageCopyExternalImage::new(source);

        inner.origin(origin.as_ref());
        inner.flip_y(flip_y);

        ExternalImageCopySrc {
            inner,
            width,
            height,
        }
    }

    pub fn image_bitmap(image_bitmap: &ImageBitmap, options: ExternalImageCopySrcOptions) -> Self {
        let width = image_bitmap.width();
        let height = image_bitmap.height();

        validate_size_origin(width, height, options.origin_x, options.origin_y);

        Self::new(image_bitmap.as_ref(), options, width, height)
    }

    pub fn html_canvas_element(
        html_canvas_element: &HtmlCanvasElement,
        options: ExternalImageCopySrcOptions,
    ) -> Self {
        let width = html_canvas_element.width();
        let height = html_canvas_element.height();

        validate_size_origin(width, height, options.origin_x, options.origin_y);

        Self::new(html_canvas_element.as_ref(), options, width, height)
    }
}

fn validate_size_origin(width: u32, height: u32, origin_x: u32, origin_y: u32) {
    assert!(width > 0, "the image width must be greater than `0`");
    assert!(height > 0, "the image height must be greater than `0`");
    assert!(
        origin_x < width,
        "the `x` origin must be less than the width"
    );
    assert!(
        origin_y < height,
        "the `y` origin must be less than the height"
    );
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ExternalImageCopyDstDescriptor {
    pub mipmap_level: u8,
    // Ignoring for now, see comment above.
    // pub color_space: PredefinedColorSpace,
    pub premultiplied_alpha: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ExternalSubImageCopyDstDescriptor {
    pub mipmap_level: u8,
    pub origin_x: u32,
    pub origin_y: u32,
    pub origin_layer: u32,
    // Ignoring for now, see comment above.
    // pub color_space: PredefinedColorSpace,
    pub premultiplied_alpha: bool,
}

pub struct ExternalImageCopyDst {
    inner: GpuImageCopyTextureTagged,
    width: u32,
    height: u32,
}

pub trait ExternalImageCopyFormat: TextureFormat {}

impl ExternalImageCopyFormat for r8unorm {}
impl ExternalImageCopyFormat for r16float {}
impl ExternalImageCopyFormat for r32float {}
impl ExternalImageCopyFormat for rg8unorm {}
impl ExternalImageCopyFormat for rg16float {}
impl ExternalImageCopyFormat for rg32float {}
impl ExternalImageCopyFormat for rgba8unorm {}
impl ExternalImageCopyFormat for rgba8unorm_srgb {}
impl ExternalImageCopyFormat for bgra8unorm {}
impl ExternalImageCopyFormat for bgra8unorm_srgb {}
impl ExternalImageCopyFormat for rgb10a2unorm {}
impl ExternalImageCopyFormat for rgba16float {}
impl ExternalImageCopyFormat for rgba32float {}

mod texture_2d_ext_seal {
    pub trait Seal {}
}

pub trait Texture2DExt<F, U>: texture_2d_ext_seal::Seal {
    fn external_image_copy_dst(
        &self,
        descriptor: ExternalImageCopyDstDescriptor,
    ) -> ExternalImageCopyDst
    where
        F: ExternalImageCopyFormat,
        U: texture::CopyDst + texture::RenderAttachment;

    fn external_sub_image_copy_dst(
        &self,
        descriptor: ExternalSubImageCopyDstDescriptor,
    ) -> ExternalImageCopyDst
    where
        F: ExternalImageCopyFormat,
        U: texture::CopyDst + texture::RenderAttachment;
}

impl<F, U> texture_2d_ext_seal::Seal for Texture2D<F, U> {}
impl<F, U> Texture2DExt<F, U> for Texture2D<F, U> {
    fn external_image_copy_dst(
        &self,
        descriptor: ExternalImageCopyDstDescriptor,
    ) -> ExternalImageCopyDst
    where
        F: ExternalImageCopyFormat,
        U: texture::CopyDst + texture::RenderAttachment,
    {
        let ExternalImageCopyDstDescriptor {
            mipmap_level,
            // Ignoring for now, see comment above
            // color_space,
            premultiplied_alpha,
        } = descriptor;

        assert!(
            mipmap_level < self.mip_level_count,
            "mipmap level out of bounds"
        );

        let mut inner = GpuImageCopyTextureTagged::new(&self.handle.inner);

        inner.mip_level(mipmap_level as u32);
        // Ignoring for now, see comment above
        // inner.color_space(color_space.to_web_sys());
        inner.premultiplied_alpha(premultiplied_alpha);

        ExternalImageCopyDst {
            inner,
            width: self.width,
            height: self.height,
        }
    }

    fn external_sub_image_copy_dst(
        &self,
        descriptor: ExternalSubImageCopyDstDescriptor,
    ) -> ExternalImageCopyDst
    where
        F: ExternalImageCopyFormat,
        U: texture::CopyDst + texture::RenderAttachment,
    {
        let ExternalSubImageCopyDstDescriptor {
            mipmap_level,
            origin_x,
            origin_y,
            origin_layer,
            // Ignoring for now, see comment above
            // color_space,
            premultiplied_alpha,
        } = descriptor;

        assert!(
            mipmap_level < self.mip_level_count,
            "mipmap level out of bounds"
        );
        assert!(origin_x < self.width, "`x` origin out of bounds");
        assert!(origin_y < self.height, "`y` origin out of bounds");
        assert!(origin_layer < self.layers, "layer origin out of bounds");

        let mut origin = GpuOrigin3dDict::new();

        origin.x(origin_x);
        origin.y(origin_y);
        origin.z(origin_layer);

        let mut inner = GpuImageCopyTextureTagged::new(&self.handle.inner);

        inner.origin(origin.as_ref());
        inner.mip_level(mipmap_level as u32);
        // Ignoring for now, see comment above
        // inner.color_space(color_space.to_web_sys());
        inner.premultiplied_alpha(premultiplied_alpha);

        ExternalImageCopyDst {
            inner,
            width: self.width - origin_x,
            height: self.height - origin_y,
        }
    }
}
