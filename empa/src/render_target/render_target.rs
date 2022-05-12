use crate::render_target::multisample_attachment::{
    MultisampleColorTargets, MultisampleDepthStencilTarget,
};
use crate::render_target::{
    ColorTargets, DepthStencilTarget, MultisampleRenderLayout, RenderLayout, TypedRenderLayout,
};
use web_sys::GpuRenderPassDepthStencilAttachment;

mod valid_render_target_seal {
    pub trait Seal {}
}

pub trait ValidRenderTarget: valid_render_target_seal::Seal {
    type RenderLayout: TypedRenderLayout;

    fn encoding(&self) -> RenderTargetEncoding;
}

pub struct RenderTargetEncoding {
    pub(crate) color_attachments: js_sys::Array,
    pub(crate) depth_stencil_attachment: Option<GpuRenderPassDepthStencilAttachment>,
}

pub struct RenderTarget<C, Ds> {
    pub color: C,
    pub depth_stencil: Ds,
}

impl<C, Ds> valid_render_target_seal::Seal for RenderTarget<C, Ds>
where
    C: ColorTargets,
    Ds: DepthStencilTarget,
{
}
impl<C, Ds> ValidRenderTarget for RenderTarget<C, Ds>
where
    C: ColorTargets,
    Ds: DepthStencilTarget,
{
    type RenderLayout = RenderLayout<C::Layout, Ds::Format>;

    fn encoding(&self) -> RenderTargetEncoding {
        let RenderTarget {
            color,
            depth_stencil,
        } = self;

        let color_attachments = color
            .encodings()
            .map(|e| e.inner)
            .collect::<js_sys::Array>();
        let depth_stencil_attachment = depth_stencil.to_encoding().inner;

        RenderTargetEncoding {
            color_attachments,
            depth_stencil_attachment: Some(depth_stencil_attachment),
        }
    }
}

impl<C> valid_render_target_seal::Seal for RenderTarget<C, ()> where C: ColorTargets {}
impl<C> ValidRenderTarget for RenderTarget<C, ()>
where
    C: ColorTargets,
{
    type RenderLayout = RenderLayout<C::Layout, ()>;

    fn encoding(&self) -> RenderTargetEncoding {
        let color_attachments = self
            .color
            .encodings()
            .map(|e| e.inner)
            .collect::<js_sys::Array>();

        RenderTargetEncoding {
            color_attachments,
            depth_stencil_attachment: None,
        }
    }
}

impl<Ds> valid_render_target_seal::Seal for RenderTarget<(), Ds> where Ds: DepthStencilTarget {}
impl<Ds> ValidRenderTarget for RenderTarget<(), Ds>
where
    Ds: DepthStencilTarget,
{
    type RenderLayout = RenderLayout<(), Ds::Format>;

    fn encoding(&self) -> RenderTargetEncoding {
        let depth_stencil_attachment = self.depth_stencil.to_encoding().inner;

        RenderTargetEncoding {
            color_attachments: js_sys::Array::new(),
            depth_stencil_attachment: Some(depth_stencil_attachment),
        }
    }
}

pub struct MultisampleRenderTarget<C, Ds, const SAMPLES: u8> {
    pub color: C,
    pub depth_stencil: Ds,
}

impl<C, Ds, const SAMPLES: u8> valid_render_target_seal::Seal
    for MultisampleRenderTarget<C, Ds, SAMPLES>
where
    C: MultisampleColorTargets<SAMPLES>,
    Ds: MultisampleDepthStencilTarget<SAMPLES>,
{
}
impl<C, Ds, const SAMPLES: u8> ValidRenderTarget for MultisampleRenderTarget<C, Ds, SAMPLES>
where
    C: MultisampleColorTargets<SAMPLES>,
    Ds: MultisampleDepthStencilTarget<SAMPLES>,
{
    type RenderLayout = MultisampleRenderLayout<C::Layout, Ds::Format, SAMPLES>;

    fn encoding(&self) -> RenderTargetEncoding {
        let MultisampleRenderTarget {
            color,
            depth_stencil,
        } = self;

        let color_attachments = color
            .encodings()
            .map(|e| e.inner)
            .collect::<js_sys::Array>();
        let depth_stencil_attachment = depth_stencil.to_encoding().inner;

        RenderTargetEncoding {
            color_attachments,
            depth_stencil_attachment: Some(depth_stencil_attachment),
        }
    }
}

impl<C, const SAMPLES: u8> valid_render_target_seal::Seal
    for MultisampleRenderTarget<C, (), SAMPLES>
where
    C: MultisampleColorTargets<SAMPLES>,
{
}
impl<C, const SAMPLES: u8> ValidRenderTarget for MultisampleRenderTarget<C, (), SAMPLES>
where
    C: MultisampleColorTargets<SAMPLES>,
{
    type RenderLayout = MultisampleRenderLayout<C::Layout, (), SAMPLES>;

    fn encoding(&self) -> RenderTargetEncoding {
        let color_attachments = self
            .color
            .encodings()
            .map(|e| e.inner)
            .collect::<js_sys::Array>();

        RenderTargetEncoding {
            color_attachments,
            depth_stencil_attachment: None,
        }
    }
}

impl<Ds, const SAMPLES: u8> valid_render_target_seal::Seal
    for MultisampleRenderTarget<(), Ds, SAMPLES>
where
    Ds: MultisampleDepthStencilTarget<SAMPLES>,
{
}
impl<Ds, const SAMPLES: u8> ValidRenderTarget for MultisampleRenderTarget<(), Ds, SAMPLES>
where
    Ds: MultisampleDepthStencilTarget<SAMPLES>,
{
    type RenderLayout = MultisampleRenderLayout<(), Ds::Format, SAMPLES>;

    fn encoding(&self) -> RenderTargetEncoding {
        let depth_stencil_attachment = self.depth_stencil.to_encoding().inner;

        RenderTargetEncoding {
            color_attachments: js_sys::Array::new(),
            depth_stencil_attachment: Some(depth_stencil_attachment),
        }
    }
}
