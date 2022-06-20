mod attachment;
pub use self::attachment::*;

mod multisample_attachment;
pub use self::attachment::*;

mod render_layout;
pub use self::render_layout::*;

mod render_target;
pub use self::render_target::*;

use std::error::Error;
use std::fmt;

use crate::texture::TextureDestroyer;
use std::sync::Arc;
use web_sys::{GpuColorDict, GpuLoadOp, GpuStoreOp};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct InvalidDepthValue(f32);

impl fmt::Display for InvalidDepthValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "`{}` is not a valid depth value; must by in the range `0.0..=1.0`",
            self.0
        )
    }
}

impl Error for InvalidDepthValue {}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct DepthValue(pub(crate) f32);

impl DepthValue {
    pub const ZERO: DepthValue = DepthValue(0.0);

    pub const ONE: DepthValue = DepthValue(1.0);
}

impl TryFrom<f32> for DepthValue {
    type Error = InvalidDepthValue;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value >= 0.0 && value <= 1.0 {
            Ok(DepthValue(value))
        } else {
            Err(InvalidDepthValue(value))
        }
    }
}

pub enum LoadOp<T> {
    Load,
    Clear(T),
}

impl<T> LoadOp<T> {
    pub(crate) fn op_to_web_sys(&self) -> GpuLoadOp {
        match self {
            LoadOp::Load => GpuLoadOp::Load,
            LoadOp::Clear(_) => GpuLoadOp::Clear,
        }
    }
}

impl LoadOp<[f32; 4]> {
    pub(crate) fn value_to_web_sys(&self) -> Option<GpuColorDict> {
        match self {
            LoadOp::Load => None,
            LoadOp::Clear([r, g, b, a]) => Some(GpuColorDict::new(
                *r as f64, *g as f64, *b as f64, *a as f64,
            )),
        }
    }
}

impl LoadOp<[i32; 4]> {
    pub(crate) fn value_to_web_sys(&self) -> Option<GpuColorDict> {
        match self {
            LoadOp::Load => None,
            LoadOp::Clear([r, g, b, a]) => Some(GpuColorDict::new(
                *r as f64, *g as f64, *b as f64, *a as f64,
            )),
        }
    }
}

impl LoadOp<[u32; 4]> {
    pub(crate) fn value_to_web_sys(&self) -> Option<GpuColorDict> {
        match self {
            LoadOp::Load => None,
            LoadOp::Clear([r, g, b, a]) => Some(GpuColorDict::new(
                *r as f64, *g as f64, *b as f64, *a as f64,
            )),
        }
    }
}

impl LoadOp<DepthValue> {
    pub(crate) fn value_to_web_sys(&self) -> Option<f32> {
        match self {
            LoadOp::Load => None,
            LoadOp::Clear(v) => Some(v.0),
        }
    }
}

impl LoadOp<u32> {
    pub(crate) fn value_to_web_sys(&self) -> Option<u32> {
        match self {
            LoadOp::Load => None,
            LoadOp::Clear(v) => Some(*v),
        }
    }
}

pub enum StoreOp {
    Store,
    Discard,
}

impl StoreOp {
    pub(crate) fn to_web_sys(&self) -> GpuStoreOp {
        match self {
            StoreOp::Store => GpuStoreOp::Store,
            StoreOp::Discard => GpuStoreOp::Discard,
        }
    }
}

pub struct ColorTargetEncoding {
    pub(crate) inner: web_sys::GpuRenderPassColorAttachment,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) _texture_destroyer: Arc<TextureDestroyer>,
}

pub struct DepthStencilTargetEncoding {
    pub(crate) inner: web_sys::GpuRenderPassDepthStencilAttachment,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) _texture_destroyer: Arc<TextureDestroyer>,
}
