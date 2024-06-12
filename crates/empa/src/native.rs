use std::fmt::Debug;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;
use std::{error, fmt};

use arrayvec::ArrayVec;
use flagset::{flags, FlagSet};
use raw_window_handle::{
    HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use wgc::gfx_select;
use wgc::global::Global;
use wgc::id::SurfaceId;
use wgc::present::SurfaceOutput;
use wgt::SurfaceStatus;

use crate::adapter::Adapter;
use crate::device::Device;
use crate::driver::native::{texture_format_to_wgc, texture_usage_to_wgc};
use crate::texture::format::{TextureFormat, TextureFormatId, ViewFormats};
use crate::texture::Texture2D;
use crate::{driver, texture};

flags! {
    pub enum Backend: u32 {
        None   = 0,
        Vulkan = 1 << 0,
        Dx12   = 1 << 1,
        Metal  = 1 << 2,
        All    = (Backend::Vulkan | Backend::Dx12 | Backend::Metal).bits(),
    }

    pub enum InstanceFlag: u32 {
        None                               = 0,
        Debug                              = 1 << 0,
        Validation                         = 1 << 1,
        DiscardHalLabels                   = 1 << 2,
        AllowUnderlyingNoncompliantAdapter = 1 << 3,
        GpuBasedValidation                 = 1 << 4,
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
pub enum Dx12ShaderCompiler {
    #[default]
    Fxc,
    Dxc {
        dxil_path: Option<PathBuf>,
        dxc_path: Option<PathBuf>,
    },
}

pub struct InstanceDescriptor<B, F> {
    pub backends: B,
    pub flags: F,
    pub dx12_shader_comiler: Dx12ShaderCompiler,
}

impl Default for InstanceDescriptor<FlagSet<Backend>, FlagSet<InstanceFlag>> {
    fn default() -> Self {
        InstanceDescriptor {
            backends: Backend::All.into(),
            flags: InstanceFlag::None.into(),
            dx12_shader_comiler: Default::default(),
        }
    }
}

pub struct RawSurfaceHandles {
    pub raw_display_handle: RawDisplayHandle,
    pub raw_window_handle: RawWindowHandle,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PowerPreference {
    #[default]
    DontCare,
    LowPower,
    HighPerformance,
}

pub struct AdapterOptions<'a, 'b> {
    pub power_preference: PowerPreference,
    pub force_fallback_adapter: bool,
    pub compatible_surface: Option<&'a Surface<'b>>,
}

impl Default for AdapterOptions<'_, '_> {
    fn default() -> Self {
        AdapterOptions {
            power_preference: Default::default(),
            force_fallback_adapter: false,
            compatible_surface: None,
        }
    }
}

pub struct Instance {
    global: Arc<Global>,
}

impl Instance {
    pub fn enabled_backends() -> FlagSet<Backend> {
        let mut backends = FlagSet::from(Backend::None);

        if cfg!(feature = "dx12") {
            backends |= Backend::Dx12;
        }

        if cfg!(feature = "metal") {
            backends |= Backend::Metal;
        }

        // Windows, Android, Linux currently always enable Vulkan.
        // See <https://github.com/gfx-rs/wgpu/issues/3514>
        if cfg!(target_os = "windows") || cfg!(unix) {
            backends |= Backend::Vulkan;
        }

        backends
    }

    pub fn new<B, F>(descriptor: InstanceDescriptor<B, F>) -> Self
    where
        B: Into<FlagSet<Backend>> + Copy,
        F: Into<FlagSet<InstanceFlag>> + Copy,
    {
        let requested_backends = descriptor.backends.into();

        if requested_backends.is_empty() {
            panic!("must request at least one backend");
        }

        if requested_backends.is_disjoint(Self::enabled_backends()) {
            panic!("None of the requested backends are enabled/available.");
        }

        let global = Global::new(
            "wgpu",
            wgt::InstanceDescriptor {
                backends: backends_to_wgc(requested_backends),
                flags: instance_flags_to_wgc(descriptor.flags.into()),
                dx12_shader_compiler: dx12_shader_compiler_to_wgc(descriptor.dx12_shader_comiler),
                gles_minor_version: Default::default(),
            },
        );

        Instance {
            global: Arc::new(global),
        }
    }

    pub fn create_surface<'a, T>(&self, window_handle: T) -> Result<Surface<'a>, CreateSurfaceError>
    where
        T: WindowHandle + 'a,
    {
        let raw_display_handle = window_handle
            .display_handle()
            .map(|h| h.as_raw())
            .map_err(|err| CreateSurfaceError { inner: err.into() })?;

        let raw_window_handle = window_handle
            .window_handle()
            .map(|h| h.as_raw())
            .map_err(|err| CreateSurfaceError { inner: err.into() })?;

        let mut surface = unsafe {
            self.create_surface_unsafe(RawSurfaceHandles {
                raw_display_handle,
                raw_window_handle,
            })?
        };

        surface._window_handle = Some(Box::new(window_handle));

        Ok(surface)
    }

    pub unsafe fn create_surface_unsafe(
        &self,
        raw_surface_handles: RawSurfaceHandles,
    ) -> Result<Surface<'static>, CreateSurfaceError> {
        let id = unsafe {
            self.global
                .instance_create_surface(
                    raw_surface_handles.raw_display_handle,
                    raw_surface_handles.raw_window_handle,
                    None,
                )
                .map_err(|err| CreateSurfaceError { inner: err.into() })?
        };

        Ok(Surface {
            global: self.global.clone(),
            id,
            _window_handle: None,
        })
    }

    pub fn get_adapter(&self, options: AdapterOptions) -> Result<Adapter, GetAdapterError> {
        let descriptor = wgc::instance::RequestAdapterOptions {
            power_preference: power_preference_to_wgc(&options.power_preference),
            force_fallback_adapter: options.force_fallback_adapter,
            compatible_surface: options.compatible_surface.map(|surface| surface.id),
        };

        self.global
            .request_adapter(
                &descriptor,
                wgc::instance::AdapterInputs::Mask(wgt::Backends::all(), |_| None),
            )
            .map(|id| {
                Adapter::from_handle(driver::native::AdapterHandle::new(self.global.clone(), id))
            })
            .map_err(|inner| GetAdapterError { inner })
    }

    pub fn poll_all(&self, force_wait: bool) -> bool {
        match self.global.poll_all_devices(force_wait) {
            Ok(all_queue_empty) => all_queue_empty,
            Err(err) => panic!("{}", err),
        }
    }
}

#[derive(Clone, Debug)]
pub struct GetAdapterError {
    inner: wgc::instance::RequestAdapterError,
}

impl fmt::Display for GetAdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl error::Error for GetAdapterError {}

impl Default for Instance {
    fn default() -> Self {
        Instance::new(Default::default())
    }
}

#[derive(Clone, Debug)]
enum CreateSurfaceErrorKind {
    Wgc(wgc::instance::CreateSurfaceError),
    Handle(HandleError),
}

impl From<wgc::instance::CreateSurfaceError> for CreateSurfaceErrorKind {
    fn from(value: wgc::instance::CreateSurfaceError) -> Self {
        CreateSurfaceErrorKind::Wgc(value)
    }
}

impl From<HandleError> for CreateSurfaceErrorKind {
    fn from(value: HandleError) -> Self {
        CreateSurfaceErrorKind::Handle(value)
    }
}

#[derive(Clone, Debug)]
pub struct CreateSurfaceError {
    inner: CreateSurfaceErrorKind,
}

impl fmt::Display for CreateSurfaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner {
            CreateSurfaceErrorKind::Wgc(err) => fmt::Display::fmt(err, f),
            CreateSurfaceErrorKind::Handle(err) => fmt::Display::fmt(err, f),
        }
    }
}

impl error::Error for CreateSurfaceError {}

pub trait WindowHandle: HasWindowHandle + HasDisplayHandle {}

impl<T> WindowHandle for T where T: HasWindowHandle + HasDisplayHandle {}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PresentMode {
    AutoVsync,
    AutoNoVsync,
    #[default]
    Fifo,
    FifoRelaxed,
    Immediate,
    Mailbox,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AlphaMode {
    #[default]
    Auto,
    Opaque,
    PreMultiplied,
    PostMultiplied,
    Inherit,
}

pub struct SurfaceConfiguration<F, U, V> {
    pub format: F,
    pub usage: U,
    pub width: u32,
    pub height: u32,
    pub present_mode: PresentMode,
    pub desired_maximum_frame_latency: u32,
    pub alpha_mode: AlphaMode,
    pub view_formats: V,
}

pub struct Surface<'a> {
    global: Arc<Global>,
    id: SurfaceId,
    _window_handle: Option<Box<dyn WindowHandle + 'a>>,
}

impl<'a> Surface<'a> {
    pub fn configure<F, U, V>(
        self,
        device: &Device,
        config: &SurfaceConfiguration<F, U, V>,
    ) -> ConfiguredSurface<'a, F, U>
    where
        F: TextureFormat + Copy,
        U: texture::UsageFlags,
        V: ViewFormats<F>,
    {
        let err = gfx_select!(device.device_handle.id() => self.global.surface_configure(self.id, device.device_handle.id(), &surface_configuration_to_wgc(config)));

        if let Some(err) = err {
            panic!("{}", err);
        }

        ConfiguredSurface {
            surface: self,
            device: device.clone(),
            width: config.width,
            height: config.height,
            present_mode: config.present_mode,
            desired_maximum_frame_latency: config.desired_maximum_frame_latency,
            alpha_mode: config.alpha_mode,
            view_formats: config.view_formats.formats().collect(),
            _format: config.format,
            usage: config.usage,
        }
    }
}

pub struct ConfiguredSurface<'a, F, U> {
    device: Device,
    surface: Surface<'a>,
    width: u32,
    height: u32,
    present_mode: PresentMode,
    desired_maximum_frame_latency: u32,
    alpha_mode: AlphaMode,
    view_formats: ArrayVec<TextureFormatId, 8>,
    _format: F,
    usage: U,
}

