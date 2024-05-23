mod attachment;
pub use self::attachment::*;

mod multisample_attachment;
pub use self::attachment::*;

mod render_layout;
pub use self::render_layout::*;

mod render_target;
use std::error::Error;
use std::fmt;

pub use self::render_target::*;
use crate::driver::{Dvr, RenderPassColorAttachment, RenderPassDepthStencilAttachment};

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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LoadOp<T> {
    Load,
    Clear(T),
}

impl LoadOp<[f32; 4]> {
    pub(crate) fn to_4xf64(&self) -> LoadOp<[f64; 4]> {
        match self {
            LoadOp::Load => LoadOp::Load,
            LoadOp::Clear([r, g, b, a]) => {
                LoadOp::Clear([*r as f64, *g as f64, *b as f64, *a as f64])
            }
        }
    }
}

impl LoadOp<[i32; 4]> {
    pub(crate) fn to_4xf64(&self) -> LoadOp<[f64; 4]> {
        match self {
            LoadOp::Load => LoadOp::Load,
            LoadOp::Clear([r, g, b, a]) => {
                LoadOp::Clear([*r as f64, *g as f64, *b as f64, *a as f64])
            }
        }
    }
}

impl LoadOp<[u32; 4]> {
    pub(crate) fn to_4xf64(&self) -> LoadOp<[f64; 4]> {
        match self {
            LoadOp::Load => LoadOp::Load,
            LoadOp::Clear([r, g, b, a]) => {
                LoadOp::Clear([*r as f64, *g as f64, *b as f64, *a as f64])
            }
        }
    }
}

impl LoadOp<DepthValue> {
    pub(crate) fn to_f32(&self) -> LoadOp<f32> {
        match self {
            LoadOp::Load => LoadOp::Load,
            LoadOp::Clear(v) => LoadOp::Clear(v.0),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StoreOp {
    Store,
    Discard,
}

pub struct ColorTargetEncoding<'a> {
    pub(crate) inner: Option<RenderPassColorAttachment<'a, Dvr>>,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

pub struct DepthStencilTargetEncoding<'a> {
    pub(crate) inner: Option<RenderPassDepthStencilAttachment<'a, Dvr>>,
    pub(crate) width: u32,
    pub(crate) height: u32,
}
