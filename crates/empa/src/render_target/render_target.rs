use staticvec::StaticVec;

use crate::render_target::multisample_attachment::{
    MultisampleColorTargets, MultisampleDepthStencilTarget,
};
use crate::render_target::{
    ColorTargetEncoding, ColorTargets, DepthStencilTarget, DepthStencilTargetEncoding,
    MultisampleRenderLayout, RenderLayout, TypedRenderLayout,
};

mod valid_render_target_seal {
    pub trait Seal {}
}

pub trait ValidRenderTarget: valid_render_target_seal::Seal {
    type RenderLayout: TypedRenderLayout;

    fn encoding(&self) -> RenderTargetEncoding;
}

pub struct RenderTargetEncoding {
    pub(crate) color_attachments: StaticVec<ColorTargetEncoding, 8>,
    pub(crate) depth_stencil_attachment: Option<DepthStencilTargetEncoding>,
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

        RenderTargetEncoding {
            color_attachments: color.encodings().collect(),
            depth_stencil_attachment: Some(depth_stencil.to_encoding()),
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
        RenderTargetEncoding {
            color_attachments: self.color.encodings().collect(),
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
        RenderTargetEncoding {
            color_attachments: StaticVec::new(),
            depth_stencil_attachment: Some(self.depth_stencil.to_encoding()),
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

        RenderTargetEncoding {
            color_attachments: color.encodings().collect(),
            depth_stencil_attachment: Some(depth_stencil.to_encoding()),
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
        RenderTargetEncoding {
            color_attachments: self.color.encodings().collect(),
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
        RenderTargetEncoding {
            color_attachments: StaticVec::new(),
            depth_stencil_attachment: Some(self.depth_stencil.to_encoding()),
        }
    }
}