impl<'a, F, U> ConfiguredSurface<'a, F, U>
where
    F: TextureFormat,
    U: texture::UsageFlags,
{
    pub fn resize(&mut self, width: u32, height: u32) {
        let ConfiguredSurface {
            device,
            surface,
            present_mode,
            desired_maximum_frame_latency,
            alpha_mode,
            view_formats,
            ..
        } = self;

        let view_formats = view_formats.iter().map(texture_format_to_wgc).collect();

        let err = gfx_select!(device.device_handle.id() => surface.global.surface_configure(surface.id, device.device_handle.id(), &wgt::SurfaceConfiguration {
            usage: texture_usage_to_wgc(&U::FLAG_SET),
            format: texture_format_to_wgc(&F::FORMAT_ID),
            width,
            height,
            present_mode: present_mode_to_wgc(present_mode),
            desired_maximum_frame_latency: *desired_maximum_frame_latency,
            alpha_mode: alpha_mode_to_wgc(alpha_mode),
            view_formats,
        }));

        if let Some(err) = err {
            panic!("{}", err);
        }

        self.width = width;
        self.height = height;
    }

    pub fn get_current_texture(&self) -> Result<SurfaceTexture<F, U>, SurfaceError> {
        let surface = &self.surface;
        let res = gfx_select!(self.device.device_handle.id() => surface.global.surface_get_current_texture(self.surface.id, None));

        match res {
            Ok(SurfaceOutput { status, texture_id }) => {
                let suboptimal = match status {
                    SurfaceStatus::Good => false,
                    SurfaceStatus::Suboptimal => true,
                    SurfaceStatus::Timeout => return Err(SurfaceError::Timeout),
                    SurfaceStatus::Outdated => return Err(SurfaceError::Outdated),
                    SurfaceStatus::Lost => return Err(SurfaceError::Lost),
                };

                let texture_id = if let Some(id) = texture_id {
                    id
                } else {
                    return Err(SurfaceError::Lost);
                };

                let texture = Texture2D::from_swap_chain_texture(
                    driver::native::TextureHandle::swap_chain(
                        self.surface.global.clone(),
                        texture_id,
                    ),
                    self.width,
                    self.height,
                    self.view_formats.as_slice(),
                    self.usage,
                );

                Ok(SurfaceTexture {
                    global: surface.global.clone(),
                    surface_id: surface.id,
                    texture,
                    suboptimal,
                })
            }
            Err(err) => panic!("{}", err),
        }
    }

    pub fn unconfigure(self) -> Surface<'a> {
        self.surface
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SurfaceError {
    Timeout,
    Outdated,
    Lost,
}

pub struct SurfaceTexture<F, U> {
    global: Arc<Global>,
    surface_id: SurfaceId,
    texture: Texture2D<F, U>,
    suboptimal: bool,
}

impl<F, U> SurfaceTexture<F, U> {
    pub fn suboptimal(&self) -> bool {
        self.suboptimal
    }

    pub fn present(self) {
        let res =
            gfx_select!(self.texture.handle.id() => self.global.surface_present(self.surface_id));

        if let Err(err) = res {
            panic!("{}", err);
        }
    }
}

impl<F, U> Deref for SurfaceTexture<F, U> {
    type Target = Texture2D<F, U>;

    fn deref(&self) -> &Self::Target {
        &self.texture
    }
}

fn backends_to_wgc(backends: FlagSet<Backend>) -> wgt::Backends {
    let mut res = wgt::Backends::empty();

    if backends.contains(Backend::Metal) {
        res |= wgt::Backends::METAL;
    }

    if backends.contains(Backend::Dx12) {
        res |= wgt::Backends::DX12;
    }

    if backends.contains(Backend::Vulkan) {
        res |= wgt::Backends::VULKAN;
    }

    res
}

fn instance_flags_to_wgc(flags: FlagSet<InstanceFlag>) -> wgt::InstanceFlags {
    let mut res = wgt::InstanceFlags::empty();

    if flags.contains(InstanceFlag::Debug) {
        res |= wgt::InstanceFlags::DEBUG;
    }

    if flags.contains(InstanceFlag::Validation) {
        res |= wgt::InstanceFlags::VALIDATION;
    }

    if flags.contains(InstanceFlag::DiscardHalLabels) {
        res |= wgt::InstanceFlags::DISCARD_HAL_LABELS;
    }

    if flags.contains(InstanceFlag::AllowUnderlyingNoncompliantAdapter) {
        res |= wgt::InstanceFlags::ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER;
    }

    if flags.contains(InstanceFlag::GpuBasedValidation) {
        res |= wgt::InstanceFlags::GPU_BASED_VALIDATION;
    }

    res
}

fn dx12_shader_compiler_to_wgc(dx12shader_compiler: Dx12ShaderCompiler) -> wgt::Dx12Compiler {
    match dx12shader_compiler {
        Dx12ShaderCompiler::Fxc => wgt::Dx12Compiler::Fxc,
        Dx12ShaderCompiler::Dxc {
            dxil_path,
            dxc_path,
        } => wgt::Dx12Compiler::Dxc {
            dxil_path,
            dxc_path,
        },
    }
}

fn present_mode_to_wgc(present_mode: &PresentMode) -> wgt::PresentMode {
    match present_mode {
        PresentMode::AutoVsync => wgt::PresentMode::AutoVsync,
        PresentMode::AutoNoVsync => wgt::PresentMode::AutoNoVsync,
        PresentMode::Fifo => wgt::PresentMode::Fifo,
        PresentMode::FifoRelaxed => wgt::PresentMode::FifoRelaxed,
        PresentMode::Immediate => wgt::PresentMode::Immediate,
        PresentMode::Mailbox => wgt::PresentMode::Mailbox,
    }
}

fn alpha_mode_to_wgc(alpha_mode: &AlphaMode) -> wgt::CompositeAlphaMode {
    match alpha_mode {
        AlphaMode::Auto => wgt::CompositeAlphaMode::Auto,
        AlphaMode::Opaque => wgt::CompositeAlphaMode::Opaque,
        AlphaMode::PreMultiplied => wgt::CompositeAlphaMode::PreMultiplied,
        AlphaMode::PostMultiplied => wgt::CompositeAlphaMode::PostMultiplied,
        AlphaMode::Inherit => wgt::CompositeAlphaMode::Inherit,
    }
}

fn surface_configuration_to_wgc<F, U, V>(
    surface_configuration: &SurfaceConfiguration<F, U, V>,
) -> wgt::SurfaceConfiguration<Vec<wgt::TextureFormat>>
where
    F: TextureFormat,
    U: texture::UsageFlags,
    V: ViewFormats<F>,
{
    let view_formats = surface_configuration
        .view_formats
        .formats()
        .map(|f| texture_format_to_wgc(&f))
        .collect();

    wgt::SurfaceConfiguration {
        usage: texture_usage_to_wgc(&U::FLAG_SET),
        format: texture_format_to_wgc(&F::FORMAT_ID),
        width: surface_configuration.width,
        height: surface_configuration.height,
        present_mode: present_mode_to_wgc(&surface_configuration.present_mode),
        desired_maximum_frame_latency: surface_configuration.desired_maximum_frame_latency,
        alpha_mode: alpha_mode_to_wgc(&surface_configuration.alpha_mode),
        view_formats,
    }
}

fn power_preference_to_wgc(power_preference: &PowerPreference) -> wgt::PowerPreference {
    match power_preference {
        PowerPreference::DontCare => wgt::PowerPreference::None,
        PowerPreference::LowPower => wgt::PowerPreference::LowPower,
        PowerPreference::HighPerformance => wgt::PowerPreference::HighPerformance,
    }
}
