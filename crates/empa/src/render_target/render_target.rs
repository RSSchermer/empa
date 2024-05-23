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

    type ColorTargetEncodings<'a>: IntoIterator<Item = ColorTargetEncoding<'a>>
    where
        Self: 'a;

    fn color_target_encodings<'a>(&'a self) -> Self::ColorTargetEncodings<'a>;

    fn depth_stencil_target_encoding(&self) -> DepthStencilTargetEncoding;
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

    type ColorTargetEncodings<'a> = C::Encodings<'a> where Self: 'a;

    fn color_target_encodings<'a>(&'a self) -> Self::ColorTargetEncodings<'a> {
        self.color.encodings()
    }

    fn depth_stencil_target_encoding(&self) -> DepthStencilTargetEncoding {
        self.depth_stencil.to_encoding()
    }
}

impl<C> valid_render_target_seal::Seal for RenderTarget<C, ()> where C: ColorTargets {}
impl<C> ValidRenderTarget for RenderTarget<C, ()>
where
    C: ColorTargets,
{
    type RenderLayout = RenderLayout<C::Layout, ()>;

    type ColorTargetEncodings<'a> = C::Encodings<'a> where C: 'a;

    fn color_target_encodings<'a>(&'a self) -> Self::ColorTargetEncodings<'a> {
        self.color.encodings()
    }

    fn depth_stencil_target_encoding(&self) -> DepthStencilTargetEncoding {
        DepthStencilTargetEncoding {
            inner: None,
            width: 0,
            height: 0,
        }
    }
}

impl<Ds> valid_render_target_seal::Seal for RenderTarget<(), Ds> where Ds: DepthStencilTarget {}
impl<Ds> ValidRenderTarget for RenderTarget<(), Ds>
where
    Ds: DepthStencilTarget,
{
    type RenderLayout = RenderLayout<(), Ds::Format>;

    type ColorTargetEncodings<'a> = [ColorTargetEncoding<'a>; 0] where Ds: 'a;

    fn color_target_encodings<'a>(&'a self) -> Self::ColorTargetEncodings<'a> {
        []
    }

    fn depth_stencil_target_encoding(&self) -> DepthStencilTargetEncoding {
        self.depth_stencil.to_encoding()
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

    type ColorTargetEncodings<'a> = C::Encodings<'a> where Self: 'a;

    fn color_target_encodings<'a>(&'a self) -> Self::ColorTargetEncodings<'a> {
        self.color.encodings()
    }

    fn depth_stencil_target_encoding(&self) -> DepthStencilTargetEncoding {
        self.depth_stencil.to_encoding()
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

    type ColorTargetEncodings<'a> = C::Encodings<'a> where C: 'a;

    fn color_target_encodings<'a>(&'a self) -> Self::ColorTargetEncodings<'a> {
        self.color.encodings()
    }

    fn depth_stencil_target_encoding(&self) -> DepthStencilTargetEncoding {
        DepthStencilTargetEncoding {
            inner: None,
            width: 0,
            height: 0,
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

    type ColorTargetEncodings<'a> = [ColorTargetEncoding<'a>; 0] where Ds: 'a;

    fn color_target_encodings<'a>(&'a self) -> Self::ColorTargetEncodings<'a> {
        []
    }

    fn depth_stencil_target_encoding(&self) -> DepthStencilTargetEncoding {
        self.depth_stencil.to_encoding()
    }
}
