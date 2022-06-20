use std::future::Future;
use std::marker;
use std::pin::Pin;
use std::task::{Context, Poll};

use arwa::html::HtmlCanvasElement;
use arwa::window::WindowNavigator;
use arwa::worker::WorkerNavigator;
use staticvec::StaticVec;
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Gpu, GpuCanvasCompositingAlphaMode, GpuCanvasConfiguration, GpuCanvasContext,
    GpuPowerPreference, GpuPredefinedColorSpace, GpuRequestAdapterOptions,
};

use crate::adapter::Adapter;
use crate::device::Device;
use crate::texture;
use crate::texture::format::{
    bgra8unorm, rgba16float, rgba8unorm, TextureFormat, TextureFormatId, ViewFormats,
};
use crate::texture::Texture2D;

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
                    Some(Adapter::from_web_sys(v.unchecked_into()))
                }
            })
        })
    }
}

pub trait CanvasContextFormat: TextureFormat {}

impl CanvasContextFormat for bgra8unorm {}
impl CanvasContextFormat for rgba8unorm {}
impl CanvasContextFormat for rgba16float {}

#[allow(non_camel_case_types)]
pub enum PredefinedColorSpace {
    srgb,
}

impl PredefinedColorSpace {
    fn to_web_sys(&self) -> GpuPredefinedColorSpace {
        match self {
            PredefinedColorSpace::srgb => GpuPredefinedColorSpace::Srgb,
        }
    }
}

pub enum CompositingAlphaMode {
    Opaque,
    Premultiplied,
}

impl CompositingAlphaMode {
    fn to_web_sys(&self) -> GpuCanvasCompositingAlphaMode {
        match self {
            CompositingAlphaMode::Opaque => GpuCanvasCompositingAlphaMode::Opaque,
            CompositingAlphaMode::Premultiplied => GpuCanvasCompositingAlphaMode::Premultiplied,
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
    pub color_space: PredefinedColorSpace,
    pub compositing_alpha_mode: CompositingAlphaMode,
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
            color_space,
            compositing_alpha_mode,
            ..
        } = configuration;

        let mut config =
            GpuCanvasConfiguration::new(device.as_web_sys(), F::FORMAT_ID.to_web_sys());

        config.usage(U::BITS);

        let formats = js_sys::Array::new();

        for format in view_formats.formats() {
            formats.push(&JsValue::from(format.as_str()));
        }

        // TODO: view formats not in web-sys

        config.color_space(color_space.to_web_sys());
        config.compositing_alpha_mode(compositing_alpha_mode.to_web_sys());

        self.inner.configure(&config);

        ConfiguredCanvasContext {
            inner: self.inner,
            canvas: self.canvas,
            view_formats: view_formats.formats().collect(),
            _marker: Default::default(),
        }
    }
}

pub struct ConfiguredCanvasContext<F, U> {
    inner: GpuCanvasContext,
    canvas: HtmlCanvasElement,
    view_formats: StaticVec<TextureFormatId, 8>,
    _marker: marker::PhantomData<(F, U)>,
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
        unsafe {
            Texture2D::from_swap_chain_texture(
                self.inner.get_current_texture(),
                self.canvas.width(),
                self.canvas.height(),
                &self.view_formats,
            )
        }
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
