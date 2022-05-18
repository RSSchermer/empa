mod attachment;
pub use self::attachment::*;

mod multisample_attachment;
pub use self::attachment::*;

mod render_layout;
pub use self::render_layout::*;

mod render_target;
pub use self::render_target::*;

pub enum LoadOp<T> {
    Load,
    Clear(T),
}

pub enum StoreOp {
    Store,
    Discard,
}

pub struct ColorTargetEncoding {
    pub(crate) inner: web_sys::GpuRenderPassColorAttachment,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

pub struct DepthStencilTargetEncoding {
    pub(crate) inner: web_sys::GpuRenderPassDepthStencilAttachment,
    pub(crate) width: u32,
    pub(crate) height: u32,
}
